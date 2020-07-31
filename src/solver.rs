use crate::{
    grid::{Column, Grid, Node},
    ExactCover,
};
use core::iter;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Solver<'e, E: ExactCover> {
    possibilities: &'e [E::Possibility],
    constraints: &'e [E::Constraint],

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
    min_column: *mut Column,
    selected_rows: VecDeque<(usize, Vec<*mut Column>)>,
    state: FrameState,
}

impl<'e, E> Solver<'e, E>
where
    E: ExactCover,
{
    /// Create a new Solver with the given possibilities and constraints.
    pub fn new(possibilities: &'e [E::Possibility], constraints: &'e [E::Constraint]) -> Self {
        let grid = Self::populate_grid(possibilities, constraints);

        let mut solver = Solver {
            possibilities,
            constraints,

            grid,
            partial_solution: Vec::new(),
            stack: Vec::new(),
        };

        // If the grid is already solved (no primary columns), don't bother to put a
        // stack frame in
        if !Self::solution_test(&solver.grid) {
            let min_column = Self::choose_column(&mut solver.grid);
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
        self.grid = Self::populate_grid(&self.possibilities, &self.constraints);
        self.partial_solution.clear();
        self.stack.clear();
    }

    fn populate_grid(possibilities: &[E::Possibility], constraints: &[E::Constraint]) -> Grid {
        let coordinates_iter = possibilities
            .iter()
            .enumerate()
            .flat_map({
                move |(row_idx, poss)| {
                    constraints
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
                if E::satisfies(poss, cons) {
                    Some(coord)
                } else {
                    None
                }
            });

        Grid::new(constraints.len(), coordinates_iter)
    }

    fn solution_test(grid: &Grid) -> bool {
        grid.is_empty()

        // !self
        //     .grid
        //     .uncovered_columns()
        //     .any(|column| !E::is_optional(&self.constraints[column.index()]))
    }

    fn choose_column(grid: &mut Grid) -> *mut Column {
        grid.uncovered_columns_mut()
            //     .filter(|column| !E::is_optional(&self.constraints[column.index()]))
            .min_by_key(|column_ptr| Column::size(*column_ptr))
            .unwrap()
    }

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

    pub fn all_solutions(&mut self) -> Vec<Vec<&E::Possibility>> {
        self.collect()
    }

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
                    let stack_op = if Self::solution_test(&self.grid) {
                        (StackOp::None, Some(self.partial_solution.clone()))
                    } else {
                        let min_column = Self::choose_column(&mut self.grid);
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
                        .map(|row_index| &self.possibilities[row_index])
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
