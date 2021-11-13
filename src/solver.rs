use crate::{
    grid::{Column, Grid, Node},
    ExactCover,
};
use core::iter;
use std::collections::VecDeque;

/// Solver that iteratively returns solutions to exact cover problems.
#[derive(Debug)]
pub struct Solver<'e, E: ExactCover> {
    problem: &'e E,

    // Values used to track the state of solving
    grid: Grid,
    partial_solution: Vec<usize>,
    stack: Vec<Frame>,
}

#[derive(Debug)]
enum FrameState {
    // Before covering one of the rows
    Cover,
    // After checking, before uncovering
    Uncover,
}

#[derive(Debug)]
struct Frame {
    #[allow(dead_code)]
    min_column: *mut Column,
    selected_rows: VecDeque<(usize, Vec<*mut Column>)>,
    state: FrameState,
}

impl<'e, E> Solver<'e, E>
where
    E: ExactCover,
{
    /// Create a new `Solver` with the given instance of an exact cover problem.
    pub fn new(problem: &'e E) -> Self {
        let grid = Self::populate_grid(problem);

        let mut solver = Self {
            problem,

            grid,
            partial_solution: Vec::new(),
            stack: Vec::new(),
        };

        // If the grid is already solved (no primary columns), don't bother to put a
        // stack frame in
        if !Self::solution_test(&solver.grid, &solver.problem) {
            let min_column = Self::choose_column(&mut solver.grid, &solver.problem);
            let selected_rows = Self::select_rows_from_column(min_column);

            if !selected_rows.is_empty() {
                solver.stack.push(Frame {
                    state: FrameState::Cover,
                    min_column,
                    selected_rows,
                });
            }
        }

        solver
    }

    /// Reset all solver state except for the stored possibilities and
    /// constraints.
    pub fn reset(&mut self) {
        self.grid = Self::populate_grid(&self.problem);
        self.partial_solution.clear();
        self.stack.clear();
    }

    fn populate_grid(problem: &E) -> Grid {
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

        Grid::new(problem.constraints().len(), coordinates_iter)
    }

    /// Return true if the current grid represents a valid solution.
    ///
    /// This implementation determines that the grid represents a solution if
    /// there are only optional columns left uncovered in the grid.
    fn solution_test(grid: &Grid, problem: &E) -> bool {
        !grid
            .uncovered_columns()
            .any(|column| !problem.is_optional(&problem.constraints()[Column::index(column) - 1]))
    }

    /// Select a column to remove from the matrix.
    ///
    /// This implementation chooses the non-optional column that has the least
    /// number of entries uncovered in the grid.
    fn choose_column(grid: &mut Grid, problem: &E) -> *mut Column {
        grid.uncovered_columns_mut()
            .filter(|column| {
                !problem.is_optional(&problem.constraints()[Column::index(*column as *const _) - 1])
            })
            .min_by_key(|column_ptr| Column::size(*column_ptr))
            .unwrap()
    }

    /// Return a list of rows that are uncovered and present in the given
    /// column.
    fn select_rows_from_column(min_column: *mut Column) -> VecDeque<(usize, Vec<*mut Column>)> {
        Column::rows(min_column)
            .map(|node_ptr| {
                (
                    Node::row_index(node_ptr),
                    Node::neighbors(node_ptr)
                        .map(Node::column_ptr)
                        .chain(iter::once(Node::column_ptr(node_ptr)))
                        .collect(),
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
                        Column::cover(*column_ptr);
                    }

                    // This is where the recursion happens, but we also have to check for the
                    // solution here.
                    let stack_op = if Self::solution_test(&self.grid, &self.problem) {
                        (StackOp::None, Some(self.partial_solution.clone()))
                    } else {
                        let min_column = Self::choose_column(&mut self.grid, &self.problem);
                        let selected_rows = Self::select_rows_from_column(min_column);

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
                    let (_row_index, columns) = curr_frame.selected_rows.pop_front().unwrap();

                    for column_ptr in columns {
                        Column::uncover(column_ptr);
                    }
                    self.partial_solution.pop();

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

impl<'e, E> Iterator for Solver<'e, E>
where
    E: ExactCover,
{
    type Item = Vec<&'e E::Possibility>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_solution()
    }
}
