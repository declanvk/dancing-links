use dancing_links::{
    latin_square::{self},
    sudoku::{self, Sudoku},
    ExactCover,
};

/// Generate a Sudoku puzzle from an input string.
///
/// # Expected Format
///  - 0 denotes an empty value
///  - The numbers are presented in row-major order. So the first `side_length`
///    numbers are the first row, the second nine numbers are the second row,
///    etc.
///
/// # Panics
///  - If the string is not exactly `side_length` * `side_length` characters
///  - If any character in the string is not [0-9]
#[allow(dead_code)]
pub fn parse_sudoku_possibilities(sudoku_input: &str, box_side_length: usize) -> Sudoku {
    fn index_to_row_column(index: usize, side_length: usize) -> (usize, usize) {
        // 9x9 example:
        // ┌───┬───┬───┬───┬───┬───┬───┬───┬───┐
        // │ 1 │ 2 │ 3 │ 4 │ 5 │ 6 │ 7 │ 8 │ 9 │
        // ├───┼───┼───┼───┼───┼───┼───┼───┼───┤
        // │10 │11 │12 │13 │14 │15 │16 │17 │18 │
        // ├───┼───┼───┼───┼───┼───┼───┼───┼───┤
        // │19 │20 │21 │22 │23 │24 │25 │26 │27 │
        // ├───┼───┼───┼───┼───┼───┼───┼───┼───┤
        // │28 │29 │30 │31 │32 │33 │34 │35 │36 │
        // ├───┼───┼───┼───┼───┼───┼───┼───┼───┤
        // │37 │38 │39 │40 │41 │42 │43 │44 │45 │
        // ├───┼───┼───┼───┼───┼───┼───┼───┼───┤
        // │46 │47 │48 │49 │50 │51 │52 │53 │54 │
        // ├───┼───┼───┼───┼───┼───┼───┼───┼───┤
        // │55 │56 │57 │58 │59 │60 │61 │62 │63 │
        // ├───┼───┼───┼───┼───┼───┼───┼───┼───┤
        // │64 │65 │66 │67 │68 │69 │70 │71 │72 │
        // ├───┼───┼───┼───┼───┼───┼───┼───┼───┤
        // │73 │74 │75 │76 │77 │78 │79 │80 │81 │
        // └───┴───┴───┴───┴───┴───┴───┴───┴───┘

        let row = index / side_length;
        let column = index % side_length;

        (row, column)
    }

    let side_length = box_side_length * box_side_length;

    log::debug!(
        "Parsing sudoku puzzle input [{}] for side length [{}].",
        sudoku_input,
        side_length
    );

    assert_eq!(
        sudoku_input.len(),
        side_length * side_length,
        "Input needs to be `side_length` * `side_length` characters long."
    );

    let filled_values: Vec<_> = sudoku_input
        .char_indices()
        .filter_map(|(index, c)| {
            c.to_digit(10).and_then(|value| {
                if value == 0 {
                    None
                } else {
                    let (row, column) = index_to_row_column(index, side_length);

                    Some(latin_square::Possibility {
                        row,
                        column,
                        value: usize::try_from(value).unwrap(),
                    })
                }
            })
        })
        .collect();

    log::debug!("Generated filled_values [{:?}].", filled_values);

    Sudoku::new(box_side_length, filled_values)
}

/// Format a list of sudoku possibilities into a string format matching the
/// input of `parse_sudoku_possibilities`.
///
/// See `parse_sudoku_possibilities` documentation for details.
///
/// # Panics
///  - Panics if there is more that one `Possibility` with the same (row,
///    column) values.
///  - Panics if any of the `Possibility.value` has more than a single digit.
#[allow(dead_code)]
pub fn format_sudoku_possibilities<'a>(
    possibilities: impl IntoIterator<Item = &'a sudoku::Possibility>,
    box_side_length: usize,
) -> String {
    let side_length = box_side_length * box_side_length;
    let mut output = Vec::<u8>::with_capacity(side_length * side_length);

    output.fill('0' as u8);

    for possibility in possibilities {
        let index = possibility.row * side_length + possibility.column;
        if output[index] == ('0' as u8) {
            let formated_value = possibility.value.to_string();
            assert_eq!(formated_value.len(), 1);
            output[index] = formated_value.as_bytes()[0];
        } else {
            panic!(
                "Overwriting an existing value [{}] with [{}] at position [{},{}]",
                output[index], possibility.value, possibility.row, possibility.column
            );
        }
    }

    String::from_utf8(output).unwrap()
}

pub struct Sudoku6x6 {
    possibilities: Vec<sudoku::Possibility>,
    constraints: Vec<sudoku::Constraint>,
}

impl Sudoku6x6 {
    #[allow(dead_code)]
    pub fn empty() -> Self {
        // Each column must contain 1-6
        // Each row must contain 1-6
        // Each (2 rows x 3 columns) box must contain 1-6
        let possibilities = (0..6)
            .flat_map(|row| {
                (0..6).flat_map(move |column| (1..=6).map(move |value| (row, column, value)))
            })
            .map(|(row, column, value)| {
                let square = 2 * (row / 2) + (column / 3);
                sudoku::Possibility {
                    row,
                    column,
                    square,
                    value,
                }
            })
            .collect();
        let constraints = (0..6)
            .flat_map(|row| {
                (0..6).flat_map(move |column| (1..=6).map(move |value| (row, column, value)))
            })
            .flat_map(|(row, column, value)| {
                [
                    sudoku::Constraint::Latin(latin_square::Constraint::ColumnNumber {
                        column,
                        value,
                    }),
                    sudoku::Constraint::Latin(latin_square::Constraint::RowNumber { row, value }),
                    sudoku::Constraint::Latin(latin_square::Constraint::RowColumn { row, column }),
                ]
            })
            .chain((0..6).flat_map(|square| {
                (1..=6).map(move |value| sudoku::Constraint::SquareNumber { square, value })
            }))
            .collect();

        Sudoku6x6 {
            possibilities,
            constraints,
        }
    }
}

impl ExactCover for Sudoku6x6 {
    type Constraint = sudoku::Constraint;
    type Possibility = sudoku::Possibility;

    fn satisfies(&self, poss: &Self::Possibility, cons: &Self::Constraint) -> bool {
        match cons {
            sudoku::Constraint::Latin(latin_cons) => {
                <sudoku::Possibility as Into<latin_square::Possibility>>::into(*poss)
                    .satisfies(latin_cons)
            }
            sudoku::Constraint::SquareNumber { square, value } => {
                poss.square == *square && poss.value == *value
            }
        }
    }

    fn is_optional(&self, _cons: &Self::Constraint) -> bool {
        false
    }

    fn possibilities(&self) -> &[Self::Possibility] {
        &self.possibilities
    }

    fn constraints(&self) -> &[Self::Constraint] {
        &self.constraints
    }
}
