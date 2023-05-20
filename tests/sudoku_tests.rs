mod common;

use common::Sudoku6x6;
use dancing_links::{
    sudoku::{self, Sudoku},
    Solver,
};

use crate::common::{format_sudoku_possibilities, parse_sudoku_possibilities};

// Basing these exact counts off of https://en.wikipedia.org/wiki/Mathematics_of_Sudoku#Sudoku_with_rectangular_regions
#[test]
#[cfg_attr(miri, ignore)]
fn enumerate_all_sudoku_solutions_small() {
    let puzzle_4x4 = Sudoku::new(2, std::iter::empty());
    let solver_4x4 = Solver::new(&puzzle_4x4);
    assert_eq!(solver_4x4.count(), 288);
}

#[test]
#[ignore]
// This takes too long to run. The test below generates 10,000 solutions in 1
// minute, which would give 47 hours to complete this test.
//
// In release mode this should finish in around 1.33 hours.
fn enumerate_all_sudoku_solutions_large() {
    let puzzle_6x6 = Sudoku6x6::empty();
    let solver_6x6 = Solver::new(&puzzle_6x6);
    assert_eq!(solver_6x6.count(), 28_200_960);
}

#[test]
#[ignore]
// This test takes 0.967 minutes to run, which is too long for a normal suite.
//
// In release mode this runs in 1.7 seconds, which is fine.
fn enumerate_many_sudoku_solutions() {
    let puzzle_6x6 = Sudoku6x6::empty();
    let solver_6x6 = Solver::new(&puzzle_6x6);
    // Assert that the number of solutions is at least 10,000.
    assert_eq!(solver_6x6.take(10_000).count(), 10_000);
}

#[test]
#[cfg_attr(miri, ignore)]
fn single_sudoku_test() {
    env_logger::init();

    let sudoku_input =
        "006008047000607200304009060003100005010020480740005009020930600081000034905006170";
    let expected_solved_sudoku =
        "296318547158647293374259861863194725519723486742865319427931658681572934935486172";

    let (puzzle, filled_values) = parse_sudoku_possibilities(sudoku_input, 3);
    let mut solver = Solver::new(&puzzle);

    let solutions = solver.all_solutions();
    assert_eq!(solutions.len(), 1);
    let solution = &solutions[0];
    let actual_solved_sudoku = format_sudoku_possibilities(
        filled_values
            .into_iter()
            .map(|poss| sudoku::Possibility::from_latin(poss, 3))
            .chain(solution.iter().map(|p| **p)),
        3,
    );

    assert_eq!(actual_solved_sudoku, expected_solved_sudoku);
}
