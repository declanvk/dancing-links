mod common;

use common::parse_sudoku_possibilities;
use dancing_links::{ExactCover, Solver};

#[test]
#[ignore = "test takes upwards of 300 seconds when running not in release mode"]
fn issue_4() {
    env_logger::init();

    // Derived from https://github.com/declanvk/dancing-links/issues/4#issue-1006609105
    let sudoku_input =
        "300080900000340000008005600500104070002009010003000040005001200000000000070008090";

    let (puzzle, _) = parse_sudoku_possibilities(sudoku_input, 3);
    log::debug!("Possibilities:\n{:?}", puzzle.possibilities());
    log::debug!("Constraints:\n{:?}", puzzle.constraints());
    let mut solver = Solver::new(&puzzle);

    let solutions = solver.next_solution();

    assert!(solutions.is_some());
}
