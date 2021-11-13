mod common;

use common::Sudoku6x6;
use dancing_links::{sudoku::Sudoku, Solver};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufRead, BufReader},
    iter,
    path::PathBuf,
};

use crate::common::{format_sudoku_possibilities, parse_sudoku_possibilities};

// Basing these exact counts off of https://en.wikipedia.org/wiki/Mathematics_of_Sudoku#Sudoku_with_rectangular_regions
#[test]
fn enumerate_all_sudoku_solutions_small() {
    let puzzle_4x4 = Sudoku::new(2, iter::empty());
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
fn single_sudoku_test() {
    env_logger::init();

    let sudoku_input =
        "006008047000607200304009060003100005010020480740005009020930600081000034905006170";
    let expected_solved_sudoku =
        "296318547158647293374259861863194725519723486742865319427931658681572934935486172";

    let puzzle = parse_sudoku_possibilities(sudoku_input, 3);
    let mut solver = Solver::new(&puzzle);

    let solutions = solver.all_solutions();
    assert_eq!(solutions.len(), 1);
    let solution = &solutions[0];
    let actual_solved_sudoku = format_sudoku_possibilities(
        puzzle
            .filled_values
            .iter()
            .chain(solution.iter().map(|p| *p)),
        3,
    );

    assert_eq!(actual_solved_sudoku, expected_solved_sudoku);
}

#[test]
#[ignore]
fn solve_all_test_data() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    fn test_logic(
        sudoku_input: &str,
        expected_solved_sudoku: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        log::debug!(
            "Attempting to solve [{}] into [{}].",
            sudoku_input,
            expected_solved_sudoku
        );

        let puzzle = parse_sudoku_possibilities(sudoku_input, 3);
        let mut solver = Solver::new(&puzzle);

        let solutions = solver.all_solutions();
        assert_eq!(solutions.len(), 1);
        let solution = &solutions[0];
        let actual_solved_sudoku = format_sudoku_possibilities(
            puzzle
                .filled_values
                .iter()
                .chain(solution.iter().map(|p| *p)),
            3,
        );

        assert_eq!(actual_solved_sudoku, expected_solved_sudoku);

        Ok(())
    }

    env_logger::init();

    let mut sudoku_data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    sudoku_data_path.push("tests");
    sudoku_data_path.push("data");
    sudoku_data_path.push("sudoku");

    log::info!(
        "Looking in [{}] for sudoku data.",
        sudoku_data_path.display()
    );

    let sudoku_data_paths = sudoku_data_path
        .read_dir()?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;

    sudoku_data_paths.into_par_iter().take(1).try_for_each(
        |sudoku_chunk_path| -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
            let sudoku_chunk_file = OpenOptions::new().read(true).open(&sudoku_chunk_path)?;
            let mut sudoku_file_buffer = BufReader::new(sudoku_chunk_file);
            let mut line = String::new();
            let mut line_number = 0;

            // skip first line with csv header
            sudoku_file_buffer.read_line(&mut line)?;
            line_number += 1;

            loop {
                line.clear();

                sudoku_file_buffer.read_line(&mut line)?;
                line_number += 1;
                if line.is_empty() {
                    // No more lines to read
                    break;
                }

                let mut fields = line.trim().split(",");
                let sudoku_input = fields.next().expect(
                    format!(
                        "Unable to extract field 1 from [{}:{}].",
                        sudoku_chunk_path.display(),
                        line_number
                    )
                    .as_str(),
                );
                let solved_sudoku = fields.next().expect(
                    format!(
                        "Unable to extract field 1 from [{}:{}].",
                        sudoku_chunk_path.display(),
                        line_number
                    )
                    .as_str(),
                );

                test_logic(sudoku_input, solved_sudoku)?
            }

            Ok(())
        },
    )?;

    Ok(())
}
