use std::{fmt::Debug, hash::Hash, ptr::NonNull};

use super::{
    grid::{Column, GridRoot, Node}, Possibility,
};

#[derive(Debug, PartialEq)]
pub struct Solver<P: Hash + Eq, C: Hash + Eq> {
    root: Box<GridRoot<P, C>>,
}

pub type Solution<P> = Vec<P>;

impl<P> Solver<P, P::Constraint>
where
    P: Possibility + Debug,
    P::Constraint: Debug,
{
    pub fn new(possibilities: &[P]) -> Self {
        let mut root = GridRoot::new();

        for possibility in possibilities {
            root.add_possibility(possibility.clone());
        }

        Solver { root }
    }

    pub fn first_solution(&mut self) -> Option<Solution<P>> {
        Solver::first_solution_recurse(&mut self.root, 0, Solution::new());

        None
    }

    fn first_solution_recurse(
        root: &mut GridRoot<P, P::Constraint>,
        k: usize,
        mut solution: Solution<P>,
    ) where
        P: Debug,
        P::Constraint: Debug,
    {
        eprintln!("\nLevel {}", k);

        let root_ptr = root.base.self_ptr();
        if root.base.right == root_ptr {
            eprintln!("Solution {:?}", solution);
        } else {
            unsafe {
                let (mut min_column, size) = Solver::choose_min_column(root);
                if size == 0 {
                    // If there exists a constraint with no viable rows, then this particular
                    // branch does not contain any solutions
                    return;
                }

                min_column.as_mut().cover();

                min_column
                    .as_mut()
                    .base
                    .apply_down(|_min_column_ptr, mut current_node| {
                        let mut node_ptr = current_node.cast::<Node<P, P::Constraint>>();
                        let node = node_ptr.as_mut();

                        // Add current row to solution
                        solution.push(node.possibility.clone());

                        // Removing constraints that are satisfied by this row
                        current_node.as_mut().apply_right(|_self, current_node| {
                            let row_node_ptr = current_node.cast::<Node<P, P::Constraint>>();
                            let row_node = row_node_ptr.as_ref();
                            let mut row_column_ptr =
                                row_node.column.cast::<Column<P, P::Constraint>>();
                            let row_column = row_column_ptr.as_mut();

                            row_column.cover();
                        });

                        eprintln!("Recursing");
                        Solver::first_solution_recurse(root, k + 1, solution.clone());
                        eprintln!("\nLevel {}", k);

                        eprintln!("Popping solution");
                        let _possibility = solution.pop().unwrap();

                        eprintln!(
                            "Uncovering node columns adjacent to Node at {}",
                            node.base.id
                        );

                        current_node.as_mut().apply_left(|_self, current_node| {
                            let row_node_ptr = current_node.cast::<Node<P, P::Constraint>>();
                            let row_node = row_node_ptr.as_ref();
                            let mut row_column_ptr =
                                row_node.column.cast::<Column<P, P::Constraint>>();
                            let row_column = row_column_ptr.as_mut();

                            eprintln!("Uncovering {}", row_column.display());
                            row_column.uncover();
                        });

                        eprintln!("Finished row at {:?}", current_node);
                        eprintln!("State:");
                        eprint!("{}", root.display());
                    });

                min_column.as_mut().uncover();
                eprintln!("Uncovered {:?}", min_column.as_ref());
                eprintln!("State:");
                eprint!("{}", root.display());
            }
        }
    }

    unsafe fn choose_min_column(
        root: &mut GridRoot<P, P::Constraint>,
    ) -> (NonNull<Column<P, P::Constraint>>, usize) {
        let mut min_column = root.base.right.cast::<Column<P, P::Constraint>>();
        let mut min_size = min_column.as_ref().size;

        root.base.apply_right(|_self, current_ptr| {
            let current_column = current_ptr.cast::<Column<P, P::Constraint>>();
            if current_column.as_ref().size < min_size {
                min_column = current_column;
                min_size = current_column.as_ref().size;
            }
        });

        (min_column, min_size)
    }
}
