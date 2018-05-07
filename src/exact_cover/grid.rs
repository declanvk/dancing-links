use std::{
    collections::HashMap, fmt::{self, Debug, Display}, hash::Hash, marker::PhantomData,
    ptr::NonNull,
};

use super::{base::BaseNode, Constraint, Possibility};
use crate::utils::get_pair_mut;

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct GridRoot<P: Hash + Eq, C: Hash + Eq> {
    pub base: BaseNode,
    pub columns: HashMap<C, NonNull<Column<P, C>>>,
    pub nodes: Vec<NonNull<Node<P, C>>>,
}

impl<P> GridRoot<P, P::Constraint>
where
    P: Possibility,
{
    pub fn new() -> Box<Self> {
        let mut root = Box::new(GridRoot {
            base: BaseNode::dangling(),
            columns: HashMap::new(),
            nodes: Vec::new(),
        });

        root.base.set_self_ref();

        root
    }

    fn add_constraint(&mut self, constraint: &P::Constraint) {
        // Moving node to heap memory, set node links after
        self.columns
            .insert(constraint.clone(), Column::new(constraint));

        let column = self.columns.get_mut(&constraint).unwrap();
        unsafe { column.as_mut() }.base.set_self_ref();
        unsafe {
            self.base.left.as_mut().add_right(&mut column.as_mut().base);
            self.base.add_left(&mut column.as_mut().base);
        }
    }

    pub fn add_possibility(&mut self, possibility: P) {
        // Generate and order constraints
        let mut constraints = possibility.constraints();
        constraints.sort();

        // Generate a node for each pair of possibility and constraint
        let nodes = {
            let mut nodes: Vec<NonNull<Node<P, P::Constraint>>> = Vec::new();
            for constraint in constraints {
                let column = if self.columns.contains_key(&constraint) {
                    self.columns.get_mut(&constraint).unwrap()
                } else {
                    self.add_constraint(&constraint);

                    self.columns.get_mut(&constraint).unwrap()
                };

                let node = Node::new(&possibility, unsafe { column.as_mut() });

                nodes.push(node);
            }

            let old_len = self.nodes.len();
            self.nodes.extend(nodes);

            &mut self.nodes[old_len..]
        };

        // For each pair of nodes, doubly link them
        let len = nodes.len();
        for idx in 0..len {
            let (left, right) = get_pair_mut(nodes, idx, (idx + 1) % len);

            unsafe {
                left.as_mut().base.add_right(&mut right.as_mut().base);
            }
        }

        // Add each node in list to corresponding column
        for node in nodes {
            unsafe {
                let mut column_ptr = node.as_mut().column.cast::<Column<P, P::Constraint>>();
                let column = column_ptr.as_mut();
                column.add_node(node.as_mut());
            }
        }
    }

    pub fn display(&mut self) -> RootDisplay<P, P::Constraint> {
        let mut display = RootDisplay {
            columns: Vec::new(),
        };

        self.base.apply_right(|_self, current_column| unsafe {
            let mut column_ptr = current_column.cast::<Column<P, P::Constraint>>();
            let column = column_ptr.as_mut();

            display.columns.push(column.display());
        });

        display
    }
}

pub struct RootDisplay<P, C> {
    columns: Vec<ColumnDisplay<P, C>>,
}

impl<P, C> Display for RootDisplay<P, C>
where
    P: Debug,
    C: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut intermediate = Ok(());

        for column in &self.columns {
            intermediate = intermediate.and_then(|_past| write!(f, "{}\n", column))
        }

        intermediate
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Column<P, C> {
    pub base: BaseNode,

    pub size: usize,
    pub constraint: C,
    _possibility: PhantomData<P>,
}

impl<P, C> Column<P, C>
where
    P: Possibility,
    C: Constraint,
{
    pub fn new(constraint: &C) -> NonNull<Self> {
        let boxed_column = Box::new(Column {
            base: BaseNode::dangling(),

            size: 0,
            constraint: constraint.clone(),
            _possibility: PhantomData,
        });

        Box::into_raw_non_null(boxed_column)
    }

    // Cover entire column, and any rows that that appear in this column
    pub unsafe fn cover(&mut self) {
        self.base.cover_lr();

        let column_base = &mut self.base;
        column_base.apply_down(|_column_self, mut current_row| {
            current_row.as_mut().apply_right(|_self, mut current_node| {
                let mut node_ptr = current_node.cast::<Node<P, C>>();
                let column = node_ptr.as_mut().column.as_mut();

                current_node.as_mut().cover_ud();
                column.size -= 1;
            });
        });
    }

    // Uncover entire column, and any rows that appear in this column
    pub unsafe fn uncover(&mut self) {
        self.base.apply_up(|_column_self, mut current_row| {
            current_row.as_mut().apply_left(|_self, mut current_node| {
                let mut node_ptr = current_node.cast::<Node<P, C>>();
                let column = node_ptr.as_mut().column.as_mut();

                current_node.as_mut().uncover_ud();
                column.size += 1;
            });
        });

        self.base.uncover_lr();
    }

    // Insert node on bottom of linked column
    pub fn add_node(&mut self, node: &mut Node<P, C>) {
        unsafe {
            self.base.up.as_mut().add_below(&mut node.base);
            self.base.add_above(&mut node.base);
        }

        self.size += 1;
    }

    pub fn display(&mut self) -> ColumnDisplay<P, C> {
        let mut display = ColumnDisplay {
            id: self.base.id,
            constraint: self.constraint.clone(),
            nodes: Vec::new(),
        };

        self.base.apply_down(|_self, current_node| {
            display.nodes.push(current_node.cast::<Node<P, C>>());
        });

        display
    }
}

#[derive(Debug)]
pub struct ColumnDisplay<P, C> {
    id: usize,
    constraint: C,
    nodes: Vec<NonNull<Node<P, C>>>,
}

impl<P, C> Display for ColumnDisplay<P, C>
where
    P: Debug,
    C: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let transformed: Vec<String> = unsafe {
            self.nodes
                .iter()
                .cloned()
                .map(|node_ptr| format!("{}", node_ptr.as_ref()))
                .collect()
        };

        write!(
            f,
            "{:<2} {:>12}: {:?}",
            self.id,
            format!("{:?}", self.constraint),
            transformed
        )
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[repr(C)]
pub struct Node<P, C> {
    pub base: BaseNode,

    pub column: NonNull<Column<P, C>>,
    pub possibility: P,

    _constraint: PhantomData<C>,
}

impl<P, C> Node<P, C>
where
    P: Possibility,
    C: Constraint,
{
    fn new(possibility: &P, column: &mut Column<P, C>) -> NonNull<Self> {
        let boxed_node = Box::new(Node {
            base: BaseNode::dangling(),

            column: unsafe { NonNull::new_unchecked(column) },
            possibility: possibility.clone(),
            _constraint: PhantomData,
        });

        Box::into_raw_non_null(boxed_node)
    }
}

impl<P, C> Display for Node<P, C>
where
    P: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node({}, {:?})", self.base, self.possibility)
    }
}
