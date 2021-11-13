#![no_main]

use dancing_links::{latin_square, sudoku::Sudoku, Solver};
use libfuzzer_sys::fuzz_target;

#[derive(Debug)]
struct Sudoku4x4Input {
    filled_entries: Vec<latin_square::Possibility>,
}

impl<'a> arbitrary::Arbitrary<'a> for Sudoku4x4Input {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let filled_entries = u
            .arbitrary::<[u8; 16]>()?
            .into_iter()
            .map(usize::from)
            .enumerate()
            .filter_map(|(index, value)| {
                let row = index / 4;
                let column = index % 4;
                let value = value % 5;

                if value == 0 {
                    None
                } else {
                    Some(latin_square::Possibility { row, column, value })
                }
            })
            .collect();

        Ok(Sudoku4x4Input { filled_entries })
    }
}

fuzz_target!(|data: Sudoku4x4Input| {
    let puzzle_4x4 = Sudoku::new(2, data.filled_entries.into_iter());
    let mut solver = Solver::new(&puzzle_4x4);

    let _all_solutions = solver.all_solutions();
});
