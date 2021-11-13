#![no_main]

use dancing_links::{latin_square::LatinSquare, Solver};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|puzzle: LatinSquare| {
    let mut solver = Solver::new(&puzzle);

    let _solutions = solver.all_solutions();
});
