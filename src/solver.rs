use crate::{sparse_grid::SparseGrid, ExactCover, Grid};
use std::collections::VecDeque;

/// Solver that iteratively returns solutions to exact cover problems.
#[derive(Debug)]
pub struct Solver<'e, E: ExactCover, G: Grid = SparseGrid> {
    problem: &'e E,

    // Values used to track the state of solving
    grid: G,
    partial_solution: Vec<usize>,
    stack: Vec<Frame<G>>,
}

#[derive(Debug)]
enum FrameState {
    // Before covering one of the rows
    Cover,
    // After checking, before uncovering
    Uncover,
}

#[derive(Debug)]
struct Frame<G: Grid> {
    #[allow(dead_code)]
    min_column: G::Column,
    selected_rows: VecDeque<(usize, Vec<G::Column>)>,
    state: FrameState,
}

impl<'e, E> Solver<'e, E>
where
    E: ExactCover,
{
    /// Create a new `Solver` with the given instance of an exact cover problem.
    pub fn new(problem: &'e E) -> Self {
        Self::with_grid(problem)
    }
}

impl<'e, E, G> Solver<'e, E, G>
where
    E: ExactCover,
    G: Grid,
{
    /// Create a new `Solver` with the given instance of an exact cover problem
    /// and the specified `Grid` implementation.
    pub fn with_grid(problem: &'e E) -> Self {
        let grid = Self::populate_grid(problem);

        let mut solver = Self {
            problem,

            grid,
            partial_solution: Vec::new(),
            stack: Vec::new(),
        };

        solver.reset();

        solver
    }

    /// Reset all solver state except for the stored possibilities and
    /// constraints.
    pub fn reset(&mut self) {
        self.grid = Self::populate_grid(self.problem);
        self.partial_solution.clear();
        self.stack.clear();

        // If the grid is already solved (no primary columns), don't bother to put a
        // stack frame in
        if !Self::solution_test(&self.grid, self.problem) {
            let min_column = Self::choose_column(&mut self.grid, self.problem);
            let selected_rows = Self::select_rows_from_column(&self.grid, min_column);

            if !selected_rows.is_empty() {
                self.stack.push(Frame {
                    state: FrameState::Cover,
                    min_column,
                    selected_rows,
                });
            }
        }
    }

    fn populate_grid(problem: &E) -> G {
        let coordinates_iter = problem
            .possibilities()
            .iter()
            .enumerate()
            .flat_map({
                move |(row_idx, poss)| {
                    problem
                        .constraints()
                        .iter()
                        .enumerate()
                        .zip(iter::repeat((row_idx, poss)))
                        .map({
                            |((col_idx, cons), (row_idx, poss))| {
                                ((row_idx + 1, col_idx + 1), poss, cons)
                            }
                        })
                }
            })
            .filter_map(|(coord, poss, cons)| {
                if problem.satisfies(poss, cons) {
                    Some(coord)
                } else {
                    None
                }
            });

        G::new(problem.constraints().len(), coordinates_iter)
    }

    /// Return true if the current grid represents a valid solution.
    ///
    /// This implementation determines that the grid represents a solution if
    /// there are only optional columns left uncovered in the grid.
    fn solution_test(grid: &G, problem: &E) -> bool {
        !grid
            .uncovered_columns()
            .any(|column| !problem.is_optional(&problem.constraints()[grid.column_id(column)]))
    }

    /// Select a column to remove from the matrix.
    ///
    /// This implementation chooses the non-optional column that has the least
    /// number of entries uncovered in the grid.
    fn choose_column(grid: &mut G, problem: &E) -> G::Column {
        grid.uncovered_columns()
            .filter(|column| !problem.is_optional(&problem.constraints()[grid.column_id(*column)]))
            .min_by_key(|column_ptr| grid.column_size(*column_ptr))
            .unwrap()
    }

    /// Return a list of rows that are uncovered and present in the given
    /// column.
    fn select_rows_from_column(
        grid: &G,
        min_column: G::Column,
    ) -> VecDeque<(usize, Vec<G::Column>)> {
        grid.uncovered_rows_in_column(min_column)
            .map(|node_ptr| {
                (
                    grid.row_id(node_ptr),
                    grid.uncovered_columns_in_row(node_ptr).collect(),
                )
            })
            .collect()
    }

    /// Return all possible solutions.
    pub fn all_solutions(&mut self) -> Vec<Vec<&'e E::Possibility>> {
        self.collect()
    }

    /// Compute up to the next solution, returning `None` if there are no more.
    pub fn next_solution<'s>(&'s mut self) -> Option<Vec<&'e E::Possibility>>
    where
        'e: 's,
    {
        enum StackOp<T> {
            Push(T),
            Pop,
            None,
        }

        while !self.stack.is_empty() {
            let curr_frame = self.stack.last_mut().unwrap();

            let (stack_op, possible_solution) = match curr_frame.state {
                // for the current row of this frame, cover the selected columns and add the row
                // to the solution.
                FrameState::Cover => {
                    let (row_index, columns) = curr_frame.selected_rows.front().unwrap();

                    self.partial_solution.push(row_index - 1);
                    for column_ptr in columns {
                        self.grid.cover(*column_ptr);
                    }

                    // This is where the recursion happens, but we also have to check for the
                    // solution here.
                    let stack_op = if Self::solution_test(&self.grid, self.problem) {
                        (StackOp::None, Some(self.partial_solution.clone()))
                    } else {
                        let min_column = Self::choose_column(&mut self.grid, self.problem);
                        let selected_rows = Self::select_rows_from_column(&self.grid, min_column);

                        if selected_rows.is_empty() {
                            (StackOp::None, None)
                        } else {
                            (
                                StackOp::Push(Frame {
                                    state: FrameState::Cover,
                                    min_column,
                                    selected_rows,
                                }),
                                None,
                            )
                        }
                    };

                    curr_frame.state = FrameState::Uncover;
                    stack_op
                }
                // Cleanup the current row, uncover the selected columns, remove the row from
                // the solution.
                FrameState::Uncover => {
                    let (row_index, columns) = curr_frame.selected_rows.pop_front().unwrap();

                    for column_ptr in columns {
                        self.grid.uncover(column_ptr);
                    }
                    debug_assert_eq!(self.partial_solution.pop(), Some(row_index - 1));

                    if curr_frame.selected_rows.is_empty() {
                        (StackOp::Pop, None)
                    } else {
                        curr_frame.state = FrameState::Cover;
                        (StackOp::None, None)
                    }
                }
            };

            match stack_op {
                StackOp::Push(val) => {
                    self.stack.push(val);
                }
                StackOp::Pop => {
                    self.stack.pop();
                }
                StackOp::None => {}
            }

            if let Some(solution) = possible_solution {
                return Some(
                    solution
                        .into_iter()
                        .map(|row_index| &self.problem.possibilities()[row_index])
                        .collect(),
                );
            }
        }

        None
    }
}

impl<'e, E, G> Iterator for Solver<'e, E, G>
where
    E: ExactCover,
    G: Grid,
{
    type Item = Vec<&'e E::Possibility>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_solution()
    }
}
