#![deny(missing_docs)]

//! Implementation of [Dancing Links](https://en.wikipedia.org/wiki/Dancing_Links)
//! and [Algorithm X](https://en.wikipedia.org/wiki/Knuth%27s_Algorithm_X) for solving
//!  [exact cover](https://en.wikipedia.org/wiki/Exact_cover) problems.

pub mod grid;
pub mod latin_square;
pub(crate) mod solver;
pub mod sudoku;
pub(crate) mod util;

pub use solver::Solver;

/// An exact cover problem.
pub trait ExactCover {
    /// The type of values that are elements of a solution to the exact cover
    /// problem.
    type Possibility: core::fmt::Debug;

    /// The type of value that are constraints on a given instance of an exact
    /// cover problem.
    type Constraint: core::fmt::Debug;

    /// Return true if the given `Possibility` will satisfy the given
    /// `Constraint`.
    fn satisfies(poss: &Self::Possibility, cons: &Self::Constraint) -> bool;

    /// Return true if the given `Constraint` is optional.
    fn is_optional(cons: &Self::Constraint) -> bool;
}
