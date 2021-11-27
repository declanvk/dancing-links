use std::fmt::Debug;

/// A trait that describes a generic grid, which can be permuted to solve exact
/// cover problems using Algorithm X.
pub trait Grid {
    /// The type representing a column in the grid.
    type Column: Debug + Copy;
    /// The type of an iterator over all the uncovered columns in the grid.
    type UncoveredColumnsIter: Iterator<Item = Self::Column>;
    /// The type representing a row in the grid.
    type Row: Debug + Copy;
    /// The type of an iterator over all uncovered rows in a column.
    type UncoveredRowsIter: Iterator<Item = Self::Row>;
    /// The type of an iterator over all uncovered columns in a row.
    type UncoveredColumnsInRowIter: Iterator<Item = Self::Column>;

    /// Create a new grid with a specified number of columns, and the given
    /// coordinates filled.
    ///
    /// Rows and columns are based 1 indexed for this grid, matching the
    /// indexing notation for matrices in general.
    fn new(
        num_columns: usize,
        filled_coordinates: impl IntoIterator<Item = (usize, usize)>,
    ) -> Self;

    /// Cover entire column, and any rows that that appear in this column.
    ///
    /// # Panics
    ///
    /// Panics if multiple calls to `cover` are made with the same `column`
    /// value, without an intermediate `uncover` for the same column.
    fn cover(&self, column: Self::Column);

    /// Uncover entire column, and any rows that appear in this column.
    ///
    /// # Panics
    ///
    /// Panics if there was not a previous call to `cover` the same column.
    fn uncover(&self, column: Self::Column);

    /// Return an iterator of pointers to columns that are uncovered.
    fn uncovered_columns(&self) -> Self::UncoveredColumnsIter;

    /// Return an iterator of pointers to all uncovered `Node`s in this column.
    fn uncovered_rows_in_column(&self, column: Self::Column) -> Self::UncoveredRowsIter;

    /// Return a stable unique identifier for this column.
    fn column_id(&self, column: Self::Column) -> usize;

    /// Return a stable unique identifier for this row.
    fn row_id(&self, row: Self::Row) -> usize;

    /// Return the number of rows uncovered in this column.
    fn column_size(&self, column: Self::Column) -> usize;

    /// Return the list of columns that are uncovered in the given row.
    fn uncovered_columns_in_row(&self, row: Self::Row) -> Self::UncoveredColumnsInRowIter;
}
