//! A [Polyomino tiling puzzle](https://en.wikipedia.org/wiki/Polyomino#Tiling_with_polyominoes)
//! is a tiling of a rectangular grid with polyominoes, where each polyomino
//! represents a specific shape and must be placed in the grid without
//! overlaps or gaps.

use crate::ExactCover;
use std::rc::Rc;

/// Type representing shape of a single polyomino, encoded as binary mask.
/// 0 represents an empty cell, 1 represents a filled cell.
type PShape = Vec<Vec<u8>>;

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

        // For each shape, remove rows and columns near the edge that are completely
        // empty.
        let polyominoes: Vec<_> = polyominoes
            .into_iter()
            .inspect(|shape| {
                assert!(!shape.is_empty(), "Polyomino shape cannot be empty.");
                assert!(
                    shape.iter().all(|row| !row.is_empty()),
                    "Polyomino shape rows cannot be empty."
                );
                assert!(
                    shape.iter().all(|row| row.len() == shape[0].len()),
                    "All rows in a polyomino shape must have the same length."
                );
            })
            .map(|shape| {
                let mut shape = shape.clone();
                // Keep removing empty rows from the top and bottom
                // until no more can be removed.
                while let Some(last_row) = shape.last() {
                    if last_row.iter().all(|&cell| cell == 0) {
                        shape.pop();
                    } else {
                        break;
                    }
                }
                while !shape.is_empty() && shape[0].iter().all(|&cell| cell == 0) {
                    shape.remove(0);
                }
                // Keep removing empty columns from the left and right
                // until no more can be removed.
                for col in (0..shape[0].len()).rev() {
                    if shape.iter().all(|row| row[col] == 0) {
                        for row in &mut shape {
                            row.remove(col);
                        }
                    } else {
                        break;
                    }
                }
                while !shape.is_empty() && shape.iter().all(|row| row[0] == 0) {
                    for row in &mut shape {
                        row.remove(0);
                    }
                }
                shape
            })
            .collect();

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
                        let symmetry_height = symmetry.len();
                        let symmetry_width = symmetry[0].len();
                        // If the symmetry is larger than the grid, skip it.
                        if symmetry_height > grid_dim.0 || symmetry_width > grid_dim.1 {
                            return vec![];
                        }
                        (0..=grid_dim.0 - symmetry_height)
                            .flat_map(move |grid_row| {
                                (0..=grid_dim.1 - symmetry_width).map({
                                    let symmetry_ref = Rc::clone(&symmetry);
                                    move |grid_col| {
                                        let occupied_cells: Vec<(usize, usize)> = symmetry_ref
                                            .iter()
                                            .enumerate()
                                            .flat_map(|(r, row)| {
                                                row.iter().enumerate().filter_map(
                                                    move |(c, &cell)| {
                                                        if cell == 1 {
                                                            Some((grid_row + r, grid_col + c))
                                                        } else {
                                                            None
                                                        }
                                                    },
                                                )
                                            })
                                            .collect();
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
                        let mut reflected_shape = s.clone();
                        reflected_shape.reverse();
                        reflected_shape
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
        let rows = shape.len();
        let cols = shape[0].len();
        let mut rotated_shape = vec![vec![0; rows]; cols];
        for (r, row) in shape.iter().enumerate() {
            for (c, &cell) in row.iter().enumerate() {
                rotated_shape[c][rows - 1 - r] = cell;
            }
        }
        rotated_shape
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
            vec![vec![1, 1], vec![1, 0], vec![1, 0]], // J-shape
            vec![vec![0, 1], vec![1, 1], vec![1, 0]], // Z-shape
            vec![vec![0, 1], vec![0, 1], vec![1, 1]], // J-shape
        ];
        let grid_dimensions = (3, 4);
        let transformations = ShapeTransform::NoTransform;

        let polyomino = Polyomino::new(grid_dimensions, polyominoes, transformations);

        assert_eq!(polyomino.grid_dimensions, (3, 4));
        assert_eq!(polyomino.polyominoes.len(), 3);
        assert_eq!(polyomino.transformations, ShapeTransform::NoTransform);
    }

    #[test]
    #[should_panic(expected = "Polyominoes list cannot be empty.")]
    fn test_empty_list_polyomino() {
        let _polyomino = Polyomino::new((3, 4), vec![], ShapeTransform::NoTransform);
    }

    #[test]
    #[should_panic(expected = "Grid dimensions must be positive.")]
    fn test_invalid_grid_dimensions() {
        let _polyomino = Polyomino::new((0, 4), vec![vec![vec![1]]], ShapeTransform::NoTransform);
    }

    #[test]
    #[should_panic(expected = "Polyomino shape cannot be empty.")]
    fn test_empty_polyomino_shape() {
        let _polyomino = Polyomino::new((3, 4), vec![vec![]], ShapeTransform::NoTransform);
    }

    #[test]
    #[should_panic(expected = "Polyomino shape rows cannot be empty.")]
    fn test_empty_polyomino_shape_row() {
        let _polyomino = Polyomino::new((3, 4), vec![vec![vec![]]], ShapeTransform::NoTransform);
    }

    #[test]
    #[should_panic(expected = "All rows in a polyomino shape must have the same length.")]
    fn test_polyomino_shape_row_length() {
        let _polyomino = Polyomino::new(
            (3, 4),
            vec![vec![vec![1, 0], vec![1]]],
            ShapeTransform::NoTransform,
        );
    }

    #[test]
    fn test_removal_empty_rows_and_columns() {
        let shape1: PShape = vec![
            vec![0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 1, 1, 0, 0, 0],
            vec![0, 0, 1, 0, 0, 1, 0],
            vec![0, 0, 1, 1, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0],
        ];
        let expected1: PShape = vec![
            vec![1, 1, 0, 0],
            vec![1, 0, 0, 1],
            vec![1, 1, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 1, 0, 0],
            vec![0, 1, 0, 0],
        ];
        let shape2: PShape = vec![
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];
        let expected2: PShape = vec![
            vec![1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
        ];
        let polyomino = Polyomino::new((4, 3), vec![shape1, shape2], ShapeTransform::NoTransform);

        assert_eq!(polyomino.polyominoes.len(), 2);
        assert_eq!(polyomino.polyominoes[0], expected1);
        assert_eq!(polyomino.polyominoes[1], expected2);
    }

    #[test]
    fn test_rotate() {
        let shape: PShape = vec![vec![1, 0, 0], vec![1, 1, 1]];
        let rotated_shape = Polyomino::rotate(&shape);
        let expected_shape: PShape = vec![vec![1, 1], vec![1, 0], vec![1, 0]];
        assert_eq!(rotated_shape, expected_shape);
    }

    #[test]
    fn test_generate_rotations() {
        let shape: PShape = vec![vec![1, 0, 0], vec![1, 1, 1]];
        let rotations = Polyomino::generate_rotations(&shape);
        let expected1: PShape = vec![vec![1, 1], vec![1, 0], vec![1, 0]];
        let expected2: PShape = vec![vec![1, 1, 1], vec![0, 0, 1]];
        let expected3: PShape = vec![vec![0, 1], vec![0, 1], vec![1, 1]];
        assert_eq!(rotations.len(), 4);
        assert_eq!(rotations[0], shape);
        assert_eq!(rotations[1], expected1);
        assert_eq!(rotations[2], expected2);
        assert_eq!(rotations[3], expected3);
    }

    #[test]
    fn test_generate_symmetries() {
        let shape: PShape = vec![vec![1, 0, 0], vec![1, 1, 1]];
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
        let expected4: PShape = vec![vec![1, 1, 1], vec![1, 0, 0]];
        let expected5: PShape = vec![vec![1, 0], vec![1, 0], vec![1, 1]];
        let expected6: PShape = vec![vec![0, 0, 1], vec![1, 1, 1]];
        let expected7: PShape = vec![vec![1, 1], vec![0, 1], vec![0, 1]];
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
        let shape: PShape = vec![vec![1, 1], vec![1, 1]];
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
        let shape: PShape = vec![vec![1, 1], vec![1, 1]];
        let grid_dimensions = (1, 1);
        let transform = ShapeTransform::FullSymmetry;

        let possibilities =
            Polyomino::generate_all_possibilities(&[shape], grid_dimensions, transform);
        assert!(possibilities.is_empty()); // No possible placements in a 1x1
                                           // grid
    }

    #[test]
    fn test_generate_only_fitting_possibilities() {
        let shape: PShape = vec![vec![1, 0], vec![1, 0], vec![1, 1]];
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
            vec![vec![1, 1], vec![1, 0], vec![1, 0]], // J-shape
            vec![vec![0, 1], vec![1, 1], vec![1, 0]], // Z-shape
            vec![vec![0, 1], vec![0, 1], vec![1, 1]], // J-shape
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
        assert_eq!(possibilities, expected_possibilities);
    }

    #[test]
    fn test_generate_possibilities_pure_rotation() {
        let shapes = vec![
            vec![vec![1, 1], vec![1, 0], vec![1, 0]], // J-shape
            vec![vec![0, 1], vec![1, 1], vec![1, 0]], // Z-shape
            vec![vec![0, 1], vec![0, 1], vec![1, 1]], // J-shape
        ];
        let grid_dimensions = (3, 4);
        let transform = ShapeTransform::PureRotation;

        let possibilities =
            Polyomino::generate_all_possibilities(&shapes, grid_dimensions, transform);

        dbg!(possibilities.len());
        dbg!(&possibilities);

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
        assert_eq!(possibilities, expected_possibilities);
    }

    #[test]
    fn test_generate_possibilities_full_symmetry() {
        let shapes = vec![
            vec![vec![1, 1], vec![1, 0], vec![1, 0]], // J-shape
            vec![vec![0, 1], vec![1, 1], vec![1, 0]], // Z-shape
            vec![vec![0, 1], vec![0, 1], vec![1, 1]], // J-shape
        ];
        let grid_dimensions = (3, 4);
        let transform = ShapeTransform::FullSymmetry;

        let possibilities =
            Polyomino::generate_all_possibilities(&shapes, grid_dimensions, transform);

        dbg!(&possibilities);
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

        assert_eq!(possibilities, expected_possibilities);
    }

    #[test]
    fn test_new_polyomino_puzzle() {
        let shapes = vec![
            vec![vec![1, 1], vec![1, 0], vec![1, 0]], // J-shape
            vec![vec![0, 1], vec![1, 1], vec![1, 0]], // Z-shape
            vec![vec![0, 1], vec![0, 1], vec![1, 1]], // J-shape
        ];
        let grid_dimensions = (3, 4);
        let transform = ShapeTransform::NoTransform;
        let polyomino = Polyomino::new(grid_dimensions, shapes, transform);
        assert_eq!(polyomino.grid_dimensions, (3, 4));
        assert_eq!(
            polyomino.polyominoes,
            vec![
                vec![vec![1, 1], vec![1, 0], vec![1, 0]], // J-shape
                vec![vec![0, 1], vec![1, 1], vec![1, 0]], // Z-shape
                vec![vec![0, 1], vec![0, 1], vec![1, 1]], // J-shape
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
            vec![vec![1, 1], vec![1, 0], vec![1, 0]], // J-shape
            vec![vec![0, 1], vec![1, 1], vec![1, 0]], // Z-shape
            vec![vec![0, 1], vec![0, 1], vec![1, 1]], // J-shape
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
