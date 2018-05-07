use std::hash::Hash;

#[macro_use]
mod base;
mod grid;
pub mod solver;

pub trait Constraint: Eq + Ord + Hash + Sized + Clone {}

pub trait Possibility: Eq + Ord + Hash + Sized + Clone {
    type Constraint: Constraint;

    fn constraints(&self) -> Vec<Self::Constraint>;
}

#[cfg(test)]
mod tests {
    use super::{solver::Solver, Constraint, Possibility};

    #[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Copy, Clone)]
    struct LatinSquarePoss(u8, u8, u8); // (r , c , v )

    #[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Copy, Clone)]
    enum LatinSquareConst {
        Cell(u8, u8),   // (r, c)
        Row(u8, u8),    // (r, v)
        Column(u8, u8), // (c, v)
    }

    impl Possibility for LatinSquarePoss {
        type Constraint = LatinSquareConst;

        fn constraints(&self) -> Vec<Self::Constraint> {
            let LatinSquarePoss(r, c, v) = *self;

            vec![
                LatinSquareConst::Cell(r, c),
                LatinSquareConst::Row(r, v),
                LatinSquareConst::Column(c, v),
            ]
        }
    }

    impl Constraint for LatinSquareConst {}

    #[test]
    fn generate_latin_constraint_grid() {
        let possibilities = vec![
            LatinSquarePoss(1, 1, 1),
            LatinSquarePoss(1, 1, 2),
            LatinSquarePoss(1, 2, 1),
            LatinSquarePoss(1, 2, 2),
            LatinSquarePoss(2, 1, 1),
            LatinSquarePoss(2, 1, 2),
            LatinSquarePoss(2, 2, 1),
            LatinSquarePoss(2, 2, 2),
        ];

        let mut solver = Solver::new(&possibilities);

        solver.first_solution();
        panic!("Testing");
    }
}
