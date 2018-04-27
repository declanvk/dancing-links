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
