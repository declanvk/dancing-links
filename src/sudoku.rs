use super::{latin_square, ExactCover};
use core::iter;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Sudoku;

impl Sudoku {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        box_side_length: usize,
        filled_values: impl IntoIterator<Item = latin_square::Possibility>,
    ) -> (Vec<Possibility>, Vec<Constraint>) {
        let side_length = box_side_length * box_side_length;
        let filled_values: Vec<_> = filled_values.into_iter().collect();

        let (latin_possibilities, latin_constraints) =
            latin_square::LatinSquare::new(side_length, filled_values.iter().copied());

        let satisfied: HashSet<_> = filled_values
            .iter()
            .copied()
            .map(|latin_poss| Possibility::from_latin(latin_poss, box_side_length))
            .flat_map(Possibility::satisfied_constraints)
            .collect();

        let possibilities = latin_possibilities
            .into_iter()
            .map(|latin_poss| Possibility::from_latin(latin_poss, box_side_length))
            .collect();

        let constraints = latin_constraints
            .into_iter()
            .map(Constraint::from)
            .chain(Constraint::all_square_number(box_side_length))
            .filter(|cons| !satisfied.contains(cons))
            .collect();

        (possibilities, constraints)
    }
}

impl ExactCover for Sudoku {
    type Constraint = Constraint;
    type Possibility = Possibility;

    fn satisfies(poss: &Self::Possibility, cons: &Self::Constraint) -> bool {
        use Constraint::*;

        match cons {
            Latin(latin_cons) => latin_square::LatinSquare::satisfies(&poss.latin, latin_cons),
            SquareNumber { square, value } => poss.square == *square && poss.latin.value == *value,
        }
    }

    fn is_optional(_cons: &Self::Constraint) -> bool {
        false
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Possibility {
    pub(crate) latin: latin_square::Possibility,
    pub(crate) square: usize,
}

impl Possibility {
    pub fn from_latin(latin: latin_square::Possibility, box_side_length: usize) -> Self {
        let side_length = box_side_length * box_side_length;
        let index = latin.row * side_length + latin.column;
        let square = ((index % side_length) / box_side_length)
            + box_side_length * (index / (side_length * box_side_length));

        Possibility { latin, square }
    }

    pub fn satisfied_constraints(self) -> impl Iterator<Item = Constraint> {
        iter::successors(
            Some(Constraint::Latin(latin_square::Constraint::RowNumber {
                row: self.latin.row,
                value: self.latin.value,
            })),
            move |cons| match cons {
                Constraint::Latin(latin_square::Constraint::RowNumber { .. }) => {
                    Some(Constraint::Latin(latin_square::Constraint::ColumnNumber {
                        column: self.latin.column,
                        value: self.latin.value,
                    }))
                }
                Constraint::Latin(latin_square::Constraint::ColumnNumber { .. }) => {
                    Some(Constraint::Latin(latin_square::Constraint::RowColumn {
                        row: self.latin.row,
                        column: self.latin.column,
                    }))
                }
                Constraint::Latin(latin_square::Constraint::RowColumn { .. }) => {
                    Some(Constraint::SquareNumber {
                        square: self.square,
                        value: self.latin.value,
                    })
                }
                Constraint::SquareNumber { .. } => None,
            },
        )
    }
}

impl Into<latin_square::Possibility> for Possibility {
    fn into(self) -> latin_square::Possibility {
        self.latin
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Constraint {
    Latin(latin_square::Constraint),
    SquareNumber { square: usize, value: usize },
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
            latin: latin_square::Possibility { row, column, value },
            square,
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
        let (mut possibilities, mut constraints) = Sudoku::new(
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

        possibilities.sort();
        assert_eq!(
            possibilities,
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
        constraints.sort();
        assert_eq!(
            constraints,
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
        let (possibilities, constraints) = Sudoku::new(
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

        let mut solver = crate::solver::Solver::<Sudoku>::new(&possibilities, &constraints);
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
