use core::ptr;

#[derive(Debug, PartialEq, Hash, Clone, Copy, Eq)]
#[repr(C)]
pub struct BaseNode {
    pub left: *mut BaseNode,
    pub right: *mut BaseNode,
    pub up: *mut BaseNode,
    pub down: *mut BaseNode,
}

impl BaseNode {
    pub fn new() -> Self {
        BaseNode {
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            up: ptr::null_mut(),
            down: ptr::null_mut(),
        }
    }

    pub fn set_self_ptr(&mut self) {
        let self_ptr: *mut BaseNode = self;

        self.left = self_ptr;
        self.right = self_ptr;
        self.up = self_ptr;
        self.down = self_ptr;
    }
}

impl Default for BaseNode {
    fn default() -> Self {
        Self::new()
    }
}

// NOTE: The [un]cover methods should always read and write a single location
// atomically (not interleaving read + write) in the case that the 2 neighbor
// locations are in fact the same. If this happens while interleaved, only one
// write will "win" and the operation will be corrupted.
impl BaseNode {
    pub fn cover_horizontal(self_ptr: *mut Self) {
        unsafe {
            let BaseNode {
                left: left_ptr,
                right: right_ptr,
                ..
            } = ptr::read(self_ptr);

            let mut left_node = ptr::read(left_ptr);
            left_node.right = right_ptr;
            ptr::write(left_ptr, left_node);

            let mut right_node = ptr::read(right_ptr);
            right_node.left = left_ptr;
            ptr::write(right_ptr, right_node);
        }
    }

    pub fn cover_vertical(self_ptr: *mut Self) {
        unsafe {
            let BaseNode {
                up: up_ptr,
                down: down_ptr,
                ..
            } = ptr::read(self_ptr);

            let mut up_node = ptr::read(up_ptr);
            up_node.down = down_ptr;
            ptr::write(up_ptr, up_node);

            let mut down_node = ptr::read(down_ptr);
            down_node.up = up_ptr;
            ptr::write(down_ptr, down_node);
        }
    }

    pub fn uncover_horizontal(self_ptr: *mut Self) {
        unsafe {
            let BaseNode {
                left: left_ptr,
                right: right_ptr,
                ..
            } = ptr::read(self_ptr);

            let mut left_node = ptr::read(left_ptr);
            left_node.right = self_ptr;
            ptr::write(left_ptr, left_node);

            let mut right_node = ptr::read(right_ptr);
            right_node.left = self_ptr;
            ptr::write(right_ptr, right_node);
        }
    }

    pub fn uncover_vertical(self_ptr: *mut Self) {
        unsafe {
            let BaseNode {
                up: up_ptr,
                down: down_ptr,
                ..
            } = ptr::read(self_ptr);

            let mut up_node = ptr::read(up_ptr);
            up_node.down = self_ptr;
            ptr::write(up_ptr, up_node);

            let mut down_node = ptr::read(down_ptr);
            down_node.up = self_ptr;
            ptr::write(down_ptr, down_node);
        }
    }
}

macro_rules! add_direction {
    ($name:ident, $direction:ident, $opposite:ident, $lint:meta) => {
        #[$lint]
        pub fn $name(self_ptr: *mut Self, neighbor_ptr: *mut BaseNode) {
            unsafe {
                let mut self_node = ptr::read(self_ptr);
                self_node.$direction = neighbor_ptr;
                ptr::write(self_ptr, self_node);

                let mut neighbor_node = ptr::read(neighbor_ptr);
                neighbor_node.$opposite = self_ptr;
                ptr::write(neighbor_ptr, neighbor_node);
            }
        }
    };

    ($name:ident, $direction:ident, $opposite:ident) => {
        add_direction!($name, $direction, $opposite, allow());
    };
}

impl BaseNode {
    add_direction!(add_below, down, up);

    add_direction!(add_above, up, down);

    add_direction!(add_right, right, left);

    add_direction!(add_left, left, right, allow(dead_code));
}

pub mod iter {
    use std::iter::Map;

    use super::*;

    #[allow(dead_code)]
    pub fn up(
        original: *const BaseNode,
        skip: Option<*const BaseNode>,
    ) -> Map<BaseNodeIterator<fn(&BaseNode) -> *mut BaseNode>, fn(*mut BaseNode) -> *const BaseNode>
    {
        up_mut(original as *mut _, skip.map(|skip| skip as *mut _)).map(|ptr| ptr as *const _)
    }

    pub fn up_mut(
        original: *mut BaseNode,
        skip: Option<*mut BaseNode>,
    ) -> BaseNodeIterator<fn(&BaseNode) -> *mut BaseNode> {
        BaseNodeIterator {
            original,
            current: original,
            skip,
            direction: |node: &BaseNode| node.up,
        }
    }

    pub fn down(
        original: *const BaseNode,
        skip: Option<*const BaseNode>,
    ) -> Map<BaseNodeIterator<fn(&BaseNode) -> *mut BaseNode>, fn(*mut BaseNode) -> *const BaseNode>
    {
        down_mut(original as *mut _, skip.map(|skip| skip as *mut _)).map(|ptr| ptr as *const _)
    }

    pub fn down_mut(
        original: *mut BaseNode,
        skip: Option<*mut BaseNode>,
    ) -> BaseNodeIterator<fn(&BaseNode) -> *mut BaseNode> {
        BaseNodeIterator {
            original,
            current: original,
            skip,
            direction: |node: &BaseNode| node.down,
        }
    }

    pub fn left(
        original: *const BaseNode,
        skip: Option<*const BaseNode>,
    ) -> Map<BaseNodeIterator<fn(&BaseNode) -> *mut BaseNode>, fn(*mut BaseNode) -> *const BaseNode>
    {
        left_mut(original as *mut _, skip.map(|skip| skip as *mut _)).map(|ptr| ptr as *const _)
    }

    pub fn left_mut(
        original: *mut BaseNode,
        skip: Option<*mut BaseNode>,
    ) -> BaseNodeIterator<fn(&BaseNode) -> *mut BaseNode> {
        BaseNodeIterator {
            original,
            current: original,
            skip,
            direction: |node: &BaseNode| node.left,
        }
    }

    pub fn right(
        original: *const BaseNode,
        skip: Option<*const BaseNode>,
    ) -> Map<BaseNodeIterator<fn(&BaseNode) -> *mut BaseNode>, fn(*mut BaseNode) -> *const BaseNode>
    {
        right_mut(original as *mut _, skip.map(|skip| skip as *mut _)).map(|ptr| ptr as *const _)
    }

    pub fn right_mut(
        original: *mut BaseNode,
        skip: Option<*mut BaseNode>,
    ) -> BaseNodeIterator<fn(&BaseNode) -> *mut BaseNode> {
        BaseNodeIterator {
            original,
            current: original,
            skip,
            direction: |node: &BaseNode| node.right,
        }
    }
}

#[derive(Debug)]
pub struct BaseNodeIterator<Func> {
    original: *mut BaseNode,
    current: *mut BaseNode,
    skip: Option<*mut BaseNode>,
    direction: Func,
}

impl<Func> Iterator for BaseNodeIterator<Func>
where
    Func: FnMut(&BaseNode) -> *mut BaseNode,
{
    type Item = *mut BaseNode;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = (self.direction)(unsafe { self.current.as_ref().unwrap() });

            if next == self.original {
                return None;
            } else if Some(next) == self.skip {
                self.current = next;

                continue;
            } else {
                self.current = next;

                return Some(next);
            }
        }
    }
}
