use crate::ExactCover;
use core::iter;
use std::collections::HashSet;

pub struct LatinSquare;

impl LatinSquare {
    // Symbols are in the range (0..side_length)
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        side_length: usize,
        filled_values: impl IntoIterator<Item = Possibility>,
    ) -> (Vec<Possibility>, Vec<Constraint>) {
        let filled_values: Vec<_> = filled_values
            .into_iter()
            .inspect(|poss| {
                debug_assert!(
                    0 < poss.value && poss.value <= side_length,
                    "Symbol values should be in range (0..side_length)"
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

        (possibilities, constraints)
    }
}

impl ExactCover for LatinSquare {
    type Constraint = Constraint;
    type Possibility = Possibility;

    fn satisfies(poss: &Self::Possibility, cons: &Self::Constraint) -> bool {
        use Constraint::*;

        match cons {
            RowNumber { row, value } => poss.row == *row && poss.value == *value,
            ColumnNumber { column, value } => poss.column == *column && poss.value == *value,
            RowColumn { row, column } => poss.row == *row && poss.column == *column,
        }
    }

    fn is_optional(_cons: &Self::Constraint) -> bool {
        false
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Possibility {
    pub(crate) row: usize,
    pub(crate) column: usize,
    pub(crate) value: usize,
}

impl Possibility {
    pub fn all(side_length: usize) -> impl Iterator<Item = Self> {
        crate::util::three_combination_iter([side_length, side_length, side_length + 1], [0, 0, 1])
            .map(|[column, row, value]| Possibility { row, column, value })
    }

    pub fn satisfied_constraints(self) -> impl Iterator<Item = Constraint> {
        iter::successors(
            Some(Constraint::RowNumber {
                row: self.row,
                value: self.value,
            }),
            move |cons| match cons {
                Constraint::RowNumber { .. } => Some(Constraint::ColumnNumber {
                    column: self.column,
                    value: self.value,
                }),
                Constraint::ColumnNumber { .. } => Some(Constraint::RowColumn {
                    row: self.row,
                    column: self.column,
                }),
                Constraint::RowColumn { .. } => None,
            },
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Constraint {
    RowNumber { row: usize, value: usize },
    ColumnNumber { column: usize, value: usize },
    RowColumn { row: usize, column: usize },
}

impl Constraint {
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
        let (mut possibilities, mut constraints) =
            LatinSquare::new(2, vec![p(0, 0, 1), p(0, 1, 2)]);

        possibilities.sort();
        assert_eq!(
            possibilities,
            vec![p(1, 0, 1), p(1, 0, 2), p(1, 1, 1), p(1, 1, 2)]
        );
        constraints.sort();
        assert_eq!(
            constraints,
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
        let (possibilities, constraints) = LatinSquare::new(2, vec![p(0, 0, 1), p(0, 1, 2)]);
        let mut solver = crate::solver::Solver::<LatinSquare>::new(&possibilities, &constraints);
        let solutions = solver.all_solutions();

        assert_eq!(solutions.len(), 1);
        assert_eq!(solutions[0], vec![&p(1, 0, 2), &p(1, 1, 1)]);
    }

    #[test]
    fn solve_multi_solution_latin_square() {
        let (possibilities, constraints) = LatinSquare::new(2, vec![]);
        let mut solver = crate::solver::Solver::<LatinSquare>::new(&possibilities, &constraints);
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
        let (possibilities, constraints) = LatinSquare::new(2, vec![p(0, 0, 1), p(0, 1, 1)]);
        let mut solver = crate::solver::Solver::<LatinSquare>::new(&possibilities, &constraints);
        let solutions = solver.all_solutions();

        assert_eq!(solutions.len(), 0);
    }
}
