use super::utils::{get_pair_mut, WindowsMut};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ptr::NonNull;

pub trait Constraint: Eq + Ord + Hash + Sized + Clone {}

pub trait Possibility: Eq + Ord + Hash + Sized + Clone {
    type Constraint: Constraint;

    fn constraints(&self) -> Vec<Self::Constraint>;
}

#[derive(Debug, PartialEq)]
pub struct Solver<P: Hash + Eq, C: Hash + Eq> {
    root: GridRoot<P, C>,
}

impl<P> Solver<P, P::Constraint>
where
    P: Possibility + Debug,
    P::Constraint: Debug,
{
    pub fn new(possibilities: &[P]) -> Self {
        let mut root = GridRoot::new();

        for possibility in possibilities {
            root.add_possibility(possibility.clone());
        }

        Solver { root }
    }
}

#[derive(Debug, PartialEq)]
#[repr(C)]
struct GridRoot<P: Hash + Eq, C: Hash + Eq> {
    base: BaseNode,
    columns: HashMap<C, Column<P, C>>,
    rows: HashMap<P, Vec<Node<P, C>>>,
}

impl<P> GridRoot<P, P::Constraint>
where
    P: Possibility,
{
    fn new() -> Self {
        GridRoot {
            base: BaseNode::new_self_ref(),
            columns: HashMap::new(),
            rows: HashMap::new(),
        }
    }

    fn add_constraint(&mut self, constraint: &P::Constraint) {
        let mut new_column: Column<P, P::Constraint> = Column::new(constraint);

        unsafe {
            self.base.left.as_mut().add_right(&mut new_column.base);
            self.base.add_left(&mut new_column.base);
        }

        self.columns.insert(constraint.clone(), new_column);
    }

    fn add_possibility(&mut self, possibility: P) {
        // Generate and order constraints
        let mut constraints = possibility.constraints();
        constraints.sort();

        // Generate a node for each pair of possibility and constraint
        let mut nodes: Vec<Node<P, P::Constraint>> = Vec::new();
        for constraint in constraints {
            let column = if self.columns.contains_key(&constraint) {
                self.columns.get_mut(&constraint).unwrap()
            } else {
                self.add_constraint(&constraint);

                self.columns.get_mut(&constraint).unwrap()
            };

            let node = Node::new(&possibility, column);

            nodes.push(node);
        }

        // For each pair of nodes, doubly link them
        let mut windows = WindowsMut::new(nodes.as_mut(), 2);
        while let Some(pair) = windows.next() {
            let (left, right) = pair.split_at_mut(1);

            left[0].base.add_right(&mut right[0].base);
        }

        // Make the entire list circularly linked
        let len = nodes.len() - 1;
        let (first, last) = get_pair_mut(&mut nodes, 0, len);
        first.base.add_left(&mut last.base);

        // Add each node in list to corresponding column
        for mut node in &mut nodes {
            unsafe {
                let mut column_ptr = node.column.cast::<Column<P, P::Constraint>>();
                let column = column_ptr.as_mut();
                column.add_node(&mut node);
            }
        }

        self.rows.insert(possibility, nodes);
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(C)]
struct Column<P, C> {
    base: BaseNode,

    size: usize,
    constraint: C,
    _possibility: PhantomData<P>,
}

impl<P, C> Column<P, C>
where
    P: Possibility,
    C: Constraint,
{
    fn new(constraint: &C) -> Self {
        Column {
            base: BaseNode::new_self_ref(),

            size: 0,
            constraint: constraint.clone(),
            _possibility: PhantomData,
        }
    }

    // Cover entire column, and any rows that that appear in this column
    unsafe fn cover(&mut self) {
        self.base.cover_lr();

        let self_ptr = self.base.self_ptr();
        let mut current_ptr = self.base.down;
        while current_ptr != self_ptr {
            let row_ptr = current_ptr.as_ref().right;
            while row_ptr != current_ptr {
                row_ptr.cast::<Node<P, C>>().as_mut().cover();
            }

            current_ptr = current_ptr.as_ref().down;
        }
    }

    // Uncover entire column, and any rows that appear in this column
    unsafe fn uncover(&mut self) {
        let self_ptr = self.base.self_ptr();
        let mut current_ptr = self.base.up;
        while current_ptr != self_ptr {
            let row_ptr = current_ptr.as_ref().left;
            while row_ptr != current_ptr {
                row_ptr.cast::<Node<P, C>>().as_mut().uncover();
            }

            current_ptr = current_ptr.as_ref().up;
        }

        self.base.uncover_lr();
    }

    // Insert node on bottom of linked column
    fn add_node(&mut self, node: &mut Node<P, C>) {
        debug_assert_eq!(self.base.self_ptr(), node.column);

        unsafe {
            self.base.up.as_mut().add_below(&mut node.base);
            self.base.add_above(&mut node.base);
        }

        self.size += 1;
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[repr(C)]
struct Node<P, C> {
    base: BaseNode,

    column: NonNull<BaseNode>,
    possibility: P,

    _constraint: PhantomData<C>,
}

impl<P, C> Node<P, C>
where
    P: Possibility,
    C: Constraint,
{
    fn new(possibility: &P, column: &mut Column<P, C>) -> Self {
        Node {
            base: BaseNode::new_self_ref(),

            column: column.base.self_ptr(),
            possibility: possibility.clone(),
            _constraint: PhantomData,
        }
    }

    // Cover self and update column count
    fn cover(&mut self) {
        unsafe {
            self.base.cover_ud();
            self.column.cast::<Column<P, C>>().as_mut().size -= 1;
        }
    }

    // Uncover self and update column count
    fn uncover(&mut self) {
        unsafe {
            self.base.uncover_ud();
            self.column.cast::<Column<P, C>>().as_mut().size += 1;
        }
    }
}

#[derive(Debug, PartialEq, Hash, Clone, Eq)]
#[repr(C)]
struct BaseNode {
    left: NonNull<BaseNode>,
    right: NonNull<BaseNode>,
    up: NonNull<BaseNode>,
    down: NonNull<BaseNode>,
}

impl BaseNode {
    fn new_dangling() -> Self {
        BaseNode {
            left: NonNull::dangling(),
            right: NonNull::dangling(),
            up: NonNull::dangling(),
            down: NonNull::dangling(),
        }
    }

    fn new_self_ref() -> Self {
        let mut new_base = Self::new_dangling();
        let self_ptr = new_base.self_ptr();

        new_base.left = self_ptr;
        new_base.right = self_ptr;
        new_base.up = self_ptr;
        new_base.down = self_ptr;

        new_base
    }

    fn self_ptr(&mut self) -> NonNull<BaseNode> {
        unsafe { NonNull::new_unchecked(self) }
    }

    fn add_left(&mut self, node: &mut BaseNode) {
        self.left = node.self_ptr();
        node.right = self.self_ptr();
    }

    fn add_right(&mut self, node: &mut BaseNode) {
        self.right = node.self_ptr();
        node.left = self.self_ptr();
    }

    fn add_above(&mut self, node: &mut BaseNode) {
        self.up = node.self_ptr();
        node.down = self.self_ptr();
    }

    fn add_below(&mut self, node: &mut BaseNode) {
        self.down = node.self_ptr();
        node.up = self.self_ptr();
    }

    fn cover_lr(&mut self) {
        unsafe {
            self.left.as_mut().right = self.right;
            self.right.as_mut().left = self.left;
        }
    }

    fn cover_ud(&mut self) {
        unsafe {
            self.up.as_mut().down = self.down;
            self.down.as_mut().up = self.up;
        }
    }

    fn uncover_lr(&mut self) {
        let self_ptr = self.self_ptr();

        unsafe {
            self.left.as_mut().right = self_ptr;
            self.right.as_mut().left = self_ptr;
        }
    }

    fn uncover_ud(&mut self) {
        let self_ptr = self.self_ptr();

        unsafe {
            self.up.as_mut().down = self_ptr;
            self.down.as_mut().up = self_ptr;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Copy, Clone)]
    struct LatinSquarePoss(u8, u8, u8); // (r , c , v )

    #[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Copy, Clone)]
    enum LatinSquareConst {
        Cell(u8, u8),   // (r, c)
        Row(u8, u8),    // (r, v)
        Column(u8, u8), // (c, v)
    }

    impl Possibility for LatinSquarePoss {
        type Constraint = LatinSquareConst;

        fn constraints(&self) -> Vec<Self::Constraint> {
            let LatinSquarePoss(r, c, v) = *self;

            vec![
                LatinSquareConst::Cell(r, c),
                LatinSquareConst::Row(r, v),
                LatinSquareConst::Column(c, v),
            ]
        }
    }

    impl Constraint for LatinSquareConst {}

    #[test]
    fn generate_latin_constraint_grid() {
        let possibilities = vec![
            LatinSquarePoss(1, 1, 1),
            LatinSquarePoss(1, 1, 2),
            LatinSquarePoss(1, 2, 1),
            LatinSquarePoss(1, 2, 2),
            LatinSquarePoss(2, 1, 1),
            LatinSquarePoss(2, 1, 2),
            LatinSquarePoss(2, 2, 1),
            LatinSquarePoss(2, 2, 2),
        ];

        let solver = Solver::new(&possibilities);
        eprintln!("{:#?}", solver);
        panic!("Testing");
    }
}
