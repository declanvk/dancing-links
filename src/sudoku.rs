use super::exact_cover::{Constraint, Possibility};
use std::iter::Iterator;

const MAX_ROW: usize = 9;
const MAX_COL: usize = 9;
const MAX_VAL: usize = 9;
const MAX_BLOCK: usize = 9;

// (row, column, value)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct CellPossibility(u8, u8, u8);

impl CellPossibility {
    fn next(&self) -> Option<Self> {
        let CellPossibility(r, c, v) = *self;

        if v == 9 {
            if c == 9 {
                if r == 9 {
                    None
                } else {
                    Some(CellPossibility(r + 1, 1, 1))
                }
            } else {
                Some(CellPossibility(r, c + 1, 1))
            }
        } else {
            Some(CellPossibility(r, c, v + 1))
        }
    }

    pub fn all_possibilities() -> AllPossibilities {
        AllPossibilities {
            current: Some(CellPossibility(1, 1, 1)),
        }
    }

    fn cell_index(&self) -> u8 {
        self.0 + MAX_ROW as u8 * self.1
    }

    fn block_index(&self) -> u8 {
        self.0 / (MAX_BLOCK as u8) + self.1 / (MAX_BLOCK as u8 / 3)
    }
}

impl Possibility for CellPossibility {
    type Constraint = SudokuConstraint;

    fn constraints(&self) -> Vec<Self::Constraint> {
        vec![
            SudokuConstraint::Cell(self.cell_index()),
            SudokuConstraint::Row(self.0, self.2),
            SudokuConstraint::Column(self.1, self.2),
            SudokuConstraint::Block(self.block_index(), self.2),
        ]
    }
}

pub struct AllPossibilities {
    current: Option<CellPossibility>,
}

impl Iterator for AllPossibilities {
    type Item = CellPossibility;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current;

        self.current = result.and_then(|x| x.next());

        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum SudokuConstraint {
    Cell(u8),       // 1 - 81 cell
    Row(u8, u8),    // (1 - 9 row, 1 - 9 value)
    Column(u8, u8), // (1 - 9 column, 1 - 9 value)
    Block(u8, u8),  // (1 - 9 block, 1 - 9 value)
}

impl SudokuConstraint {
    fn next(&self) -> Option<Self> {
        match *self {
            SudokuConstraint::Cell(c) => if c >= 81 {
                Some(SudokuConstraint::Row(1, 1))
            } else {
                Some(SudokuConstraint::Cell(c + 1))
            },
            SudokuConstraint::Row(r, v) => if v >= 9 {
                if r >= 9 {
                    Some(SudokuConstraint::Column(1, 1))
                } else {
                    Some(SudokuConstraint::Row(r + 1, 1))
                }
            } else {
                Some(SudokuConstraint::Row(r, v + 1))
            },
            SudokuConstraint::Column(c, v) => if v >= 9 {
                if c >= 9 {
                    Some(SudokuConstraint::Block(1, 1))
                } else {
                    Some(SudokuConstraint::Column(c + 1, 1))
                }
            } else {
                Some(SudokuConstraint::Column(c, v + 1))
            },
            SudokuConstraint::Block(b, v) => if v >= 9 {
                if b >= 9 {
                    None
                } else {
                    Some(SudokuConstraint::Block(b + 1, 1))
                }
            } else {
                Some(SudokuConstraint::Block(b, v + 1))
            },
        }
    }

    pub fn all_constraints() -> AllConstraints {
        AllConstraints {
            current: Some(SudokuConstraint::Cell(1)),
        }
    }
}

pub struct AllConstraints {
    current: Option<SudokuConstraint>,
}

impl Iterator for AllConstraints {
    type Item = SudokuConstraint;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current;

        self.current = result.and_then(|x| x.next());

        result
    }
}

impl Constraint for SudokuConstraint {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_all_possibilities() {
        use std::collections::HashSet;

        let all_possibilities: HashSet<CellPossibility> =
            CellPossibility::all_possibilities().collect();

        assert_eq!(all_possibilities.len(), MAX_ROW * MAX_VAL * MAX_COL);
    }

    #[test]
    fn generate_all_constraints() {
        use std::collections::HashSet;

        let all_possibilities: HashSet<SudokuConstraint> =
            SudokuConstraint::all_constraints().collect();

        assert_eq!(
            all_possibilities.len(),
            MAX_ROW * MAX_COL + MAX_ROW * MAX_VAL + MAX_COL * MAX_VAL + MAX_BLOCK * MAX_VAL
        );
    }
}
