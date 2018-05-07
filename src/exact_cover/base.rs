use std::{
    fmt::{self, Display}, ptr::NonNull,
};

static mut NODE_COUNT: usize = 0;

#[derive(Debug, PartialEq, Hash, Clone, Copy, Eq)]
#[repr(C)]
pub struct BaseNode {
    pub left: NonNull<BaseNode>,
    pub right: NonNull<BaseNode>,
    pub up: NonNull<BaseNode>,
    pub down: NonNull<BaseNode>,
    pub id: usize,
}

macro_rules! apply_direction {
    ($name:ident, $field:ident) => (
        pub fn $name<F>(&mut self, mut func: F) where F: FnMut(NonNull<BaseNode>, NonNull<BaseNode>) {
            let self_ptr = self.self_ptr();
            let mut current_ptr = self.$field;
            while current_ptr != self_ptr {
                func(self_ptr, current_ptr);

                unsafe { current_ptr = current_ptr.as_ref().$field };
            }
        }
    )
}

macro_rules! add_direction {
    ($name:ident, $direction:ident, $opposite:ident) => (
        pub fn $name(&mut self, node: &mut BaseNode) {
            self.$direction = node.self_ptr();
            node.$opposite = self.self_ptr();
        }
    )
}

impl BaseNode {
    apply_direction!(apply_left, left);

    apply_direction!(apply_right, right);

    apply_direction!(apply_up, up);

    apply_direction!(apply_down, down);

    add_direction!(add_left, left, right);

    add_direction!(add_right, right, left);

    add_direction!(add_above, up, down);

    add_direction!(add_below, down, up);

    pub fn dangling() -> Self {
        let node = BaseNode {
            left: NonNull::dangling(),
            right: NonNull::dangling(),
            up: NonNull::dangling(),
            down: NonNull::dangling(),
            id: unsafe { NODE_COUNT },
        };

        unsafe {
            NODE_COUNT += 1;
        }

        node
    }

    pub fn set_self_ref(&mut self) {
        let self_ptr = self.self_ptr();

        self.left = self_ptr;
        self.right = self_ptr;
        self.up = self_ptr;
        self.down = self_ptr;
    }

    pub fn self_ptr(&mut self) -> NonNull<BaseNode> {
        unsafe { NonNull::new_unchecked(self) }
    }

    pub fn cover_lr(&mut self) {
        unsafe {
            self.left.as_mut().right = self.right;
            self.right.as_mut().left = self.left;
        }
    }

    pub fn cover_ud(&mut self) {
        unsafe {
            self.up.as_mut().down = self.down;
            self.down.as_mut().up = self.up;
        }
    }

    pub fn uncover_lr(&mut self) {
        let self_ptr = self.self_ptr();

        unsafe {
            self.left.as_mut().right = self_ptr;
            self.right.as_mut().left = self_ptr;
        }
    }

    pub fn uncover_ud(&mut self) {
        let self_ptr = self.self_ptr();

        unsafe {
            self.up.as_mut().down = self_ptr;
            self.down.as_mut().up = self_ptr;
        }
    }
}

impl Display for BaseNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            write!(
                f,
                "Base({}, left: {}, right: {}, up: {}, down: {})",
                self.id,
                self.left.as_ref().id,
                self.right.as_ref().id,
                self.up.as_ref().id,
                self.down.as_ref().id
            )
        }
    }
}
