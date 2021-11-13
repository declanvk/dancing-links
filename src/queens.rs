//! The [`n` queens puzzle](https://en.wikipedia.org/wiki/Eight_queens_puzzle)
//!  is the problem of placing `n` chess queens on an `n`×`n` chessboard so that
//! no two queens threaten each other.
//!
//! A solution to the problem requires that no two queens share the same row,
//! column, or diagonal.

use crate::ExactCover;
#[cfg(fuzzing)]
use arbitrary::Arbitrary;
use std::collections::HashSet;

/// An instance of the `n` queens problem.
#[derive(Debug)]
#[cfg_attr(fuzzing, derive(Arbitrary))]
pub struct NQueens {
    /// The list of possible positions that could solve the `n` queens puzzle.
    pub possibilities: Vec<Possibility>,
    /// The list of constraints that must be satisfied for this `n` queens
    /// puzzle.
    pub constraints: Vec<Constraint>,
    /// The length of the chess board side, equal to `n`.
    pub side_length: usize,
    /// The list of values and positions that are given as fixed when the puzzle
    /// is created.
    pub filled_values: Vec<Possibility>,
}

impl NQueens {
    /// Create a new instance of the `n` queens problem with the given filled
    /// values and side length.
    pub fn new(side_length: usize, filled_values: impl IntoIterator<Item = Possibility>) -> Self {
        let filled_values: Vec<_> = filled_values.into_iter().collect();

        let satisfied: HashSet<_> = filled_values
            .iter()
            .copied()
            .flat_map(|poss| poss.satisfied_constraints(side_length))
            .collect();

        let filled_coordinates: HashSet<_> = filled_values
            .iter()
            .map(|poss| (poss.row, poss.column))
            .collect();

        let possibilities: Vec<_> = Possibility::all(side_length)
            .filter(|poss| !filled_coordinates.contains(&(poss.row, poss.column)))
            .collect();

        let constraints = Constraint::all(side_length)
            .filter(|cons| !satisfied.contains(cons))
            .collect();

        Self {
            possibilities,
            constraints,
            side_length,
            filled_values,
        }
    }
}

impl ExactCover for NQueens {
    type Constraint = Constraint;
    type Possibility = Possibility;

    fn satisfies(&self, poss: &Self::Possibility, cons: &Self::Constraint) -> bool {
        use Constraint::*;

        match cons {
            Row { index } => poss.row == *index,
            Column { index } => poss.column == *index,
            LeadingDiagonal { index } => poss.leading_diagonal(self.side_length) == *index,
            TrailingDiagonal { index } => poss.trailing_diagonal() == *index,
        }
    }

    fn is_optional(&self, cons: &Self::Constraint) -> bool {
        matches!(
            cons,
            Constraint::LeadingDiagonal { .. } | Constraint::TrailingDiagonal { .. }
        )
    }

    fn possibilities(&self) -> &[Self::Possibility] {
        &self.possibilities
    }

    fn constraints(&self) -> &[Self::Constraint] {
        &self.constraints
    }
}

/// A position on the chess board.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(fuzzing, derive(Arbitrary))]
pub struct Possibility {
    /// The row index, ranging from 0 to `n - 1`.
    row: usize,
    /// The column index, ranging form 0 to `n - 1`.
    column: usize,
}

impl Possibility {
    /// Return an iterator over all positions on the chess board for a given
    /// side length.
    pub fn all(side_length: usize) -> impl Iterator<Item = Self> {
        crate::util::two_combination_iter([side_length, side_length], [0, 0])
            .map(|[column, row]| Possibility { row, column })
    }

    /// Return the leading diagonal index for a given side length.
    ///
    /// This value ranges from 0 to `n - 2`.
    pub fn leading_diagonal(self, side_length: usize) -> usize {
        ((self.column as i128 - self.row as i128) + (side_length - 1) as i128) as usize
    }

    /// Return the trailing diagonal index.
    ///
    /// The value ranges from 0 to `n - 2`.
    pub fn trailing_diagonal(self) -> usize {
        self.row + self.column
    }

    /// Return an iterator over all the `Constraint`s that are satisfied by this
    /// `Possibility`.
    pub fn satisfied_constraints(self, side_length: usize) -> impl Iterator<Item = Constraint> {
        [
            Constraint::Row { index: self.row },
            Constraint::Column { index: self.column },
            Constraint::LeadingDiagonal {
                index: self.leading_diagonal(side_length),
            },
            Constraint::TrailingDiagonal {
                index: self.trailing_diagonal(),
            },
        ]
        .into_iter()
    }
}

/// A condition which must be satisfied in order to solve an `n` queens puzzle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(fuzzing, derive(Arbitrary))]
pub enum Constraint {
    /// A condition that a given row should have exactly one queen.
    Row {
        /// The row index
        index: usize,
    },
    /// A condition that a given column should have exactly one queen.
    Column {
        /// The column index
        index: usize,
    },
    /// A condition that a leading diagonal should have at most one queen.
    LeadingDiagonal {
        /// The leading diagonal index
        index: usize,
    },
    /// A condition that a trailing diagonal should have at most one queen.
    TrailingDiagonal {
        /// The trailing diagonal index
        index: usize,
    },
}

impl Constraint {
    /// Return an iterator over all possible `Constraint`s for a given
    /// `side_length`.
    pub fn all(side_length: usize) -> impl Iterator<Item = Constraint> {
        let row_it = (0..side_length).map(|index| Constraint::Row { index });
        let column_it = (0..side_length).map(|index| Constraint::Column { index });
        let leading_it =
            (0..(2 * side_length - 1)).map(|index| Constraint::LeadingDiagonal { index });
        let trailing_it =
            (0..(2 * side_length - 1)).map(|index| Constraint::TrailingDiagonal { index });

        row_it.chain(column_it).chain(leading_it).chain(trailing_it)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter;

    fn p(row: usize, column: usize) -> Possibility {
        Possibility { row, column }
    }

    #[test]
    fn check_diagonal_indices() {
        let side_length = 8;
        let leading_possibilities_it = (0..side_length)
            .rev()
            .map(|row| Possibility { row, column: 0 })
            .chain((1..side_length).map(|column| Possibility { row: 0, column }));

        let leading_diagonal_indices: Vec<_> = leading_possibilities_it
            .map(|poss| poss.leading_diagonal(side_length))
            .collect();

        assert_eq!(leading_diagonal_indices, (0..15).collect::<Vec<_>>());

        let trailing_possibilities_it = (0..side_length)
            .map(|column| Possibility { column, row: 0 })
            .chain((1..side_length).map(|row| Possibility {
                row,
                column: side_length - 1,
            }));

        let trailing_diagonal_indices: Vec<_> = trailing_possibilities_it
            .map(|poss| poss.trailing_diagonal())
            .collect();
        assert_eq!(trailing_diagonal_indices, (0..15).collect::<Vec<_>>());
    }

    #[test]
    fn check_tiny_boards() {
        let size_one_board = NQueens::new(1, iter::empty());
        let size_one_solutions = size_one_board.solver().all_solutions();

        assert_eq!(size_one_solutions.len(), 1);
        assert_eq!(size_one_solutions[0], vec![&p(0, 0)]);

        let size_two_board = NQueens::new(2, iter::empty());
        assert_eq!(size_two_board.solver().count(), 0);

        let size_three_board = NQueens::new(3, iter::empty());
        assert_eq!(size_three_board.solver().count(), 0);
    }

    #[test]
    fn check_small_board() {
        let queens = NQueens::new(4, iter::empty());
        let mut solver = queens.solver();

        let mut first_solution = solver.next().unwrap();
        first_solution.sort();
        assert_eq!(first_solution, vec![&p(0, 1), &p(1, 3), &p(2, 0), &p(3, 2)]);

        let mut second_solution = solver.next().unwrap();
        second_solution.sort();
        assert_eq!(
            second_solution,
            vec![&p(0, 2), &p(1, 0), &p(2, 3), &p(3, 1)]
        );

        assert!(solver.next().is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore)] // takes too long on miri
    fn count_medium_board() {
        let queens = NQueens::new(8, iter::empty());

        assert_eq!(queens.solver().count(), 92);
    }
}
