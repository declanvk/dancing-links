pub fn three_combination_iter(
    limits: [usize; 3],
    start: [usize; 3],
) -> impl Iterator<Item = [usize; 3]> {
    (start[0]..limits[0]).flat_map(move |first| {
        (start[1]..limits[1])
            .flat_map(move |second| (start[2]..limits[2]).map(move |third| [first, second, third]))
    })
}

pub fn two_combination_iter(
    limits: [usize; 2],
    start: [usize; 2],
) -> impl Iterator<Item = [usize; 2]> {
    (start[0]..limits[0])
        .flat_map(move |first| (start[1]..limits[1]).map(move |second| [first, second]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_combo() {
        let it = three_combination_iter([2, 4, 6], [0, 2, 4]);

        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                [0, 2, 4],
                [0, 2, 5],
                [0, 3, 4],
                [0, 3, 5],
                [1, 2, 4],
                [1, 2, 5],
                [1, 3, 4],
                [1, 3, 5],
            ]
        );
    }

    #[test]
    fn latin_square_test_1() {
        let it = three_combination_iter([2, 2, 2 + 1], [0, 0, 1]);

        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                [0, 0, 1],
                [0, 0, 2],
                [0, 1, 1],
                [0, 1, 2],
                [1, 0, 1],
                [1, 0, 2],
                [1, 1, 1],
                [1, 1, 2],
            ]
        );
    }

    #[test]
    fn latin_square_test_3() {
        let it = three_combination_iter([4, 4, 5], [0, 0, 1]);

        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                [0, 0, 1],
                [0, 0, 2],
                [0, 0, 3],
                [0, 0, 4],
                [0, 1, 1],
                [0, 1, 2],
                [0, 1, 3],
                [0, 1, 4],
                [0, 2, 1],
                [0, 2, 2],
                [0, 2, 3],
                [0, 2, 4],
                [0, 3, 1],
                [0, 3, 2],
                [0, 3, 3],
                [0, 3, 4],
                [1, 0, 1],
                [1, 0, 2],
                [1, 0, 3],
                [1, 0, 4],
                [1, 1, 1],
                [1, 1, 2],
                [1, 1, 3],
                [1, 1, 4],
                [1, 2, 1],
                [1, 2, 2],
                [1, 2, 3],
                [1, 2, 4],
                [1, 3, 1],
                [1, 3, 2],
                [1, 3, 3],
                [1, 3, 4],
                [2, 0, 1],
                [2, 0, 2],
                [2, 0, 3],
                [2, 0, 4],
                [2, 1, 1],
                [2, 1, 2],
                [2, 1, 3],
                [2, 1, 4],
                [2, 2, 1],
                [2, 2, 2],
                [2, 2, 3],
                [2, 2, 4],
                [2, 3, 1],
                [2, 3, 2],
                [2, 3, 3],
                [2, 3, 4],
                [3, 0, 1],
                [3, 0, 2],
                [3, 0, 3],
                [3, 0, 4],
                [3, 1, 1],
                [3, 1, 2],
                [3, 1, 3],
                [3, 1, 4],
                [3, 2, 1],
                [3, 2, 2],
                [3, 2, 3],
                [3, 2, 4],
                [3, 3, 1],
                [3, 3, 2],
                [3, 3, 3],
                [3, 3, 4]
            ]
        );
    }
}
