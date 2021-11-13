//! A [Latin square](https://en.wikipedia.org/wiki/Latin_square) is a
//!  n × n array filled with n different symbols, each occurring exactly once in
//! each row and exactly once in each column.

use crate::ExactCover;
#[cfg(fuzzing)]
use arbitrary::Arbitrary;
use std::collections::HashSet;

/// Instance of a Latin square puzzle.
#[derive(Debug)]
#[cfg_attr(fuzzing, derive(Arbitrary))]
pub struct LatinSquare {
    /// The list of possible positions + values that could solve the Latin
    /// square puzzle.
    pub possibilities: Vec<Possibility>,
    /// The list of constraints that must be satisfied for this Latin square
    /// puzzle.
    pub constraints: Vec<Constraint>,
    /// The list of values and positions that are given as fixed when the puzzle
    /// is created.
    pub filled_values: Vec<Possibility>,
}

impl LatinSquare {
    /// Create a new Latin square puzzle.
    ///
    /// The puzzle has dimensions `side_length` × `side_length` and the given
    /// list of filled values.
    pub fn new(side_length: usize, filled_values: impl IntoIterator<Item = Possibility>) -> Self {
        let filled_values: Vec<_> = filled_values
            .into_iter()
            .inspect(|poss| {
                debug_assert!(
                    0 < poss.value && poss.value <= side_length,
                    "Symbol values should be in range (1..=side_length)"
                )
            })
            .collect();

        let satisfied: HashSet<_> = filled_values
            .iter()
            .copied()
            .flat_map(Possibility::satisfied_constraints)
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
            filled_values,
        }
    }
}

impl ExactCover for LatinSquare {
    type Constraint = Constraint;
    type Possibility = Possibility;

    fn satisfies(&self, poss: &Self::Possibility, cons: &Self::Constraint) -> bool {
        poss.satisfies(cons)
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

/// A position and value for a box inside of a Latin square puzzle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(fuzzing, derive(Arbitrary))]
pub struct Possibility {
    /// The row position of the box.
    ///
    /// The values ranges from 0 to `side_length - 1`.
    pub row: usize,

    /// The column position of the box.
    ///
    /// The values ranges from 0 to `side_length - 1`.
    pub column: usize,

    /// The value present inside of the box.
    ///
    /// The values ranges from 1 to `side_length`.
    pub value: usize,
}

impl Possibility {
    /// Return an iterator over all possible `Possibility`s for the given
    /// `side_length`.
    pub fn all(side_length: usize) -> impl Iterator<Item = Self> {
        crate::util::three_combination_iter([side_length, side_length, side_length + 1], [0, 0, 1])
            .map(|[column, row, value]| Possibility { row, column, value })
    }

    /// Return an iterator over all `Constraint`s that are satisfied by this
    /// `Possibility`.
    pub fn satisfied_constraints(self) -> impl Iterator<Item = Constraint> {
        [
            Constraint::RowNumber {
                row: self.row,
                value: self.value,
            },
            Constraint::ColumnNumber {
                column: self.column,
                value: self.value,
            },
            Constraint::RowColumn {
                row: self.row,
                column: self.column,
            },
        ]
        .into_iter()
    }

    /// Return true if this `Possibility` satisfies the given `Constraint`.
    pub fn satisfies(&self, constraint: &Constraint) -> bool {
        use Constraint::*;

        match constraint {
            RowNumber { row, value } => self.row == *row && self.value == *value,
            ColumnNumber { column, value } => self.column == *column && self.value == *value,
            RowColumn { row, column } => self.row == *row && self.column == *column,
        }
    }
}

/// A condition which must be satisfied in order to solve a Latin square puzzle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(fuzzing, derive(Arbitrary))]
pub enum Constraint {
    /// A condition that each row should only have a single instance of a
    /// numeric value.
    RowNumber {
        /// The row index
        row: usize,
        /// The unique numeric value
        value: usize,
    },
    /// A condition that each column should only have a single instance of a
    /// numeric value.
    ColumnNumber {
        /// The column index
        column: usize,
        /// The unique numeric value
        value: usize,
    },
    /// A condition that each row, column pair should exist exactly once.
    RowColumn {
        /// The row index
        row: usize,
        /// The column index
        column: usize,
    },
}

impl Constraint {
    /// Return an iterator over all possibly `Constraint`s for the given
    /// `side_length`.
    pub fn all(side_length: usize) -> impl Iterator<Item = Constraint> {
        let row_number_it =
            crate::util::two_combination_iter([side_length, side_length + 1], [0, 1])
                .map(|[row, value]| Constraint::RowNumber { row, value });

        let column_number_it =
            crate::util::two_combination_iter([side_length, side_length + 1], [0, 1])
                .map(|[column, value]| Constraint::ColumnNumber { column, value });

        let row_column_it = crate::util::two_combination_iter([side_length, side_length], [0, 0])
            .map(|[row, column]| Constraint::RowColumn { row, column });

        row_number_it.chain(column_number_it).chain(row_column_it)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn p(row: usize, column: usize, value: usize) -> Possibility {
        Possibility { row, column, value }
    }

    fn c_row(row: usize, value: usize) -> Constraint {
        Constraint::RowNumber { row, value }
    }

    fn c_col(column: usize, value: usize) -> Constraint {
        Constraint::ColumnNumber { column, value }
    }

    fn c_row_col(row: usize, column: usize) -> Constraint {
        Constraint::RowColumn { row, column }
    }

    #[test]
    fn check_all_possibilities() {
        let some_possibilities: Vec<_> = Possibility::all(2).collect();

        assert_eq!(
            &some_possibilities,
            &[
                p(0, 0, 1),
                p(0, 0, 2),
                p(1, 0, 1),
                p(1, 0, 2),
                p(0, 1, 1),
                p(0, 1, 2),
                p(1, 1, 1),
                p(1, 1, 2),
            ]
        );
    }

    #[test]
    fn check_generated_possibilities_constraints() {
        let mut square = LatinSquare::new(2, vec![p(0, 0, 1), p(0, 1, 2)]);

        square.possibilities.sort();
        assert_eq!(
            square.possibilities,
            vec![p(1, 0, 1), p(1, 0, 2), p(1, 1, 1), p(1, 1, 2)]
        );
        square.constraints.sort();
        assert_eq!(
            square.constraints,
            vec![
                c_row(1, 1),
                c_row(1, 2),
                c_col(0, 2),
                c_col(1, 1),
                c_row_col(1, 0),
                c_row_col(1, 1)
            ]
        );
    }

    #[test]
    fn solve_small_latin_square() {
        let square = LatinSquare::new(2, vec![p(0, 0, 1), p(0, 1, 2)]);
        let mut solver = square.solver();
        let solutions = solver.all_solutions();

        assert_eq!(solutions.len(), 1);
        assert_eq!(solutions[0], vec![&p(1, 0, 2), &p(1, 1, 1)]);
    }

    #[test]
    fn solve_multi_solution_latin_square() {
        let square = LatinSquare::new(2, vec![]);
        let mut solver = square.solver();
        let solutions = solver.all_solutions();

        assert_eq!(solutions.len(), 2);

        assert_eq!(
            solutions[0],
            vec![&p(0, 0, 1), &p(0, 1, 2), &p(1, 1, 1), &p(1, 0, 2)]
        );
        assert_eq!(
            solutions[1],
            vec![&p(0, 1, 1), &p(0, 0, 2), &p(1, 0, 1), &p(1, 1, 2)]
        );
    }

    #[test]
    fn solve_impossible_latin_square() {
        let square = LatinSquare::new(2, vec![p(0, 0, 1), p(0, 1, 1)]);
        let mut solver = square.solver();
        let solutions = solver.all_solutions();

        assert_eq!(solutions.len(), 0);
    }
}
