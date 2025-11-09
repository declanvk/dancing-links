//! A [Polyomino tiling puzzle](https://en.wikipedia.org/wiki/Polyomino#Tiling_with_polyominoes)
//! is a tiling of a rectangular grid with polyominoes, where each polyomino
//! represents a specific shape and must be placed in the grid without
//! overlaps or gaps.

use crate::ExactCover;
use std::rc::Rc;

/// Type representing shape of a single polyomino, encoded as binary mask.
/// `PShape` is represented as a vector of `height * width` elements,
/// where only width is explicitly stored as a structure field.
/// In this representation rows are stored consecutively - element of `i`th row
/// and `j`th column is available under index `i * width + j`.
/// 0 represents an empty cell, 1 represents a filled cell.
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct PShape {
    /// Width of polyomino
    pub width: usize,
    /// Binary mask encoded in row-first manner.
    pub mask: Vec<u8>,
}

impl PShape {
    /// Create a new PShape of a given width and mask;
    /// Empty rows and columns on the edges will be truncated,
    /// so values of `init_width` and `init_mask` parameters may be changed
    /// before storing to the structure instance.
    pub fn new(init_width: usize, init_mask: Vec<u8>) -> Self {
        assert!(init_width != 0, "Width of shape must be non-zero.");
        assert!(!init_mask.is_empty(), "Mask cannot be empty.");
        assert!(
            init_mask.len() % init_width == 0,
            "Mask with incorrect length - incorrect number of elements supplied."
        );

        // From each side of shape, find index of a first row/column with at least one
        // non-zero element.

        let init_height = init_mask.len() / init_width;

        let (mut r1, mut r2) = (0usize, init_height - 1);
        let (mut c1, mut c2) = (0usize, init_width - 1);

        while r1 < init_height
            && init_mask[r1 * init_width..(r1 + 1) * init_width]
                .iter()
                .all(|el| *el == 0)
        {
            r1 += 1;
        }

        assert!(r1 != init_height, "No ones found - PShape mask empty!");

        while r2 > r1
            && init_mask[r2 * init_width..(r2 + 1) * init_width]
                .iter()
                .all(|el| *el == 0)
        {
            r2 -= 1;
        }

        while c1 < init_width
            && init_mask
                .iter()
                .skip(c1)
                .step_by(init_width)
                .all(|el| *el == 0)
        {
            c1 += 1;
        }

        while c2 > c1
            && init_mask
                .iter()
                .skip(c2)
                .step_by(init_width)
                .all(|el| *el == 0)
        {
            c2 -= 1;
        }

        let width = c2 - c1 + 1;
        let height = r2 - r1 + 1;

        let mut mask = Vec::with_capacity(width * height);

        for i in 0..height {
            for j in 0..width {
                mask.push(init_mask[(i + r1) * init_width + (j + c1)]);
            }
        }

        Self { width, mask }
    }

    /// Get PShape width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get PShape height.
    pub fn height(&self) -> usize {
        self.mask.len() / self.width
    }
}

impl<const W: usize, const H: usize> From<[[u8; W]; H]> for PShape {
    fn from(arr: [[u8; W]; H]) -> Self {
        let mut mask = Vec::with_capacity(W * H);

        for row in arr {
            mask.extend_from_slice(&row);
        }

        Self::new(W, mask)
    }
}

/// Available transformations for polyomino shapes during tiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShapeTransform {
    /// No transformation applied to the shape.
    NoTransform,
    /// Allow rotations of the shape.
    PureRotation,
    /// Allow rotations and reflections of the shape.
    FullSymmetry,
}

/// Instance of a polyomino tiling puzzle.
#[derive(Debug)]
pub struct Polyomino {
    /// The list of possible placements of polyominoes in the grid.
    pub possibilities: Vec<Possibility>,
    /// The list of constraints that must be satisfied for the polyomino
    /// tiling puzzle.
    pub constraints: Vec<Constraint>,
    /// The dimensions of the grid.
    pub grid_dimensions: (usize, usize),
    /// The list of polyominoes available for tiling.
    pub polyominoes: Vec<PShape>,
    /// Group of available transformations for polyomino shapes.
    pub transformations: ShapeTransform,
}

impl Polyomino {
    /// Create a new instance of the polyomino tiling puzzle with the given
    /// grid dimensions, polyomino shapes, and type of allowed transformations.
    pub fn new(
        grid_dimensions: (usize, usize),
        polyominoes: Vec<PShape>,
        transformations: ShapeTransform,
    ) -> Self {
        assert!(
            grid_dimensions.0 > 0 && grid_dimensions.1 > 0,
            "Grid dimensions must be positive."
        );
        assert!(!polyominoes.is_empty(), "Polyominoes list cannot be empty.");

        let possibilities =
            Self::generate_all_possibilities(&polyominoes, grid_dimensions, transformations);
        let constraints = Constraint::all(grid_dimensions, polyominoes.len()).collect();

        Self {
            possibilities,
            constraints,
            grid_dimensions,
            polyominoes,
            transformations,
        }
    }

    /// Generate all possible placements of polyominoes in the grid based on the
    /// grid size, available shapes and transformations.
    fn generate_all_possibilities(
        polyominoes: &[PShape],
        grid_dim: (usize, usize),
        transform: ShapeTransform,
    ) -> Vec<Possibility> {
        polyominoes
            .iter()
            .enumerate()
            .flat_map(|(i_shape, shape)| {
                // Generate all symmetries of the shape based on the transformation type.
                let mut symmetries: Vec<PShape> = Self::generate_symmetries(shape, transform);
                // Eliminate duplicate symmetries.
                symmetries.sort();
                symmetries.dedup();
                // Generate all possible placements of the shape in the grid.
                symmetries
                    .into_iter()
                    .flat_map(|symmetry| {
                        let symmetry = Rc::new(symmetry);
                        let symmetry_height = symmetry.height();
                        let symmetry_width = symmetry.width();
                        // If the symmetry is larger than the grid, skip it.
                        if symmetry_height > grid_dim.0 || symmetry_width > grid_dim.1 {
                            return vec![];
                        }
                        (0..=grid_dim.0 - symmetry_height)
                            .flat_map(move |grid_row| {
                                (0..=grid_dim.1 - symmetry_width).map({
                                    let symmetry_ref = Rc::clone(&symmetry);
                                    move |grid_col| {
                                        let mut occupied_cells = vec![];
                                        for r in 0..symmetry_height {
                                            for c in 0..symmetry_width {
                                                if symmetry_ref.mask[r * symmetry_width + c] == 1 {
                                                    occupied_cells
                                                        .push((grid_row + r, grid_col + c));
                                                }
                                            }
                                        }
                                        Possibility {
                                            shape_index: i_shape,
                                            occupied_cells,
                                        }
                                    }
                                })
                            })
                            .collect()
                    })
                    .collect::<Vec<Possibility>>()
            })
            .collect()
    }

    fn generate_symmetries(shape: &PShape, transform: ShapeTransform) -> Vec<PShape> {
        match transform {
            ShapeTransform::NoTransform => vec![shape.clone()],
            ShapeTransform::PureRotation => Self::generate_rotations(shape),
            ShapeTransform::FullSymmetry => {
                let mut rotations = Self::generate_rotations(shape);
                let reflections: Vec<PShape> = rotations
                    .iter()
                    .map(|s| {
                        let width = s.width();
                        let mut reflected_mask = Vec::with_capacity(s.mask.len());
                        let mut row_it = s.mask.len() / width;

                        while row_it > 0 {
                            reflected_mask
                                .extend_from_slice(&s.mask[(row_it - 1) * width..row_it * width]);
                            row_it -= 1;
                        }

                        PShape::new(width, reflected_mask)
                    })
                    .collect();
                rotations.extend(reflections);
                rotations
            }
        }
    }

    fn generate_rotations(shape: &PShape) -> Vec<PShape> {
        let mut rotations = vec![shape.clone()];
        let mut current_shape = shape.clone();
        for _ in 0..3 {
            current_shape = Self::rotate(&current_shape);
            rotations.push(current_shape.clone());
        }
        rotations
    }

    fn rotate(shape: &PShape) -> PShape {
        let cols = shape.width();
        let rows = shape.height();
        let mut rotated_shape = Vec::with_capacity(shape.mask.len());

        for c in 0..cols {
            for r in (0..rows).rev() {
                rotated_shape.push(shape.mask[r * cols + c]);
            }
        }
        PShape::new(rows, rotated_shape)
    }
}

impl ExactCover for Polyomino {
    type Constraint = Constraint;
    type Possibility = Possibility;

    fn satisfies(&self, poss: &Self::Possibility, cons: &Self::Constraint) -> bool {
        poss.satisfies(cons)
    }

    fn is_optional(&self, _cons: &Self::Constraint) -> bool {
        false
    }

    fn possibilities(&self) -> &[Self::Possibility] {
        &self.possibilities
    }

    fn constraints(&self) -> &[Self::Constraint] {
        &self.constraints
    }
}

/// A possible placement of a polyomino in the grid.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Possibility {
    /// Index of the shape in the list of polyominoes.
    pub shape_index: usize,
    /// List of coordinates of the cells occupied by the polyomino in the grid.
    pub occupied_cells: Vec<(usize, usize)>,
}

impl Possibility {
    /// Check if this possibility satisfies a given constraint.
    pub fn satisfies(&self, constraint: &Constraint) -> bool {
        match constraint {
            Constraint::ShapeIndex(index) => self.shape_index == *index,
            Constraint::Field(row, col) => self.occupied_cells.contains(&(*row, *col)),
        }
    }

    /// Return an iterator over all `Constraint`s that are satisfied by this
    /// `Possibility`.
    pub fn satisfied_constraints<'a>(&'a self) -> impl Iterator<Item = Constraint> + 'a {
        let shape_constraint = Constraint::ShapeIndex(self.shape_index);
        let field_constraints = self
            .occupied_cells
            .iter()
            .map(|&(row, col)| Constraint::Field(row, col));
        std::iter::once(shape_constraint).chain(field_constraints)
    }
}

/// A condition that must be satisfied in order to solve a polyomino
/// tiling puzzle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Constraint {
    /// A constraint that a specific shape must be placed in the grid.
    /// Inner integer represents the index of the shape in the list
    /// of polyominoes.
    ShapeIndex(usize),
    /// A constraint that a specific cell in the grid must be filled.
    Field(usize, usize),
}

impl Constraint {
    /// Return an iterator over all possible `Constraint`s
    /// for a given grid size and number of polyominoes.
    pub fn all(
        grid_size: (usize, usize),
        polyomino_count: usize,
    ) -> impl Iterator<Item = Constraint> {
        let shape_it = (0..polyomino_count).map(Constraint::ShapeIndex);
        let field_it = (0..grid_size.0)
            .flat_map(move |row| (0..grid_size.1).map(move |col| Constraint::Field(row, col)));

        shape_it.chain(field_it)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new_pshape_no_truncate() {
        let pshape = PShape::new(3, vec![0, 0, 1, 1, 1, 1]);
        assert!(pshape.width == 3, "");
        assert!(pshape.mask == vec![0, 0, 1, 1, 1, 1]);
    }

    #[test]
    #[should_panic(expected = "Width of shape must be non-zero.")]
    fn test_new_pshape_zero_width() {
        let _pshape = PShape::new(0, vec![]);
    }

    #[test]
    #[should_panic(expected = "Mask cannot be empty.")]
    fn test_new_pshape_zero_length_mask() {
        let _pshape = PShape::new(1, vec![]);
    }

    #[test]
    #[should_panic(
        expected = "Mask with incorrect length - incorrect number of elements supplied."
    )]
    fn test_new_pshape_incorrect_mask_length() {
        let _pshape = PShape::new(2, vec![1, 0, 1, 1, 1]);
    }

    #[test]
    #[should_panic(expected = "No ones found - PShape mask empty!")]
    fn test_new_pshape_only_zeros() {
        let _pshape = PShape::new(3, vec![0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_new_pshape_left_truncate() {
        let pshape = PShape::new(4, vec![0, 0, 1, 1, 0, 0, 0, 1]);
        assert!(pshape.width == 2);
        assert!(pshape.mask == vec![1, 1, 0, 1]);
    }

    #[test]
    fn test_new_pshape_right_truncate() {
        let pshape = PShape::new(3, vec![1, 1, 0, 1, 0, 0]);
        assert!(pshape.width == 2);
        assert!(pshape.mask == vec![1, 1, 1, 0]);
    }

    #[test]
    fn test_new_pshape_top_truncate() {
        let pshape = PShape::new(3, vec![0, 0, 0, 1, 1, 0, 1, 0, 1]);
        assert!(pshape.width == 3);
        assert!(pshape.mask == vec![1, 1, 0, 1, 0, 1]);
    }

    #[test]
    fn test_new_pshape_bottom_truncate() {
        let pshape = PShape::new(2, vec![1, 0, 0, 1, 1, 0, 0, 0]);
        assert!(pshape.width == 2);
        assert!(pshape.mask == vec![1, 0, 0, 1, 1, 0]);
    }

    #[test]
    fn test_new_pshape_all_trucates() {
        let pshape = PShape::new(
            5,
            vec![
                0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0,
            ],
        );
        assert!(pshape.width == 3);
        assert!(pshape.mask == vec![1, 0, 1, 0, 1, 0, 1, 0, 1]);
    }

    #[test]
    fn test_pshape_from_array() {
        assert_eq!(
            PShape::from([[1, 0, 1], [0, 1, 0], [0, 1, 0]]),
            PShape::new(3, vec![1, 0, 1, 0, 1, 0, 0, 1, 0])
        );
        assert_eq!(
            PShape::from([[0, 0, 0, 0], [0, 1, 1, 0], [0, 1, 1, 0], [0, 0, 0, 0]]),
            PShape::new(2, vec![1, 1, 1, 1])
        );
    }

    #[test]
    fn test_possibility_satisfied_constraints() {
        let possibility = Possibility {
            shape_index: 0,
            occupied_cells: vec![(0, 0), (0, 1), (1, 0)],
        };
        let constraints: Vec<_> = possibility.satisfied_constraints().collect();
        assert_eq!(constraints.len(), 4); // 1 shape + 3 fields
        assert!(constraints.contains(&Constraint::ShapeIndex(0)));
        assert!(constraints.contains(&Constraint::Field(0, 0)));
        assert!(constraints.contains(&Constraint::Field(0, 1)));
        assert!(constraints.contains(&Constraint::Field(1, 0)));
    }

    #[test]
    fn test_polyomino_constraints() {
        let constraints: Vec<_> = Constraint::all((3, 4), 2).collect();
        assert_eq!(constraints.len(), 14); // 2 shapes + 12 fields
        assert!(constraints.contains(&Constraint::ShapeIndex(0)));
        assert!(constraints.contains(&Constraint::ShapeIndex(1)));
        assert!(constraints.contains(&Constraint::Field(0, 0)));
        assert!(constraints.contains(&Constraint::Field(0, 1)));
        assert!(constraints.contains(&Constraint::Field(0, 2)));
        assert!(constraints.contains(&Constraint::Field(0, 3)));
        assert!(constraints.contains(&Constraint::Field(1, 0)));
        assert!(constraints.contains(&Constraint::Field(1, 1)));
        assert!(constraints.contains(&Constraint::Field(1, 2)));
        assert!(constraints.contains(&Constraint::Field(1, 3)));
        assert!(constraints.contains(&Constraint::Field(2, 0)));
        assert!(constraints.contains(&Constraint::Field(2, 1)));
        assert!(constraints.contains(&Constraint::Field(2, 2)));
        assert!(constraints.contains(&Constraint::Field(2, 3)));
    }

    #[test]
    fn test_polyomino_creation() {
        let polyominoes = vec![
            PShape::from([[1, 1], [1, 0], [1, 0]]), // J-shape
            PShape::from([[0, 1], [1, 1], [1, 0]]), // Z-shape
            PShape::from([[0, 1], [0, 1], [1, 1]]), // J-shape
        ];
        let grid_dimensions = (3, 4);
        let transformations = ShapeTransform::NoTransform;

        let polyomino = Polyomino::new(grid_dimensions, polyominoes, transformations);

        assert_eq!(polyomino.grid_dimensions, (3, 4));
        assert_eq!(polyomino.polyominoes.len(), 3);
        assert_eq!(polyomino.transformations, ShapeTransform::NoTransform);
    }

    #[test]
    #[should_panic(expected = "Grid dimensions must be positive.")]
    fn test_invalid_grid_dimensions() {
        let _polyomino = Polyomino::new(
            (0, 4),
            vec![PShape::from([[1]])],
            ShapeTransform::NoTransform,
        );
    }

    #[test]
    #[should_panic(expected = "Polyominoes list cannot be empty.")]
    fn test_empty_list_polyomino() {
        let _polyomino = Polyomino::new((3, 4), vec![], ShapeTransform::NoTransform);
    }

    #[test]
    fn test_removal_empty_rows_and_columns() {
        let shape1: PShape = PShape::from([
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 1, 1, 0, 0, 0],
            [0, 0, 1, 0, 0, 1, 0],
            [0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
        ]);
        let expected1: PShape = PShape::from([
            [1, 1, 0, 0],
            [1, 0, 0, 1],
            [1, 1, 0, 0],
            [0, 0, 0, 0],
            [0, 1, 0, 0],
            [0, 1, 0, 0],
        ]);
        assert_eq!(shape1, expected1);
        let shape2: PShape = PShape::from([
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ]);
        let expected2: PShape = PShape::from([
            [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
        ]);
        assert_eq!(shape2, expected2);
    }

    #[test]
    fn test_rotate() {
        let shape = PShape::from([[1, 0, 0], [1, 1, 1]]);
        let rotated_shape = Polyomino::rotate(&shape);
        let expected_shape = PShape::from([[1, 1], [1, 0], [1, 0]]);
        assert_eq!(rotated_shape, expected_shape);
    }

    #[test]
    fn test_generate_rotations() {
        let shape = PShape::from([[1, 0, 0], [1, 1, 1]]);
        let rotations = Polyomino::generate_rotations(&shape);
        let expected1 = PShape::from([[1, 1], [1, 0], [1, 0]]);
        let expected2 = PShape::from([[1, 1, 1], [0, 0, 1]]);
        let expected3 = PShape::from([[0, 1], [0, 1], [1, 1]]);
        assert_eq!(rotations.len(), 4);
        assert_eq!(rotations[0], shape);
        assert_eq!(rotations[1], expected1);
        assert_eq!(rotations[2], expected2);
        assert_eq!(rotations[3], expected3);
    }

    #[test]
    fn test_generate_symmetries() {
        let shape = PShape::from([[1, 0, 0], [1, 1, 1]]);
        let symmetries_no_transform =
            Polyomino::generate_symmetries(&shape, ShapeTransform::NoTransform);
        assert_eq!(symmetries_no_transform.len(), 1);
        assert_eq!(symmetries_no_transform[0], shape);

        let symmetries_rotation =
            Polyomino::generate_symmetries(&shape, ShapeTransform::PureRotation);
        assert_eq!(symmetries_rotation.len(), 4);
        assert_eq!(symmetries_rotation, Polyomino::generate_rotations(&shape));

        let symmetries_full_symmetry =
            Polyomino::generate_symmetries(&shape, ShapeTransform::FullSymmetry);
        let expected4 = PShape::from([[1, 1, 1], [1, 0, 0]]);
        let expected5 = PShape::from([[1, 0], [1, 0], [1, 1]]);
        let expected6 = PShape::from([[0, 0, 1], [1, 1, 1]]);
        let expected7 = PShape::from([[1, 1], [0, 1], [0, 1]]);
        assert_eq!(symmetries_full_symmetry.len(), 8);
        assert_eq!(
            &symmetries_full_symmetry[..4],
            Polyomino::generate_rotations(&shape)
        );
        assert_eq!(
            &symmetries_full_symmetry[4..],
            vec![expected4, expected5, expected6, expected7]
        );
    }

    #[test]
    fn test_generate_single_possibility() {
        let shape = PShape::from([[1, 1], [1, 1]]);
        let grid_dimensions = (2, 2);
        let transform = ShapeTransform::FullSymmetry;

        let possibilities =
            Polyomino::generate_all_possibilities(&[shape], grid_dimensions, transform);
        assert_eq!(possibilities.len(), 1); // 1 possible placement in a 2x2 grid

        let expected_possibilities = vec![Possibility {
            shape_index: 0,
            occupied_cells: vec![(0, 0), (0, 1), (1, 0), (1, 1)],
        }];

        assert_eq!(possibilities, expected_possibilities);
    }

    #[test]
    fn test_generate_no_possibilities() {
        let shape = PShape::from([[1, 1], [1, 1]]);
        let grid_dimensions = (1, 1);
        let transform = ShapeTransform::FullSymmetry;

        let possibilities =
            Polyomino::generate_all_possibilities(&[shape], grid_dimensions, transform);
        assert!(possibilities.is_empty()); // No possible placements in a 1x1
                                           // grid
    }

    #[test]
    fn test_generate_only_fitting_possibilities() {
        let shape = PShape::from([[1, 0], [1, 0], [1, 1]]);
        let grid_dimensions = (2, 3);
        let transform = ShapeTransform::PureRotation;

        let possibilities =
            Polyomino::generate_all_possibilities(&[shape], grid_dimensions, transform);
        assert_eq!(possibilities.len(), 2); // 2 possible placements in a 2x3 grid

        let expected_possibilities = vec![
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 2), (1, 0), (1, 1), (1, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 0), (0, 1), (0, 2), (1, 0)],
            },
        ];

        assert_eq!(possibilities, expected_possibilities);
    }

    #[test]
    fn test_generate_possibilities_no_transformations() {
        let shapes = vec![
            PShape::from([[1, 1], [1, 0], [1, 0]]), // J-shape
            PShape::from([[0, 1], [1, 1], [1, 0]]), // Z-shape
            PShape::from([[0, 1], [0, 1], [1, 1]]), // J-shape
        ];
        let grid_dimensions = (3, 4);
        let transform = ShapeTransform::NoTransform;

        let possibilities =
            Polyomino::generate_all_possibilities(&shapes, grid_dimensions, transform);

        assert_eq!(possibilities.len(), 9); // 9 possible placements for the shapes in a 3x4 grid
        let expected_possibilities = vec![
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 0), (0, 1), (1, 0), (2, 0)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 1), (0, 2), (1, 1), (2, 1)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 2), (0, 3), (1, 2), (2, 2)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 1), (1, 0), (1, 1), (2, 0)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 2), (1, 1), (1, 2), (2, 1)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 3), (1, 2), (1, 3), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 1), (1, 1), (2, 0), (2, 1)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 2), (1, 2), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 3), (1, 3), (2, 2), (2, 3)],
            },
        ];
        let possibilities: HashSet<Possibility> = HashSet::from_iter(possibilities);
        let expected_possibilities: HashSet<Possibility> =
            HashSet::from_iter(expected_possibilities);
        assert_eq!(possibilities, expected_possibilities);
    }

    #[test]
    fn test_generate_possibilities_pure_rotation() {
        let shapes = vec![
            PShape::from([[1, 1], [1, 0], [1, 0]]), // J-shape
            PShape::from([[0, 1], [1, 1], [1, 0]]), // Z-shape
            PShape::from([[0, 1], [0, 1], [1, 1]]), // J-shape
        ];
        let grid_dimensions = (3, 4);
        let transform = ShapeTransform::PureRotation;

        let possibilities =
            Polyomino::generate_all_possibilities(&shapes, grid_dimensions, transform);

        assert_eq!(possibilities.len(), 35); // 35 possible placements for the shapes in a 3x4 grid
        let expected_possibilities = vec![
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 1), (1, 1), (2, 0), (2, 1)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 2), (1, 2), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 3), (1, 3), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 0), (1, 0), (1, 1), (1, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 1), (1, 1), (1, 2), (1, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(1, 0), (2, 0), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(1, 1), (2, 1), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 0), (0, 1), (1, 0), (2, 0)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 1), (0, 2), (1, 1), (2, 1)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 2), (0, 3), (1, 2), (2, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 0), (0, 1), (0, 2), (1, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 1), (0, 2), (0, 3), (1, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(1, 0), (1, 1), (1, 2), (2, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(1, 1), (1, 2), (1, 3), (2, 3)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 1), (1, 0), (1, 1), (2, 0)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 2), (1, 1), (1, 2), (2, 1)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 3), (1, 2), (1, 3), (2, 2)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 0), (0, 1), (1, 1), (1, 2)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 1), (0, 2), (1, 2), (1, 3)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(1, 0), (1, 1), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(1, 1), (1, 2), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 1), (1, 1), (2, 0), (2, 1)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 2), (1, 2), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 3), (1, 3), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 0), (1, 0), (1, 1), (1, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 1), (1, 1), (1, 2), (1, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(1, 0), (2, 0), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(1, 1), (2, 1), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 0), (0, 1), (1, 0), (2, 0)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 1), (0, 2), (1, 1), (2, 1)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 2), (0, 3), (1, 2), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 0), (0, 1), (0, 2), (1, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 1), (0, 2), (0, 3), (1, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(1, 0), (1, 1), (1, 2), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(1, 1), (1, 2), (1, 3), (2, 3)],
            },
        ];
        let possibilities: HashSet<Possibility> = HashSet::from_iter(possibilities);
        let expected_possibilities: HashSet<Possibility> =
            HashSet::from_iter(expected_possibilities);
        assert_eq!(possibilities, expected_possibilities);
    }

    #[test]
    fn test_generate_possibilities_full_symmetry() {
        let shapes = vec![
            PShape::from([[1, 1], [1, 0], [1, 0]]), // J-shape
            PShape::from([[0, 1], [1, 1], [1, 0]]), // Z-shape
            PShape::from([[0, 1], [0, 1], [1, 1]]), // J-shape
        ];
        let grid_dimensions = (3, 4);
        let transform = ShapeTransform::FullSymmetry;

        let possibilities =
            Polyomino::generate_all_possibilities(&shapes, grid_dimensions, transform);

        assert_eq!(possibilities.len(), 70); // 70 possible placements for the shapes in a 3x4 grid

        let expected_possibilities = vec![
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 2), (1, 0), (1, 1), (1, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 3), (1, 1), (1, 2), (1, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(1, 2), (2, 0), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(1, 3), (2, 1), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 1), (1, 1), (2, 0), (2, 1)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 2), (1, 2), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 3), (1, 3), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 0), (1, 0), (2, 0), (2, 1)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 1), (1, 1), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 2), (1, 2), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 0), (1, 0), (1, 1), (1, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 1), (1, 1), (1, 2), (1, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(1, 0), (2, 0), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(1, 1), (2, 1), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 0), (0, 1), (1, 1), (2, 1)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 1), (0, 2), (1, 2), (2, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 2), (0, 3), (1, 3), (2, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 0), (0, 1), (1, 0), (2, 0)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 1), (0, 2), (1, 1), (2, 1)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 2), (0, 3), (1, 2), (2, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 0), (0, 1), (0, 2), (1, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 1), (0, 2), (0, 3), (1, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(1, 0), (1, 1), (1, 2), (2, 2)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(1, 1), (1, 2), (1, 3), (2, 3)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 0), (0, 1), (0, 2), (1, 0)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(0, 1), (0, 2), (0, 3), (1, 1)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(1, 0), (1, 1), (1, 2), (2, 0)],
            },
            Possibility {
                shape_index: 0,
                occupied_cells: vec![(1, 1), (1, 2), (1, 3), (2, 1)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 1), (1, 0), (1, 1), (2, 0)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 2), (1, 1), (1, 2), (2, 1)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 3), (1, 2), (1, 3), (2, 2)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 1), (0, 2), (1, 0), (1, 1)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 2), (0, 3), (1, 1), (1, 2)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(1, 1), (1, 2), (2, 0), (2, 1)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(1, 2), (1, 3), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 0), (1, 0), (1, 1), (2, 1)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 1), (1, 1), (1, 2), (2, 2)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 2), (1, 2), (1, 3), (2, 3)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 0), (0, 1), (1, 1), (1, 2)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(0, 1), (0, 2), (1, 2), (1, 3)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(1, 0), (1, 1), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 1,
                occupied_cells: vec![(1, 1), (1, 2), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 2), (1, 0), (1, 1), (1, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 3), (1, 1), (1, 2), (1, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(1, 2), (2, 0), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(1, 3), (2, 1), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 1), (1, 1), (2, 0), (2, 1)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 2), (1, 2), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 3), (1, 3), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 0), (1, 0), (2, 0), (2, 1)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 1), (1, 1), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 2), (1, 2), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 0), (1, 0), (1, 1), (1, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 1), (1, 1), (1, 2), (1, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(1, 0), (2, 0), (2, 1), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(1, 1), (2, 1), (2, 2), (2, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 0), (0, 1), (1, 1), (2, 1)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 1), (0, 2), (1, 2), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 2), (0, 3), (1, 3), (2, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 0), (0, 1), (1, 0), (2, 0)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 1), (0, 2), (1, 1), (2, 1)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 2), (0, 3), (1, 2), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 0), (0, 1), (0, 2), (1, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 1), (0, 2), (0, 3), (1, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(1, 0), (1, 1), (1, 2), (2, 2)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(1, 1), (1, 2), (1, 3), (2, 3)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 0), (0, 1), (0, 2), (1, 0)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(0, 1), (0, 2), (0, 3), (1, 1)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(1, 0), (1, 1), (1, 2), (2, 0)],
            },
            Possibility {
                shape_index: 2,
                occupied_cells: vec![(1, 1), (1, 2), (1, 3), (2, 1)],
            },
        ];
        let possibilities: HashSet<Possibility> = HashSet::from_iter(possibilities);
        let expected_possibilities: HashSet<Possibility> =
            HashSet::from_iter(expected_possibilities);
        assert_eq!(possibilities, expected_possibilities);
    }

    #[test]
    fn test_new_polyomino_puzzle() {
        let shapes = vec![
            PShape::from([[1, 1], [1, 0], [1, 0]]), // J-shape
            PShape::from([[0, 1], [1, 1], [1, 0]]), // Z-shape
            PShape::from([[0, 1], [0, 1], [1, 1]]), // J-shape
        ];
        let grid_dimensions = (3, 4);
        let transform = ShapeTransform::NoTransform;
        let polyomino = Polyomino::new(grid_dimensions, shapes, transform);
        assert_eq!(polyomino.grid_dimensions, (3, 4));
        assert_eq!(
            polyomino.polyominoes,
            vec![
                PShape::from([[1, 1], [1, 0], [1, 0]]), // J-shape
                PShape::from([[0, 1], [1, 1], [1, 0]]), // Z-shape
                PShape::from([[0, 1], [0, 1], [1, 1]]), // J-shape
            ]
        );
        assert_eq!(polyomino.transformations, ShapeTransform::NoTransform);
        assert_eq!(polyomino.possibilities.len(), 9);
        assert_eq!(
            polyomino.possibilities,
            Polyomino::generate_all_possibilities(
                &polyomino.polyominoes,
                polyomino.grid_dimensions,
                polyomino.transformations,
            )
        );
        assert_eq!(polyomino.constraints.len(), 15); // 3 shape + 12 field
        assert_eq!(
            polyomino.constraints,
            Constraint::all(polyomino.grid_dimensions, polyomino.polyominoes.len())
                .collect::<Vec<Constraint>>()
        );
    }

    #[test]
    fn test_solve_small_puzzle() {
        let shapes = vec![
            PShape::from([[1, 1], [1, 0], [1, 0]]), // J-shape
            PShape::from([[0, 1], [1, 1], [1, 0]]), // Z-shape
            PShape::from([[0, 1], [0, 1], [1, 1]]), // J-shape
        ];
        let grid_dimensions = (3, 4);
        let transform = ShapeTransform::NoTransform;
        let polyomino = Polyomino::new(grid_dimensions, shapes, transform);

        let mut solver = polyomino.solver();
        let solution = solver.next();
        assert!(solution.is_some());
        let mut solution = solution.unwrap();
        solution.sort();
        assert_eq!(solution.len(), 3);
        assert_eq!(
            solution.into_iter().cloned().collect::<Vec<Possibility>>(),
            vec![
                Possibility {
                    shape_index: 0,
                    occupied_cells: vec![(0, 0), (0, 1), (1, 0), (2, 0)]
                },
                Possibility {
                    shape_index: 1,
                    occupied_cells: vec![(0, 2), (1, 1), (1, 2), (2, 1)]
                },
                Possibility {
                    shape_index: 2,
                    occupied_cells: vec![(0, 3), (1, 3), (2, 2), (2, 3)]
                }
            ]
        );
        assert!(solver.next().is_none()); // No more solutions
    }
}
