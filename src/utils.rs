pub fn get_pair_mut<'a, T: 'a>(
    data: &'a mut [T],
    idx_a: usize,
    idx_b: usize,
) -> (&'a mut T, &'a mut T) {
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
