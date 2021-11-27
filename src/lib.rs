#![deny(missing_docs)]

//! Implementation of [Dancing Links](https://en.wikipedia.org/wiki/Dancing_Links)
//! and [Algorithm X](https://en.wikipedia.org/wiki/Knuth%27s_Algorithm_X) for solving
//!  [exact cover](https://en.wikipedia.org/wiki/Exact_cover) problems.

pub mod dense_grid;
pub(crate) mod grid;
pub mod latin_square;
pub mod queens;
pub(crate) mod solver;
pub mod sparse_grid;
pub mod sudoku;
pub(crate) mod util;

pub use grid::Grid;
pub use solver::Solver;

/// An instance of an exact cover problem.
pub trait ExactCover {
    /// The type of values that are elements of a solution to the exact cover
    /// problem.
    type Possibility: core::fmt::Debug;

    /// The type of value that are constraints on a given instance of an exact
    /// cover problem.
    type Constraint: core::fmt::Debug;

    /// Return true if the given `Possibility` will satisfy the given
    /// `Constraint`.
    fn satisfies(&self, poss: &Self::Possibility, cons: &Self::Constraint) -> bool;

    /// Return true if the given `Constraint` is optional.
    fn is_optional(&self, cons: &Self::Constraint) -> bool;

    /// Return a list of possibilities for this instance of the problem.
    fn possibilities(&self) -> &[Self::Possibility];

    /// Return a list of constraints that must be satisfied for this instance of
    /// the problem.
    fn constraints(&self) -> &[Self::Constraint];

    /// Return an iterator over all solutions to this instance of the exact
    /// cover problem.
    fn solver(&self) -> Solver<Self>
    where
        Self: Sized,
    {
        Solver::new(self)
    }
}

impl<E> ExactCover for &E
where
    E: ExactCover,
{
    type Constraint = E::Constraint;
    type Possibility = E::Possibility;

    fn satisfies(&self, poss: &Self::Possibility, cons: &Self::Constraint) -> bool {
        <E as ExactCover>::satisfies(self, poss, cons)
    }

    fn is_optional(&self, cons: &Self::Constraint) -> bool {
        <E as ExactCover>::is_optional(self, cons)
    }

    fn possibilities(&self) -> &[Self::Possibility] {
        <E as ExactCover>::possibilities(self)
    }

    fn constraints(&self) -> &[Self::Constraint] {
        <E as ExactCover>::constraints(self)
    }
}
