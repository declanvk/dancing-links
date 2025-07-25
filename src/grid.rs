#![allow(clippy::not_unsafe_ptr_arg_deref)]

//! Dancing links `Grid` implementation for use in the `Solver`.

mod base_node;

use base_node::BaseNode;
use core::{iter::once, ptr};
use std::collections::VecDeque;

/// Dancing links grid, support efficient removal of rows and columns.
#[derive(Debug)]
pub struct Grid {
    // This node only left-right neighbors, no children
    root: *mut Column,

    arena: bumpalo::Bump,
    columns: Vec<*mut Column>,

    num_columns: usize,
    max_row: usize,
}

impl Grid {
    /// Create a new grid with a specified number of columns, and the given
    /// coordinates filled.
    ///
    /// Rows and columns are based 1 indexed for this grid, matching the
    /// indexing notation for matrices in general.
    pub fn new(num_columns: usize, coordinates: impl IntoIterator<Item = (usize, usize)>) -> Self {
        let arena = bumpalo::Bump::new();
        let root = Column::new(&arena, 0);
        let columns = once(root)
            .chain((1..=num_columns).map(|idx| Column::new(&arena, idx)))
            .collect::<Vec<_>>();

        // Chain all the columns together, including the sentinel root column.
        for idx in 0..columns.len() {
            let next_idx = (idx + 1) % columns.len();
            let column = columns[idx];
            let next_column = columns[next_idx];

            Column::add_right(column, next_column);
        }

        let mut grid = Grid {
            root,
            columns,
            arena,
            num_columns,
            max_row: 0,
        };

        grid.add_all_coordinates(coordinates);

        grid
    }

    fn add_all_coordinates(&mut self, coordinates: impl IntoIterator<Item = (usize, usize)>) {
        // Deduct one for the sentinel column
        let mut columns_data: Vec<Vec<_>> =
            (0..(self.columns.len() - 1)).map(|_| Vec::new()).collect();

        for (row, column) in coordinates {
            debug_assert!(
                row != 0 && column != 0,
                "row or column should not equal zero [{:?}].",
                (row, column)
            );
            debug_assert!(
                column <= columns_data.len(),
                "column idx should be in bounds [{column:?}]",
            );

            columns_data[column - 1].push((row, column));

            if self.max_row < row {
                self.max_row = row
            }
        }

        for column_data in &mut columns_data {
            column_data.sort_unstable_by_key(|(k, _)| *k);
        }

        // Map all the data into nodes
        let mut nodes: Vec<VecDeque<*mut Node>> = columns_data
            .into_iter()
            .map(|column_data| {
                column_data
                    .into_iter()
                    .map(|(row_idx, column_idx)| {
                        let column = self.columns[column_idx];

                        Node::new(&self.arena, row_idx, column)
                    })
                    .collect()
            })
            .collect();

        // Then, add all the vertical connections, without wrapping around. Skip the
        // first (sentinel) column.
        for (node_column, column_header) in nodes.iter_mut().zip(self.columns.iter().skip(1)) {
            let pair_it = node_column.iter().zip(node_column.iter().skip(1));
            for (current_node, next_node) in pair_it {
                BaseNode::add_below(current_node.cast(), next_node.cast());
            }

            // Connect first and last to header
            if let Some(first) = node_column.front() {
                BaseNode::add_below(column_header.cast(), first.cast());

                if let Some(last) = node_column.back() {
                    BaseNode::add_above(column_header.cast(), last.cast());
                }
            }
        }

        // Then, add all horizontal connections, with wrap around
        //
        // To do this we need to select all nodes which have the same row value
        // and then chain them together. The column data is in sorted order from
        // before.
        //
        // For each column, collect a list with the top (least row value) node. Then,
        // for each value in the list, collect a subset that contains all the nodes with
        // the same least row value. They should also be in column order. This
        // collection will be linked together with wraparound. Then all those nodes that
        // were selected for the least subset will be replaced from the list with the
        // next relevant node from the column.

        let mut top_nodes: Vec<Option<(usize, *mut Node)>> = nodes
            .iter_mut()
            .map(|column_data| {
                let node = column_data.pop_front();

                node.map(|node| unsafe { (ptr::read(node).row, node) })
            })
            .collect();

        let mut least_nodes = Vec::<(usize, *mut Node)>::with_capacity(top_nodes.len());

        while top_nodes.iter().any(Option::is_some) {
            let mut least_row = usize::MAX;

            // Select the subcollection of least row nodes
            for (idx, row_node_pair) in top_nodes.iter().enumerate() {
                if let Some((row, node)) = row_node_pair {
                    use core::cmp::Ordering;

                    match row.cmp(&least_row) {
                        Ordering::Equal => {
                            least_nodes.push((idx, *node));
                        }
                        Ordering::Less => {
                            least_nodes.clear();
                            least_row = *row;
                            least_nodes.push((idx, *node));
                        }
                        Ordering::Greater => {}
                    }
                }
            }

            // Link all the least row nodes together
            //
            // This is fine for the case of (least_nodes.len() == 1) bc all nodes started
            // already linked to themselves.
            for (idx, (_, node)) in least_nodes.iter().enumerate() {
                let next_node_idx = (idx + 1) % least_nodes.len();
                let (_, next_node) = least_nodes[next_node_idx];

                BaseNode::add_right(node.cast(), next_node.cast());
            }

            // Replace the least row nodes with the next values from their respective
            // columns.
            for (column_idx, _) in least_nodes.drain(..) {
                top_nodes[column_idx] = nodes[column_idx]
                    .pop_front()
                    .map(|node| unsafe { (ptr::read(node).row, node) });
            }
        }
    }

    /// Convert the grid to a dense representation.
    ///
    /// This takes the original size of the grid, and only put `true` values for
    /// locations that are still present in the grid (not covered).
    pub fn to_dense(&self) -> Box<[Box<[bool]>]> {
        let seen_coords = self.uncovered_columns().flat_map(|column_ptr| {
            let column_idx = Column::index(column_ptr);
            Column::row_indices(column_ptr).map(move |row_idx| (row_idx, column_idx))
        });

        let mut output = vec![false; self.num_columns * self.max_row];

        for (row_idx, column_idx) in seen_coords {
            output[(row_idx - 1) * self.num_columns + (column_idx - 1)] = true
        }

        if self.num_columns == 0 {
            debug_assert!(output.is_empty());

            vec![].into_boxed_slice()
        } else {
            output
                .as_slice()
                .chunks(self.num_columns)
                .map(Box::<[_]>::from)
                .collect()
        }
    }

    /// Return an iterator of pointers to columns that are uncovered.
    pub fn uncovered_columns(&self) -> impl Iterator<Item = *const Column> {
        base_node::iter::right(self.root.cast(), Some(self.root.cast()))
            .map(|base_ptr| base_ptr.cast::<Column>())
    }

    /// Return an iterator of mut pointers to columns that are uncovered.
    pub fn uncovered_columns_mut(&mut self) -> impl Iterator<Item = *mut Column> {
        base_node::iter::right_mut(self.root.cast(), Some(self.root.cast()))
            .map(|base_ptr| base_ptr.cast::<Column>())
    }

    /// Return an iterator over all columns that are in the grid (covered and
    /// uncovered).
    pub fn all_columns_mut(&mut self) -> impl DoubleEndedIterator<Item = *mut Column> + '_ {
        self.columns
            .iter()
            .copied()
            // Skip the sentinel
            .skip(1)
    }

    /// Return a pointer to a specific `Column`, if it exists.
    pub fn get_column(&self, index: usize) -> Option<*const Column> {
        self.columns
            .get(index)
            .copied()
            .map(|column_ptr| column_ptr as *const _)
    }

    /// Return a mut pointer to a specific `Column`, if it exists.
    pub fn get_column_mut(&mut self, index: usize) -> Option<*mut Column> {
        self.columns.get(index).copied()
    }

    /// Return true if there are no uncovered columns in the grid.
    pub fn is_empty(&self) -> bool {
        unsafe {
            let column = ptr::read(self.root);

            std::ptr::eq(column.base.right, self.root.cast())
        }
    }
}

/// A coordinate inside of a `Grid`.
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Node {
    base: BaseNode,

    row: usize,
    column: *mut Column,
}

impl Node {
    fn new(arena: &bumpalo::Bump, row: usize, column: *mut Column) -> *mut Self {
        Column::increment_size(column);

        let node = arena.alloc(Node {
            base: BaseNode::new(),

            row,
            column,
        });

        node.base.set_self_ptr();

        node
    }

    /// Cover every `Node` that is horizontally adjacent to this `Node`.
    ///
    /// This `Node` is not covered.
    pub fn cover_row(self_ptr: *mut Node) {
        // Skip over the originating node in the row so that it can be recovered from
        // the column.
        base_node::iter::right_mut(self_ptr.cast(), Some(self_ptr.cast())).for_each(
            |base_ptr| unsafe {
                let node = ptr::read(base_ptr.cast::<Node>());

                Column::decrement_size(node.column);
                BaseNode::cover_vertical(base_ptr);
            },
        )
    }

    /// Uncover every `Node` that is horizontally adjacent to this `Node`.
    ///
    /// This `Node` is not uncovered.
    pub fn uncover_row(self_ptr: *mut Self) {
        let base_ptr = self_ptr.cast::<BaseNode>();

        base_node::iter::left_mut(base_ptr, Some(base_ptr)).for_each(|base_ptr| unsafe {
            let node = ptr::read(base_ptr.cast::<Node>());

            Column::increment_size(node.column);
            BaseNode::uncover_vertical(base_ptr);
        })
    }

    /// Return the row index of this `Node`.
    pub fn row_index(self_ptr: *const Self) -> usize {
        unsafe { ptr::read(self_ptr).row }
    }

    /// Return the column index of this `Node`.
    pub fn column_index(self_ptr: *const Self) -> usize {
        unsafe {
            let node = ptr::read(self_ptr);
            let column = ptr::read(node.column);

            column.index
        }
    }

    /// Return a mut pointer to the `Column` of this `Node`.
    pub fn column_ptr(self_ptr: *const Self) -> *mut Column {
        unsafe {
            let node = ptr::read(self_ptr);

            node.column
        }
    }

    /// Return an iterator over all `Node`s that are adjacent to this `Node`.
    pub fn neighbors(self_ptr: *const Self) -> impl Iterator<Item = *const Node> {
        base_node::iter::left(self_ptr.cast(), None).map(|base_ptr| base_ptr.cast())
    }
}

/// A column inside of a `Grid`.
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Column {
    base: BaseNode,

    size: usize,
    index: usize,
    is_covered: bool,
}

impl Column {
    fn new(arena: &bumpalo::Bump, index: usize) -> *mut Self {
        let column = arena.alloc(Column {
            base: BaseNode::new(),
            size: 0,
            is_covered: false,
            index,
        });

        column.base.set_self_ptr();

        column
    }

    fn increment_size(self_ptr: *mut Self) {
        unsafe {
            let mut column = ptr::read(self_ptr);

            column.size += 1;

            ptr::write(self_ptr, column);
        }
    }

    fn decrement_size(self_ptr: *mut Self) {
        unsafe {
            let mut column = ptr::read(self_ptr);

            column.size -= 1;

            ptr::write(self_ptr, column);
        }
    }

    /// Cover entire column, and any rows that that appear in this column.
    pub fn cover(self_ptr: *mut Self) {
        let mut column = unsafe { ptr::read(self_ptr) };
        assert!(!column.is_covered);

        let base_ptr = self_ptr.cast::<BaseNode>();

        BaseNode::cover_horizontal(base_ptr);

        base_node::iter::down_mut(base_ptr, Some(base_ptr))
            .for_each(|base_ptr| Node::cover_row(base_ptr.cast()));

        column.is_covered = true;
        unsafe {
            ptr::write(self_ptr, column);
        }
    }

    /// Uncover entire column, and any rows that appear in this column.
    pub fn uncover(self_ptr: *mut Self) {
        let mut column = unsafe { ptr::read(self_ptr) };
        assert!(column.is_covered);

        let base_ptr = self_ptr.cast::<BaseNode>();

        base_node::iter::up_mut(base_ptr, Some(base_ptr))
            .for_each(|base_ptr| Node::uncover_row(base_ptr.cast()));

        BaseNode::uncover_horizontal(base_ptr);

        column.is_covered = false;
        unsafe {
            ptr::write(self_ptr, column);
        }
    }

    fn add_right(self_ptr: *mut Self, neighbor_ptr: *mut Column) {
        BaseNode::add_right(self_ptr.cast(), neighbor_ptr.cast());
    }

    /// Return true if there are no uncovered `Node`s in this column.
    pub fn is_empty(self_ptr: *const Self) -> bool {
        unsafe {
            let column = ptr::read(self_ptr);

            let empty = (column.base.down as *const _) == self_ptr;

            debug_assert!(
                !empty && Self::size(self_ptr) == 0,
                "The size should be tracked accurately."
            );

            empty
        }
    }

    /// Return an iterator over the row indices of all uncovered `Node`s in this
    /// column.
    pub fn row_indices(self_ptr: *const Self) -> impl Iterator<Item = usize> {
        Column::rows(self_ptr).map(|node_ptr| unsafe { ptr::read(node_ptr).row })
    }

    /// Return an iterator of pointers to all uncovered `Node`s in this column.
    pub fn rows(self_ptr: *const Self) -> impl Iterator<Item = *const Node> {
        base_node::iter::down(self_ptr.cast(), Some(self_ptr.cast()))
            .map(|base_ptr| base_ptr.cast())
    }

    /// Return an iterator of mut pointers to all uncovered `Node`s in this
    /// column.
    pub fn nodes_mut(self_ptr: *mut Self) -> impl Iterator<Item = *mut Node> {
        base_node::iter::down_mut(self_ptr.cast(), Some(self_ptr.cast()))
            .map(|base_ptr| base_ptr.cast())
    }

    /// Return the column index.
    #[inline]
    pub fn index(self_ptr: *const Self) -> usize {
        unsafe { ptr::read(self_ptr).index }
    }

    /// Return the number of uncovered nodes in this column.
    #[inline]
    pub fn size(self_ptr: *const Self) -> usize {
        unsafe { ptr::read(self_ptr).size }
    }
}

/// This function will convert a grid to a string representation useful for
/// debugging
///
/// This should only be used for test functions.
#[cfg(test)]
pub fn to_string(grid: &Grid) -> String {
    use std::fmt::Write;

    let mut output = String::new();
    let dense = grid.to_dense();

    if dense.is_empty() {
        writeln!(&mut output, "Empty!").unwrap();

        return output;
    }

    for row in dense.iter() {
        writeln!(
            &mut output,
            "{:?}",
            row.iter()
                .map(|yes| if *yes { 1 } else { 0 })
                .collect::<Vec<_>>()
        )
        .unwrap();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn create_a_small_grid() {
        let grid = Grid::new(4, vec![(1, 1), (1, 4), (2, 2), (3, 3), (4, 1), (4, 4)]);

        assert_eq!(
            grid.to_dense(),
            [
                true, false, false, true,
                false, true, false, false,
                false, false, true, false,
                true, false, false, true
            ]
            .chunks(4)
            .map(Box::<[_]>::from)
            .collect()
        );
    }

    #[test]
    #[rustfmt::skip]
    fn create_weird_grids() {
        let thin_grid = Grid::new(1, vec![
            (1, 1),
            (2, 1),
            (3, 1),
            // skip 4
            (5, 1),
            // skip 6, 7
            (8, 1)
        ]);

        // The reasoning behind having the skipped rows not show up in
        // the dense output is that those rows are not present at all in
        // the
        assert_eq!(
            thin_grid.to_dense(),
            [
                true,
                true,
                true,
                false,
                true,
                false,
                false,
                true
            ]
            .chunks(1)
            .map(Box::<[_]>::from)
            .collect()
        );
        assert!(!thin_grid.is_empty());

        let very_thin_grid = Grid::new(0, vec![]);

        assert_eq!(very_thin_grid.to_dense(), vec![].into_boxed_slice());
        assert!(very_thin_grid.is_empty());
    }

    #[test]
    #[rustfmt::skip]
    fn cover_uncover_column() {
        let mut grid = Grid::new(4, vec![(1, 1), (1, 4), (2, 2), (3, 3), (4, 1), (4, 4)]);

        // mutate the grid
        Column::cover(grid.all_columns_mut().nth(3).unwrap());

        // Check remaining columns
        assert!(grid
            .uncovered_columns()
            .map(|column_ptr| unsafe { ptr::read(column_ptr).index })
            .eq(1..=3));
        assert_eq!(
            grid.to_dense(),
            [
                false, false, false, false,
                false, true, false, false,
                false, false, true, false,
                false, false, false, false
            ]
            .chunks(4)
            .map(Box::<[_]>::from)
            .collect()
        );

        // mutate the grid
        Column::uncover(grid.all_columns_mut().nth(3).unwrap());

        // Check remaining columns
        assert!(grid
            .uncovered_columns()
            .map(|column_ptr| unsafe { ptr::read(column_ptr).index })
            .eq(1..=4));
        assert_eq!(
            grid.to_dense(),
            [
                true, false, false, true,
                false, true, false, false,
                false, false, true, false,
                true, false, false, true
            ]
            .chunks(4)
            .map(Box::<[_]>::from)
            .collect()
        );
    }

    #[test]
    #[rustfmt::skip]
    fn cover_uncover_all() {
        let mut grid = Grid::new(4, vec![
            (1, 1),                 (1, 4),
                    (2, 2),
                            (3, 3),
            (4, 1),                 (4, 4)
        ]);

        // mutate the grid
        for column_ptr in grid.all_columns_mut() {
            Column::cover(column_ptr)
        }

        // Check remaining columns
        assert!(grid.uncovered_columns().map(|column_ptr| unsafe { ptr::read(column_ptr).index }).eq(0..0));
        assert_eq!(
            grid.to_dense(),
            [
                false, false, false, false,
                false, false, false, false,
                false, false, false, false,
                false, false, false, false
            ]
            .chunks(4)
            .map(Box::<[_]>::from)
            .collect()
        );
        assert!(grid.is_empty());

        // mutate the grid
        for column_ptr in grid.all_columns_mut().rev() {
            Column::uncover(column_ptr)
        }

        // Check remaining columns
        assert!(grid.uncovered_columns().map(|column_ptr| unsafe { ptr::read(column_ptr).index }).eq(1..=4));
        assert_eq!(
            grid.to_dense(),
            [
                true, false, false, true,
                false, true, false, false,
                false, false, true, false,
                true, false, false, true
            ]
            .chunks(4)
            .map(Box::<[_]>::from)
            .collect()
        );
        assert!(!grid.is_empty());
    }

    #[test]
    #[rustfmt::skip]
    fn latin_square_cover_1() {
        // [1, 0, 0, 0, 1, 0]
        // [0, 1, 1, 0, 1, 0]
        // [1, 0, 0, 1, 0, 1]
        // [0, 1, 0, 0, 0, 1]
        let mut grid = Grid::new(6, vec![
            (1, 1),                         (1, 5),
                    (2, 2), (2, 3),         (2, 5),
            (3, 1),                 (3, 4),         (3, 6),
                    (4, 2),                         (4, 6),
        ]);

        assert_eq!(
            grid.to_dense(),
            [
                true, false, false, false, true, false,
                false, true, true, false, true, false,
                true, false, false, true, false, true,
                false, true, false, false, false, true,
            ]
            .chunks(6)
            .map(Box::<[_]>::from)
            .collect()
        );
        assert!(!grid.is_empty());

        Column::cover(grid.get_column_mut(2).unwrap());
        Column::cover(grid.get_column_mut(3).unwrap());
        Column::cover(grid.get_column_mut(5).unwrap());

        assert_eq!(
            grid.to_dense(),
            [
                false, false, false, false, false, false,
                false, false, false, false, false, false,
                true, false, false, true, false, true,
                false, false, false, false, false, false,
            ]
            .chunks(6)
            .map(Box::<[_]>::from)
            .collect()
        );
    }
}
