#![warn(missing_docs)]

pub mod grid;
pub mod latin_square;
pub(crate) mod solver;
pub mod sudoku;
pub(crate) mod util;

pub use solver::Solver;

pub trait ExactCover {
    type Possibility: core::fmt::Debug;
    type Constraint: core::fmt::Debug;

    fn satisfies(poss: &Self::Possibility, cons: &Self::Constraint) -> bool;
    fn is_optional(cons: &Self::Constraint) -> bool;
}
