macro_rules! create_nary_combination_iter {
    ($name:ident, $comp:ty, $arr:ty, $num:expr) => {
        pub struct $name {
            pub dimensions: $arr,
            pub next_value: $arr,
        }

        impl Iterator for $name {
            type Item = $arr;

            fn size_hint(&self) -> (usize, Option<usize>) {
                let size = self.dimensions.iter().copied().product::<$comp>() as usize;

                (size, Some(size))
            }

            fn next(&mut self) -> Option<Self::Item> {
                if self.dimensions.iter().any(|dim| *dim == 0) {
                    None
                } else {
                    let next_value = self.next_value.clone();
                    let mut did_break = false;
                    for idx in (0..$num).rev() {
                        if self.next_value[idx] < self.dimensions[idx] - 1 {
                            for above_idx in (idx..$num) {
                                self.next_value[above_idx] = 0;
                            }

                            self.next_value[idx] += 1;
                            did_break = true;
                            break;
                        }
                    }

                    if !did_break {
                        self.dimensions = [0; $num];
                    }

                    Some(next_value)
                }
            }
        }
    };
}

create_nary_combination_iter!(ThreeCombinationIter, usize, [usize; 3], 3);
create_nary_combination_iter!(TwoCombinationIter, usize, [usize; 2], 2);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_combination_iter() {
        let it = ThreeCombinationIter {
            dimensions: [2, 2, 2],
            next_value: [0, 0, 0],
        };

        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                [0, 0, 0],
                [0, 0, 1],
                [0, 1, 0],
                [0, 1, 1],
                [1, 0, 0],
                [1, 0, 1],
                [1, 1, 0],
                [1, 1, 1],
            ]
        );
    }
}
