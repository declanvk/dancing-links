use std::slice;

pub struct WindowsMut<'a, T: 'a> {
    data: &'a mut [T],
    current_idx: usize,
    size: usize,
}

impl<'a, T: 'a> WindowsMut<'a, T> {
    pub fn new(data: &'a mut [T], size: usize) -> Self {
        WindowsMut {
            data,
            size,
            current_idx: 0,
        }
    }

    pub fn next(&mut self) -> Option<&mut [T]> {
        if self.current_idx + self.size > self.data.len() {
            None
        } else {
            unsafe {
                let data_ptr = self.data.as_mut_ptr().offset(self.current_idx as isize);

                self.current_idx += 1;
                Some(slice::from_raw_parts_mut(data_ptr, self.size))
            }
        }
    }
}

pub fn get_pair_mut<'a, T: 'a>(data: &'a mut [T], idx_a: usize, idx_b: usize) -> (&'a mut T, &'a mut T) {
    if idx_a >= data.len() || idx_b >= data.len() {
        panic!("Attempted to index beyond bounds of array");
    } else if idx_a == idx_b {
        panic!("Attempted to mutably alias same element twice");
    } else {
        unsafe {
            let data_ptr = data.as_mut_ptr();

            let item_a = data_ptr.offset(idx_a as isize).as_mut().unwrap();
            let item_b = data_ptr.offset(idx_b as isize).as_mut().unwrap();
            (item_a, item_b)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn windows_over_vec() {
        let mut data = vec![1, 2, 3, 4, 5];

        let mut windows = WindowsMut::new(&mut data, 2);

        assert_eq!(windows.next().unwrap(), &mut [1, 2]);
        assert_eq!(windows.next().unwrap(), &mut [2, 3]);
        assert_eq!(windows.next().unwrap(), &mut [3, 4]);
        assert_eq!(windows.next().unwrap(), &mut [4, 5]);
    }
}
