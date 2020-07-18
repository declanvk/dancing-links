mod base_node;

use base_node::BaseNode;
use core::{
    iter::{once, repeat},
    ptr,
};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Grid {
    // This node only left-right neighbors, no children
    root: *mut Column,

    arena: bumpalo::Bump,
    columns: Vec<*mut Column>,
}

impl Grid {
    /// Rows and columns are based 1 indexed for this grid, matching the
    /// indexing notation for matrices in general.
    pub fn new(num_columns: usize, coordinates: impl IntoIterator<Item = (usize, usize)>) -> Self {
        let arena = bumpalo::Bump::new();
        let root = Column::new(&arena, 0);
        let columns = once(root)
            .chain(
                (1..=num_columns)
                    .into_iter()
                    .map(|idx| Column::new(&arena, idx)),
            )
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
        };

        grid.add_all_coordinates(coordinates);

        grid
    }

    fn add_all_coordinates(&mut self, coordinates: impl IntoIterator<Item = (usize, usize)>) {
        // Deduct one for the sentinel column
        let mut columns_data: Vec<Vec<_>> = (0..(self.columns.len() - 1))
            .into_iter()
            .map(|_| Vec::new())
            .collect();

        for (row, column) in coordinates {
            debug_assert!(
                row != 0 && column != 0,
                "row or column should not equal zero [{:?}].",
                (row, column)
            );
            debug_assert!(
                column <= columns_data.len(),
                "column idx should be in bounds [{:?}]",
                column
            );

            columns_data[column - 1].push((row, column));
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
                    if *row == least_row {
                        least_nodes.push((idx, *node));
                    } else if *row < least_row {
                        least_nodes.clear();
                        least_row = *row;
                        least_nodes.push((idx, *node));
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

    pub fn to_dense(&self) -> Box<[Box<[bool]>]> {
        let mut column_count = 0;

        // Accessing the columns using this method to get an accurate picture of which
        // values are still uncovered in the grid.

        // Get an accurate count first so that the width of a row is accurate
        for _ in base_node::iter::right(self.root.cast(), Some(self.root.cast())) {
            column_count += 1;

            debug_assert!(column_count <= (self.columns.len() - 1));
        }

        let idx_map = |row: usize, column: usize| (row - 1) * column_count + (column - 1);
        let mut output = vec![];

        // Keep in mind that row_idx and column_idx are 1 based.
        for column_ptr in base_node::iter::right(self.root.cast(), Some(self.root.cast()))
            .map(|node_ptr| node_ptr.cast::<Column>())
        {
            let column = unsafe { ptr::read(column_ptr) };

            for row_idx in Column::values(unsafe { column_ptr.as_ref().unwrap() }) {
                let num_current_rows = output.len() / column_count;
                // If there aren't enough rows in the output, grow the output.
                if num_current_rows < row_idx {
                    let new_rows = row_idx - num_current_rows;

                    output.extend(repeat(false).take(new_rows * column_count));
                }

                debug_assert!(
                    row_idx != 0 && column.idx != 0,
                    "row or column should not equal zero [{:?}].",
                    (row_idx, column.idx)
                );
                output[idx_map(row_idx, column.idx)] = true;
            }
        }

        if column_count == 0 {
            debug_assert!(output.is_empty());

            vec![].into_boxed_slice()
        } else {
            output
                .as_slice()
                .chunks(column_count)
                .map(Box::<[_]>::from)
                .collect()
        }
    }

    pub fn uncovered_columns<'g>(&'g mut self) -> impl Iterator<Item = &'g mut Column> {
        base_node::iter::right_mut(self.root.cast(), Some(self.root.cast())).map(|base_ptr| {
            let column_ptr = base_ptr.cast::<Column>();

            unsafe { column_ptr.as_mut().unwrap() }
        })
    }

    pub fn all_columns<'g>(&'g mut self) -> impl Iterator<Item = &'g mut Column> {
        self.columns
            .iter()
            // Skip the sentinel
            .skip(1)
            .map(|column_ptr| unsafe { column_ptr.as_mut().unwrap() })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[repr(C)]
struct Node {
    pub base: BaseNode,

    pub row: usize,
    pub column: *mut Column,
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

    fn cover_row(self_ptr: *mut Node) {
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

    fn uncover_row(self_ptr: *mut Self) {
        base_node::iter::left_mut(self_ptr.cast(), Some(self_ptr.cast())).for_each(
            |base_ptr| unsafe {
                let node = ptr::read(base_ptr.cast::<Node>());

                Column::increment_size(node.column);
                BaseNode::uncover_vertical(base_ptr);
            },
        )
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(C)]
struct Column {
    base: BaseNode,

    size: usize,
    idx: usize,
    is_covered: bool,
}

impl Column {
    fn new(arena: &bumpalo::Bump, idx: usize) -> *mut Self {
        let column = arena.alloc(Column {
            base: BaseNode::new(),
            size: 0,
            is_covered: false,
            idx,
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

    // Cover entire column, and any rows that that appear in this column
    pub fn cover(&mut self) {
        assert!(!self.is_covered);

        let self_ptr = self.base_ptr();

        BaseNode::cover_horizontal(self_ptr);

        base_node::iter::down_mut(self_ptr, Some(self_ptr))
            .for_each(|base_ptr| Node::cover_row(base_ptr.cast()));

        self.is_covered = true;
    }

    // Uncover entire column, and any rows that appear in this column
    pub fn uncover(&mut self) {
        assert!(self.is_covered);

        let self_ptr = self.base_ptr();

        base_node::iter::up_mut(self_ptr, Some(self_ptr))
            .for_each(|base_ptr| Node::uncover_row(base_ptr.cast()));

        BaseNode::uncover_horizontal(self_ptr);

        self.is_covered = false;
    }

    fn base_ptr(&mut self) -> *mut BaseNode {
        (self as *mut Column).cast()
    }

    fn add_right(self_ptr: *mut Self, neighbor_ptr: *mut Column) {
        BaseNode::add_right(self_ptr.cast(), neighbor_ptr.cast());
    }

    pub fn values(&self) -> impl Iterator<Item = usize> {
        let self_ptr: *const Self = self;

        base_node::iter::down(self_ptr.cast(), Some(self_ptr.cast()))
            .map(|base_ptr| unsafe { ptr::read(base_ptr.cast::<Node>()).row })
    }
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
        let thin_grid = Grid::new(1, vec![(1, 1), (2, 1), (3, 1), (5, 1), (8, 1)]);

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

        let very_thin_grid = Grid::new(0, vec![]);

        assert_eq!(very_thin_grid.to_dense(), vec![].into_boxed_slice());
    }

    #[test]
    #[rustfmt::skip]
    fn cover_uncover_column() {
        let mut grid = Grid::new(4, vec![(1, 1), (1, 4), (2, 2), (3, 3), (4, 1), (4, 4)]);

        // mutate the grid
        grid.all_columns().nth(3).unwrap().cover();

        // Check remaining columns
        assert!(grid.uncovered_columns().map(|column| column.idx).eq(1..=3));
        assert_eq!(
            grid.to_dense(),
            [
                false, false, false,
                false, true, false,
                false, false, true,
            ]
            .chunks(3)
            .map(Box::<[_]>::from)
            .collect()
        );

        // mutate the grid
        grid.all_columns().nth(3).unwrap().uncover();

        // Check remaining columns
        assert!(grid.uncovered_columns().map(|column| column.idx).eq(1..=4));
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
        let mut grid = Grid::new(4, vec![(1, 1), (1, 4), (2, 2), (3, 3), (4, 1), (4, 4)]);

        // mutate the grid
        for column in grid.all_columns() {
            column.cover()
        }

        // Check remaining columns
        assert_eq!(grid.uncovered_columns().map(|column| column.idx).count(), 0);
        assert_eq!(
            grid.to_dense(),
            vec![].into_boxed_slice()
        );

        // mutate the grid
        for column in grid.all_columns() {
            column.uncover()
        }

        // Check remaining columns
        assert!(grid.uncovered_columns().map(|column| column.idx).eq(1..=4));
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
}
