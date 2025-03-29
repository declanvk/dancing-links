//! Run Sudoku solver
//! Usage:
//!
//! ```bash
//! cargo run --release --example sudoku 300080900000340000008005600500104070002009010003000040005001200000000000070008090
//! ```

use dancing_links::{
    latin_square,
    sudoku::{Possibility, Sudoku},
    ExactCover,
};

fn print_solution(problem: &str, solution: &Vec<&Possibility>) {
    let mut s: Vec<char> = problem.chars().collect();
    for poss in solution {
        s[poss.row * 9 + poss.column] = ('0' as usize + poss.value) as u8 as char;
    }
    for i in 0..9 {
        println!("{}", s[i * 9..(i + 1) * 9].iter().collect::<String>());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("problem needed");
        std::process::exit(1);
    }

    let problem = &args[1];
    if problem.len() != 9 * 9 {
        eprintln!("invalid problem format");
        std::process::exit(1);
    }

    let mut filled = Vec::new();
    for row in 0..9 {
        for column in 0..9 {
            let c = problem.chars().nth(row * 9 + column).unwrap();
            if c != '0' {
                let value = c as usize - '0' as usize;
                filled.push(latin_square::Possibility { row, column, value });
            }
        }
    }

    let sudoku = Sudoku::new(3, filled);
    let solver = sudoku.solver();
    for solution in solver {
        print_solution(problem, &solution);
        println!();
    }
}
