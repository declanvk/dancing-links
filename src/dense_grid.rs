//! Dense `Grid` implementation for use in the `Solver`.
//!
//! The benefit of this grid is that the implementation is easy to audit and
//! avoid pointer chasing. Likely more efficient for small grid sizes.

use std::{
    cell::RefCell,
    collections::HashSet,
    vec::{self},
};

use crate::Grid;

/// Dense grid implementation
#[derive(Debug)]
pub struct DenseGrid(RefCell<DenseGridInner>);

#[derive(Debug)]
struct DenseGridInner {
    num_rows: usize,
    num_columns: usize,

    covered_columns: HashSet<usize>,
    covered_rows: HashSet<usize>,

    data: Vec<bool>,

    covers: Vec<Cover>,
}

#[derive(Debug)]
struct Cover {
    column: usize,
    rows: Vec<usize>,
}

impl DenseGridInner {
    #[inline]
    #[allow(dead_code)]
    fn to_row_column(index: usize, num_columns: usize) -> (usize, usize) {
        let row = index / num_columns;
        let column = index % num_columns;

        (row, column)
    }

    #[inline]
    fn to_index(row: usize, column: usize, num_columns: usize) -> usize {
        row * num_columns + column
    }

    pub fn new(
        num_columns: usize,
        filled_coordinates: impl IntoIterator<Item = (usize, usize)>,
    ) -> Self {
        let filled_coordinates: Vec<_> = filled_coordinates
            .into_iter()
            .map(|(row, column)| (row - 1, column - 1))
            .collect();

        let num_rows = filled_coordinates
            .iter()
            .map(|(row, _)| *row)
            .max()
            .map(|max_row_index| max_row_index + 1)
            .unwrap_or(0);

        let mut data = vec![false; num_rows * num_columns];

        for (row, column) in filled_coordinates {
            data[Self::to_index(row, column, num_columns)] = true;
        }

        DenseGridInner {
            num_rows,
            num_columns,
            covered_columns: HashSet::with_capacity(num_columns / 2),
            covered_rows: HashSet::with_capacity(num_rows / 2),
            covers: Vec::new(),
            data,
        }
    }

    fn cover(&mut self, column: usize) {
        let cover = Cover {
            column,
            rows: self.uncovered_rows_in_column(column).collect(),
        };

        // Assert that this column was not already covered
        assert!(self.covered_columns.insert(column));
        self.covered_rows.extend(cover.rows.iter().copied());

        self.covers.push(cover);
    }

    fn uncover(&mut self, column: usize) {
        let cover = self
            .covers
            .pop()
            .expect("mismatched number of cover & uncover");
        assert_eq!(
            cover.column, column,
            "Expected column argument to match top cover"
        );

        // Check that the column to be uncovered was actually covered in the first place
        assert!(self.covered_columns.remove(&cover.column));
        for row in cover.rows {
            self.covered_rows.remove(&row);
        }
    }

    fn uncovered_columns(&self) -> vec::IntoIter<usize> {
        (0..self.num_columns)
            .filter(|column| !self.covered_columns.contains(column))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn uncovered_rows_in_column(&self, column: usize) -> vec::IntoIter<usize> {
        (0..self.num_rows)
            .filter(|row| !self.covered_rows.contains(row))
            .filter(move |row| {
                let index = Self::to_index(*row, column, self.num_columns);

                self.data[index]
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn column_size(&self, column: usize) -> usize {
        (0..self.num_rows)
            .filter(|row| !self.covered_rows.contains(row))
            .fold(0, |count, row| {
                let index = Self::to_index(row, column, self.num_columns);

                if self.data[index] {
                    count + 1
                } else {
                    count
                }
            })
    }

    fn uncovered_columns_in_row(&self, row: usize) -> vec::IntoIter<usize> {
        (0..self.num_columns)
            .filter(|column| !self.covered_columns.contains(column))
            .filter(|column| {
                let index = Self::to_index(row, *column, self.num_columns);

                self.data[index]
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl Grid for DenseGrid {
    type Column = usize;
    type Row = usize;
    type UncoveredColumnsInRowIter = vec::IntoIter<usize>;
    type UncoveredColumnsIter = vec::IntoIter<usize>;
    type UncoveredRowsIter = vec::IntoIter<usize>;

    fn new(
        num_columns: usize,
        filled_coordinates: impl IntoIterator<Item = (usize, usize)>,
    ) -> Self {
        DenseGrid(RefCell::new(DenseGridInner::new(
            num_columns,
            filled_coordinates,
        )))
    }

    fn cover(&self, column: Self::Column) {
        let mut inner = self.0.borrow_mut();
        DenseGridInner::cover(&mut inner, column)
    }

    fn uncover(&self, column: Self::Column) {
        let mut inner = self.0.borrow_mut();
        DenseGridInner::uncover(&mut inner, column)
    }

    fn uncovered_columns(&self) -> Self::UncoveredColumnsIter {
        let inner = self.0.borrow();
        DenseGridInner::uncovered_columns(&inner)
    }

    fn uncovered_rows_in_column(&self, column: Self::Column) -> Self::UncoveredRowsIter {
        let inner = self.0.borrow();
        DenseGridInner::uncovered_rows_in_column(&inner, column)
    }

    fn column_id(&self, column: Self::Column) -> usize {
        column
    }

    fn row_id(&self, row: Self::Row) -> usize {
        row
    }

    fn column_size(&self, column: Self::Column) -> usize {
        let inner = self.0.borrow();
        DenseGridInner::column_size(&inner, column)
    }

    fn uncovered_columns_in_row(&self, row: Self::Row) -> Self::UncoveredColumnsInRowIter {
        let inner = self.0.borrow();
        DenseGridInner::uncovered_columns_in_row(&inner, row)
    }
}
