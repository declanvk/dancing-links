//! A [Sudoku puzzle](https://en.wikipedia.org/wiki/Sudoku) is a
//! `n^2` × `n^2` array with sub-arrays of size `n` × `n`. Each row, column, and
//! sub-array contains the values `1` through `n` with no repeats.

use super::{latin_square, ExactCover};
use std::collections::HashSet;

/// An instance of a Sudoku puzzle.
#[derive(Debug)]
pub struct Sudoku {
    /// The list of possible values and positions that are valid for this Sudoku
    /// puzzle.
    pub possibilities: Vec<Possibility>,
    /// The list of constraints that must be satisfied for this Sudoku puzzle.
    pub constraints: Vec<Constraint>,
}

impl Sudoku {
    /// Create a new new Sudoku puzzle.
    ///
    /// The puzzle has size `n^2` × `n^2` (where `n = box_side_length`) and the
    /// given list of filled values.
    pub fn new(
        box_side_length: usize,
        filled_values: impl IntoIterator<Item = latin_square::Possibility>,
    ) -> Self {
        let side_length = box_side_length * box_side_length;
        let filled_values: Vec<_> = filled_values.into_iter().collect();

        let latin = latin_square::LatinSquare::new(side_length, filled_values.iter().copied());

        let satisfied: HashSet<_> = filled_values
            .iter()
            .copied()
            .map(|latin_poss| Possibility::from_latin(latin_poss, box_side_length))
            .flat_map(Possibility::satisfied_constraints)
            .collect();

        let possibilities = latin
            .possibilities
            .into_iter()
            .map(|latin_poss| Possibility::from_latin(latin_poss, box_side_length))
            .collect();

        let constraints = latin
            .constraints
            .into_iter()
            .map(Constraint::from)
            .chain(Constraint::all_square_number(box_side_length))
            .filter(|cons| !satisfied.contains(cons))
            .collect();

        Self {
            possibilities,
            constraints,
        }
    }
}

impl ExactCover for Sudoku {
    type Constraint = Constraint;
    type Possibility = Possibility;

    fn satisfies(&self, poss: &Self::Possibility, cons: &Self::Constraint) -> bool {
        use Constraint::*;

        match cons {
            Latin(latin_cons) => {
                <Possibility as Into<latin_square::Possibility>>::into(*poss).satisfies(latin_cons)
            }
            SquareNumber { square, value } => poss.square == *square && poss.value == *value,
        }
    }

    fn is_optional(&self, _cons: &Self::Constraint) -> bool {
        false
    }

    fn possibilities(&self) -> &[Self::Possibility] {
        &self.possibilities
    }

    fn constraints(&self) -> &[Self::Constraint] {
        &self.constraints
    }
}

/// A position and value for a box inside of a Sudoku puzzle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Possibility {
    /// The row position of the box.
    ///
    /// The values ranges from 0 to `n - 1`, where `n` is the length of the
    /// Sudoku board.
    pub row: usize,

    /// The column position of the box.
    ///
    /// The values ranges from 0 to `n - 1`, where `n` is the length of the
    /// Sudoku board.
    pub column: usize,

    /// The index of the subgrid.
    ///
    /// The values ranges from 0 to `n - 1`, where `n` is the length of the
    /// Sudoku board. This field is redundant in identifying where the box is
    /// inside of the Sudoku board, however it is necessary to speed up checking
    /// which `Constraint`s are satisfied by this `Possibility`.
    pub square: usize,

    /// The value present inside of the box.
    ///
    /// The values ranges from 1 to `n`, where `n` is the length of the
    /// Sudoku board.
    pub value: usize,
}

impl Possibility {
    /// Convert a `latin_square::Possibility` to a `sudoku::Possibility`.
    pub fn from_latin(latin: latin_square::Possibility, box_side_length: usize) -> Self {
        let side_length = box_side_length * box_side_length;
        let index = latin.row * side_length + latin.column;
        let square = ((index % side_length) / box_side_length)
            + box_side_length * (index / (side_length * box_side_length));

        Possibility {
            row: latin.row,
            column: latin.column,
            value: latin.value,
            square,
        }
    }

    /// Return an iterator over the `Constraint`s that are satisfied by this
    /// `Possibility`.
    pub fn satisfied_constraints(self) -> impl Iterator<Item = Constraint> {
        [
            Constraint::Latin(latin_square::Constraint::RowNumber {
                row: self.row,
                value: self.value,
            }),
            Constraint::Latin(latin_square::Constraint::ColumnNumber {
                column: self.column,
                value: self.value,
            }),
            Constraint::Latin(latin_square::Constraint::RowColumn {
                row: self.row,
                column: self.column,
            }),
            Constraint::SquareNumber {
                square: self.square,
                value: self.value,
            },
        ]
        .into_iter()
    }
}

impl Into<latin_square::Possibility> for Possibility {
    fn into(self) -> latin_square::Possibility {
        latin_square::Possibility {
            row: self.row,
            column: self.column,
            value: self.value,
        }
    }
}

/// A condition which must be satisfied in order to solve a Sudoku puzzle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Constraint {
    /// A constraint which is also shared by a Latin Square puzzle.
    Latin(latin_square::Constraint),
    /// A condition that each square (or sub-grid) should only have a single
    /// instance of a numeric value.
    SquareNumber {
        /// The square index.
        square: usize,
        /// The unique numeric value
        value: usize,
    },
}

impl Constraint {
    fn all_square_number(box_side_length: usize) -> impl Iterator<Item = Constraint> {
        let side_length = box_side_length * box_side_length;

        crate::util::two_combination_iter([side_length, side_length + 1], [0, 1])
            .map(|[square, value]| Constraint::SquareNumber { square, value })
    }
}

impl From<latin_square::Constraint> for Constraint {
    fn from(src: latin_square::Constraint) -> Self {
        Constraint::Latin(src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(row: usize, column: usize, square: usize, value: usize) -> Possibility {
        Possibility {
            row,
            column,
            square,
            value,
        }
    }

    fn c_row(row: usize, value: usize) -> Constraint {
        Constraint::Latin(latin_square::Constraint::RowNumber { row, value })
    }

    fn c_col(column: usize, value: usize) -> Constraint {
        Constraint::Latin(latin_square::Constraint::ColumnNumber { column, value })
    }

    fn c_row_col(row: usize, column: usize) -> Constraint {
        Constraint::Latin(latin_square::Constraint::RowColumn { row, column })
    }

    fn c_square(square: usize, value: usize) -> Constraint {
        Constraint::SquareNumber { square, value }
    }

    #[test]
    fn check_generated_possibilities_constraints() {
        let mut sudoku = Sudoku::new(
            2,
            vec![
                // top row
                latin_square::tests::p(0, 0, 1),
                latin_square::tests::p(0, 1, 2),
                latin_square::tests::p(0, 2, 3),
                latin_square::tests::p(0, 3, 4),
                // middle bits
                latin_square::tests::p(1, 0, 3),
                latin_square::tests::p(2, 0, 2),
                latin_square::tests::p(1, 3, 2),
                latin_square::tests::p(2, 3, 3),
                // bottom row
                latin_square::tests::p(3, 0, 4),
                latin_square::tests::p(3, 1, 3),
                latin_square::tests::p(3, 2, 2),
                latin_square::tests::p(3, 3, 1),
            ],
        );

        sudoku.possibilities.sort();
        assert_eq!(
            sudoku.possibilities,
            vec![
                p(1, 1, 0, 1),
                p(1, 1, 0, 2),
                p(1, 1, 0, 3),
                p(1, 1, 0, 4),
                p(1, 2, 1, 1),
                p(1, 2, 1, 2),
                p(1, 2, 1, 3),
                p(1, 2, 1, 4),
                p(2, 1, 2, 1),
                p(2, 1, 2, 2),
                p(2, 1, 2, 3),
                p(2, 1, 2, 4),
                p(2, 2, 3, 1),
                p(2, 2, 3, 2),
                p(2, 2, 3, 3),
                p(2, 2, 3, 4),
            ]
        );
        sudoku.constraints.sort();
        assert_eq!(
            sudoku.constraints,
            vec![
                c_row(1, 1),
                c_row(1, 4),
                c_row(2, 1),
                c_row(2, 4),
                c_col(1, 1),
                c_col(1, 4),
                c_col(2, 1),
                c_col(2, 4),
                c_row_col(1, 1),
                c_row_col(1, 2),
                c_row_col(2, 1),
                c_row_col(2, 2),
                c_square(0, 4),
                c_square(1, 1),
                c_square(2, 1),
                c_square(3, 4),
            ]
        );
    }

    #[test]
    fn solve_small_sudoku() {
        let sudoku = Sudoku::new(
            2,
            vec![
                // top row
                latin_square::tests::p(0, 0, 1),
                latin_square::tests::p(0, 1, 2),
                latin_square::tests::p(0, 2, 3),
                latin_square::tests::p(0, 3, 4),
                // middle bits
                latin_square::tests::p(1, 0, 3),
                latin_square::tests::p(2, 0, 2),
                latin_square::tests::p(1, 3, 2),
                latin_square::tests::p(2, 3, 3),
                // bottom row
                latin_square::tests::p(3, 0, 4),
                latin_square::tests::p(3, 1, 3),
                latin_square::tests::p(3, 2, 2),
                latin_square::tests::p(3, 3, 1),
            ],
        );

        let mut solver = sudoku.solver();
        let solutions = solver.all_solutions();

        assert_eq!(solutions.len(), 1);
        assert_eq!(
            solutions[0],
            vec![
                &p(1, 1, 0, 4),
                &p(1, 2, 1, 1),
                &p(2, 1, 2, 1),
                &p(2, 2, 3, 4)
            ]
        );
    }
}
