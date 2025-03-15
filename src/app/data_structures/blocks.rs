use crate::app::math::linear_algebra::Vec2;
use crate::app::metrics::symmetry_type::SymmetryType;
use crate::app::sampling::SampleCombineMethod;
use itertools::Itertools;

/// Captures a bit matrix. The length of the vector should always be edge_length**2
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Blocks {
    pub blocks: Vec<bool>,
    pub grid_size: usize, // of type usize because we index with it a bunch of times
    // the total number of cells where there can be blocks is edge_length**2
    // Order is x first left to right, then y down to up (to match coord space)
    origin_float: Vec2, // In bitmatrix coordinates, where is the point (0,0)? (Note that it has integer coordinates)
    origin_usize: [usize; 2], //  same as origin_float but with usize coordinates
}

// longterm: build sequences
/// Getter methods
impl Blocks {
    pub fn new(blocks: Vec<bool>, grid_size: usize) -> Self {
        Blocks {
            blocks,
            grid_size,
            origin_float: Self::get_origin_float_from_grid_size(grid_size),
            origin_usize: [grid_size / 2, grid_size / 2],
        }
    }

    pub fn get_origin_float_from_grid_size(grid_size: usize) -> Vec2 {
        Vec2::from([(grid_size / 2) as f64, (grid_size / 2) as f64])
    }

    /// Get the index of the block whose left bottom corner coordinates are the global coord, if it
    ///  exists (else return none)
    pub fn get_index_from_global_coord_usize(&self, global_coord: [isize; 2]) -> Option<usize> {
        let local_coord = [
            global_coord[0] + (self.origin_usize[0] as isize),
            global_coord[1] + (self.origin_usize[1] as isize),
        ];

        if 0 <= local_coord[0]
            && 0 <= local_coord[1]
            && local_coord[0] < (self.grid_size as isize)
            && local_coord[1] < (self.grid_size as isize)
        {
            Some((local_coord[1] * (self.grid_size as isize) + local_coord[0]) as usize)
        } else {
            None
        }
    }

    pub fn get_global_coord_usize_from_index(&self, i: usize) -> [isize; 2] {
        [
            (i % self.grid_size) as isize - (self.origin_usize[0] as isize),
            (i / self.grid_size) as isize - (self.origin_usize[1] as isize),
        ]
    }

    /// return true if there is a block with left bottom coordinate the input global coordinate
    /// if the point is outside the grid or there is no block there, return false
    pub fn is_block_on_global_coord(&self, global_coord: [isize; 2]) -> bool {
        match self.get_index_from_global_coord_usize(global_coord) {
            None => false,
            Some(index) => self.blocks[index],
        }
    }

    /// Centered block coordinates, i.e., where the origin lies at (0,0)
    /// Gives the left bottom coordinates of each block
    pub fn get_all_block_coords(&self) -> Vec<[f64; 2]> {
        let mut i = 0;
        let mut output_vec = Vec::new();

        for b in &self.blocks {
            if *b {
                output_vec.push([
                    ((i % self.grid_size) as f64) - self.origin_float.x, // Get integer x position (which is bot. left), then make origin center
                    ((i / self.grid_size) as f64) - self.origin_float.y,
                ]);
            }
            i += 1;
        }

        output_vec
    }

    /// Global centered block coordinates in usize, i.e., where the origin lies at (0,0)
    pub fn get_all_block_coords_usize(&self) -> Vec<[isize; 2]> {
        let mut i = 0;
        let mut output_vec = Vec::new();

        for b in &self.blocks {
            if *b {
                output_vec.push([
                    (i % self.grid_size) as isize - (self.origin_usize[0] as isize),
                    (i / self.grid_size) as isize - (self.origin_usize[1] as isize),
                ]);
            }
            i += 1;
        }

        output_vec
    }
}

/// Methods for getting basic metrics
impl Blocks {
    /// Get number of blocks
    pub fn get_nr_blocks(&self) -> u64 {
        (*self.blocks).into_iter().filter(|b| **b).count() as u64
    }

    /// Get (thin-walled) interior, i.e., blocks in self which have no neighbors which share a side
    /// with an air block
    pub fn get_interior(&self) -> Blocks {
        Blocks::new(
            (0..self.grid_size.pow(2))
                .map(|i| {
                    self.blocks[i] == true
                        // has to be a block in self
                        && i % self.grid_size != 0
                        && i % self.grid_size != self.grid_size - 1
                        && i / self.grid_size != 0
                        && i / self.grid_size != self.grid_size - 1
                        // cannot lie on the boundary of the grid
                        && self.blocks[i + 1]
                        && self.blocks[i - 1]
                        && self.blocks[i + self.grid_size]
                        && self.blocks[i - self.grid_size]
                    // all direct neighbors should also be blocks
                })
                .collect(),
            self.grid_size,
        )
    }

    /// Get (thin-walled) boundary
    /// (There is the obvious relation of boundary + interior = blocks, but we won't use it)
    pub fn get_boundary(&self) -> Blocks {
        Blocks::new(
            (0..self.grid_size.pow(2))
                .map(|i| {
                    self.blocks[i] == true
                        // has to be a block
                        // and (be on the grid boundary or border any non-block)
                    && (
                        i % self.grid_size == 0
                            || i % self.grid_size == self.grid_size - 1
                            || i / self.grid_size == 0
                            || i / self.grid_size == self.grid_size - 1
                            || !self.blocks[i + 1]
                            || !self.blocks[i - 1]
                            || !self.blocks[i + self.grid_size]
                            || !self.blocks[i - self.grid_size]
                        )
                })
                .collect(),
            self.grid_size,
        )
    }

    /// Complement of blocks (for building interiors)
    pub fn get_complement(&self) -> Blocks {
        Blocks::new(
            (0..self.grid_size.pow(2))
                .map(|i| !self.blocks[i])
                .collect(),
            self.grid_size,
        )
    }

    // Returns a (unordered) list of all corners of the blocks (so get_block_coords (±0.5,±0.5))
    // which are the corner of exactly 1 block (note that exactly 2 blocks corresponds to a corner
    //  which is an edge of the whole block structure, and exactly 2 blocks corresponds to an inner corner)
    // The convex hull only depends on points of this type (so it should be a large reduction).
    pub fn get_outer_corners(&self) -> Vec<[f64; 2]> {
        // loop over all 2x2s in the grid, which we think of as points between the blocks
        // then count the number of blocks

        let mut output = vec![];

        // For an n×n grid there are (n-1)×(n-1) points between 4 points... (vaguely speaking)
        for corner_point in 0..self.grid_size.pow(2) {
            // Filter out those corner points which don't have a 2×2 around them
            // (using bottom right index to match corners to their block)
            if corner_point / self.grid_size == self.grid_size - 1 || // if on left edge
                corner_point % self.grid_size == self.grid_size - 1
            // if on top edge
            {
                // do nothing
            } else if (self.blocks[corner_point] as u8)
                + (self.blocks[corner_point + 1] as u8)
                + (self.blocks[corner_point + self.grid_size] as u8)
                + (self.blocks[corner_point + self.grid_size + 1] as u8)
                == 1
            {
                // add to output
                output.push([
                    ((corner_point % self.grid_size) as f64) - self.origin_float.x + 1.0,
                    ((corner_point / self.grid_size) as f64) - self.origin_float.y + 1.0,
                ]);
            }
        }

        output
    }
}

/// Methods for getting bounds
impl Blocks {
    /// Get the smallest box that contains all blocks in the input.
    /// The output is in coordinates relative to the grid system (so usizes)
    /// Presented in the format ((x_1, y_1), (x_2, y_2)), where
    /// 0 <= x_1 <= x_2 < grid_size,   0 <= y_1 <= y_2 < grid_size
    pub fn get_bounds(&self) -> [[isize; 2]; 2] {
        let mut x_1 = self.grid_size;
        let mut y_1 = self.grid_size;
        let mut x_2 = 0;
        let mut y_2 = 0;

        for (x, y) in (0..self.grid_size).cartesian_product(0..self.grid_size) {
            if self.blocks[x + y * self.grid_size] {
                x_1 = x_1.min(x);
                y_1 = y_1.min(y);
                x_2 = x_2.max(x);
                y_2 = y_2.max(y);
            }
        }

        // handle empty inputs as the center being at the bottom left (a single block)
        if self.get_nr_blocks() == 0 {
            x_1 = 0;
            y_1 = 0;
        }

        [
            [
                x_1 as isize - self.origin_usize[0] as isize,
                y_1 as isize - self.origin_usize[1] as isize,
            ],
            [
                x_2 as isize - self.origin_usize[0] as isize,
                y_2 as isize - self.origin_usize[1] as isize,
            ],
        ]
    }

    pub fn get_bounds_floats(&self) -> [[f64; 2]; 2] {
        let [[x_1, y_1], [x_2, y_2]] = self.get_bounds();

        [
            [x_1 as f64, y_1 as f64],
            [x_2 as f64 + 1.0, y_2 as f64 + 1.0],
        ]
    }

    /// Get the diameters of the bounding box of the input
    /// Need to get the x and y diameters separately by asymetry.
    /// Diameters are preferred over radii since we always get an integer this way
    pub fn get_diameters(&self) -> [usize; 2] {
        let [[x_1, y_1], [x_2, y_2]] = self.get_bounds();
        [(x_2 - x_1 + 1) as usize, (y_2 - y_1 + 1) as usize]
    }

    // Return the blocks object which contains the center 1, 2, or 4 blocks (depending on parities)
    pub fn get_center_blocks(&self) -> Blocks {
        let [[x_1, y_1], [_, _]] = self.get_bounds();
        let [diameter_x, diameter_y] = self.get_diameters();

        // get indices, 1 or 2 depending on parity
        let x_indices = if diameter_x % 2 == 0 {
            vec![
                (self.origin_usize[0] as isize + x_1 + diameter_x as isize / 2).saturating_sub(1)
                    as usize,
                (self.origin_usize[0] as isize + x_1 + diameter_x as isize / 2) as usize,
            ]
        } else {
            vec![(self.origin_usize[0] as isize + x_1 + diameter_x as isize / 2) as usize]
        };

        let y_indices = if diameter_y % 2 == 0 {
            vec![
                (self.origin_usize[1] as isize + y_1 + diameter_y as isize / 2).saturating_sub(1)
                    as usize,
                (self.origin_usize[1] as isize + y_1 + diameter_y as isize / 2) as usize,
            ]
        } else {
            vec![(self.origin_usize[1] as isize + y_1 + diameter_y as isize / 2) as usize]
        };

        Self::squares_at_xy_coords(x_indices, y_indices, self.grid_size)
    }

    // Get the coordinates of the center of the bounding box
    pub fn get_center_coord(&self) -> [f64; 2] {
        let center_blocks = self.get_center_blocks();
        let nr_center_blocks = center_blocks.get_nr_blocks();
        // Take the average of the left bottom coordinates, then add 0.5 in both coords to get the center
        center_blocks
            .get_all_block_coords()
            .iter()
            .fold([0.0; 2], |[a, b], [c, d]| [a + c, b + d])
            .map(|coord| coord / (nr_center_blocks as f64) + 0.5)
    }
}

/// Methods for combining different Blocks
impl Blocks {
    /// A block is in the output iff there is a block at the same global position for any layer in
    ///  the input.
    fn combine_any(stack: Vec<Self>) -> Self {
        // determine largest grid size
        let grid_size = stack.iter().map(|b| b.grid_size).max().unwrap();
        // throws an error only if the vector above is empty
        let origin_usize = [grid_size / 2, grid_size / 2];

        Blocks::new(
            (0..grid_size.pow(2))
                .map(|i| {
                    let global_coord = [
                        (i % grid_size) as isize - (origin_usize[0] as isize),
                        (i / grid_size) as isize - (origin_usize[1] as isize),
                    ];

                    stack
                        .iter()
                        .map(|b| b.is_block_on_global_coord(global_coord))
                        .fold(false, |a, b| a || b)
                })
                .collect(),
            grid_size,
        )
    }

    /// A block is in the output iff for every layer in the input, there is a block at the same
    ///  global position
    fn combine_all(stack: Vec<Self>) -> Self {
        // determine largest grid size
        let grid_size = stack.iter().map(|b| b.grid_size).max().unwrap();
        // throws an error only if the vector above is empty
        let origin_usize = [grid_size / 2, grid_size / 2];

        Blocks::new(
            (0..grid_size.pow(2))
                .map(|i| {
                    let global_coord = [
                        (i % grid_size) as isize - (origin_usize[0] as isize),
                        (i / grid_size) as isize - (origin_usize[1] as isize),
                    ];

                    stack
                        .iter()
                        .map(|b| b.is_block_on_global_coord(global_coord))
                        .fold(true, |a, b| a && b)
                })
                .collect(),
            grid_size,
        )
    }

    /// A block is in the output iff there is a block at the same global position for more than the
    ///  given percentage of layers
    fn combine_percentage(stack: Vec<Self>, percentage: f64) -> Self {
        // determine the largest grid size & associated origin
        // throws an error only if the vector above is empty
        let grid_size = stack.iter().map(|b| b.grid_size).max().unwrap();
        let origin_usize = [grid_size / 2, grid_size / 2];
        // determine target number of layers (we specifically allow any f64 for percentage, but
        //  the output will be trivial for it not between zero and one).
        let target_nr_layers = stack.len() as f64 * percentage;

        Blocks::new(
            (0..grid_size.pow(2))
                .map(|i| {
                    let global_coord = [
                        (i % grid_size) as isize - (origin_usize[0] as isize),
                        (i / grid_size) as isize - (origin_usize[1] as isize),
                    ];

                    stack
                        .iter()
                        .map(|b| b.is_block_on_global_coord(global_coord))
                        .fold(0.0, |a, b| a + (b as usize) as f64)
                        >= target_nr_layers
                })
                .collect(),
            grid_size,
        )
    }

    pub fn combine(sample_combine_method: &SampleCombineMethod, stack: Vec<Self>) -> Self {
        match sample_combine_method {
            SampleCombineMethod::AllSamples => Self::combine_all(stack),
            SampleCombineMethod::AnySamples => Self::combine_any(stack),
            SampleCombineMethod::Percentage(percentage) => {
                Self::combine_percentage(stack, *percentage)
            }
        }
    }
}

/// Methods for modifying blocks (flipping and rotating)
impl Blocks {
    /// Flip the blocks along the vertical axis through the center of the bounds.
    fn flip_horizontal(&self, bounds: [[isize; 2]; 2]) -> Self {
        let [[_, y_1], [_, y_2]] = bounds;

        Blocks::new(
            (0..self.grid_size.pow(2))
                .map(|index| {
                    let [x, y] = self.get_global_coord_usize_from_index(index);
                    // formula for mirroring in the middle of y_1 and y_2
                    let image_global_coord = [x, -y + y_1 + y_2];
                    self.is_block_on_global_coord(image_global_coord)
                })
                .collect(),
            self.grid_size,
        )
    }

    /// Flip the blocks along the horizontal axis through the center of the bounds.
    fn flip_vertical(&self, bounds: [[isize; 2]; 2]) -> Self {
        let [[x_1, _], [x_2, _]] = bounds;

        Blocks::new(
            (0..self.grid_size.pow(2))
                .map(|index| {
                    let [x, y] = self.get_global_coord_usize_from_index(index);
                    // formula for mirroring in the middle of x_1 and x_2
                    let preimage_global_coord = [-x + x_1 + x_2, y];
                    self.is_block_on_global_coord(preimage_global_coord)
                })
                .collect(),
            self.grid_size,
        )
    }

    /// Flip the blocks along the 45° up diagonal through the center.
    /// Requires that the center is either 1 or 4, so that this diagonal goes nicely through the grid
    /// (Else reflection doesn't take cells to cells)
    fn flip_up_diagonal(&self) -> Self {
        let center_blocks = self.get_center_blocks();
        assert!(center_blocks.get_nr_blocks() == 1 || center_blocks.get_nr_blocks() == 4);
        let [x_m, y_m] = center_blocks.get_bounds()[0]; // Point through which the mirror goes

        Blocks::new(
            (0..self.grid_size.pow(2))
                .map(|index| {
                    let [x, y] = self.get_global_coord_usize_from_index(index);
                    // formula for mirroring in the up diagonal through [x_m, y_m]
                    let image_global_coord = [y + x_m - y_m, x - x_m + y_m];
                    self.is_block_on_global_coord(image_global_coord)
                })
                .collect(),
            self.grid_size,
        )
    }

    /// Flip the blocks along the 45° down diagonal through the center.
    /// Requires that the center is either 1 or 4, so that this diagonal goes nicely through the grid
    /// (Else reflection doesn't take cells to cells)
    fn flip_down_diagonal(&self) -> Self {
        let center_blocks = self.get_center_blocks();
        assert!(center_blocks.get_nr_blocks() == 1 || center_blocks.get_nr_blocks() == 4);
        let [x_lb, y_lb] = center_blocks.get_bounds()[0]; // left bottom corner
                                                          // Point through which the mirror goes
        let [x_m, y_m] = if center_blocks.get_nr_blocks() == 1 {
            [x_lb + 1, y_lb] // go right one from bottom left
        } else {
            [x_lb + 2, y_lb] // go right two from bottom left
        };

        Blocks::new(
            (0..self.grid_size.pow(2))
                .map(|index| {
                    let [x, y] = self.get_global_coord_usize_from_index(index);
                    // formula for mirroring in the up diagonal through [x_m, y_m]
                    let image_global_coord = [-y + x_m + y_m - 1, -x + x_m + y_m - 1];
                    // NOTE! We flip the left bottom corner! so to look up the preimage, we need to
                    //  account for looking at the right top corner, hence the -1, -1 term
                    self.is_block_on_global_coord(image_global_coord)
                })
                .collect(),
            self.grid_size,
        )
    }
}

/// Methods for symmetry detection
impl Blocks {
    /// Get the symmetry type of the block structure
    pub fn get_symmetry_type(&self) -> SymmetryType {
        let bounds = self.get_bounds();
        let diagonals_possible = self.get_center_blocks().get_nr_blocks() == 1
            || self.get_center_blocks().get_nr_blocks() == 4;

        match (
            *self == self.flip_horizontal(bounds),
            *self == self.flip_vertical(bounds),
            diagonals_possible && *self == self.flip_up_diagonal(), // rely on short-circuiting here to prevent asserting errors
            diagonals_possible && *self == self.flip_down_diagonal(),
        ) {
            // If any are true, we are in the reflection case
            (true, _, true, _) | (true, _, _, true) | (_, true, true, _) | (_, true, _, true) => {
                SymmetryType::ReflectionsAll
            }
            (true, true, false, false) => SymmetryType::ReflectionsCardinals,
            (false, false, true, true) => SymmetryType::ReflectionsDiagonals,
            (true, false, false, false) => SymmetryType::ReflectionHorizontal,
            (false, true, false, false) => SymmetryType::ReflectionVertical,
            (false, false, true, false) => SymmetryType::ReflectionDiagonalUp,
            (false, false, false, true) => SymmetryType::ReflectionDiagonalDown,
            (false, false, false, false) => {
                // Rotation case
                // Check 90° rotation symmetry
                if diagonals_possible && *self == self.flip_horizontal(bounds).flip_up_diagonal() {
                    SymmetryType::RotationQuarter
                } else if *self == self.flip_horizontal(bounds).flip_vertical(bounds) {
                    SymmetryType::RotationHalf
                } else {
                    SymmetryType::NoSymmetry
                }
            }
        }
    }
}

/// Methods for construction
impl Blocks {
    // All blocks that have x coord in the set
    fn strips_at_x_coords(x_coords: Vec<usize>, grid_size: usize) -> Blocks {
        Blocks::new(
            (0..grid_size.pow(2))
                .map(|i| x_coords.contains(&(i % grid_size)))
                .collect(),
            grid_size,
        )
    }

    // All blocks that have y coord in the set
    fn strips_at_y_coords(y_coords: Vec<usize>, grid_size: usize) -> Blocks {
        Blocks::new(
            (0..grid_size.pow(2))
                .map(|i| y_coords.contains(&(i / grid_size)))
                .collect(),
            grid_size,
        )
    }

    /// Blocks that have x and y coords contained in the provided vectors
    fn squares_at_xy_coords(
        x_coords: Vec<usize>,
        y_coords: Vec<usize>,
        grid_size: usize,
    ) -> Blocks {
        Blocks::new(
            (0..grid_size.pow(2))
                .map(|i| x_coords.contains(&(i % grid_size)) && y_coords.contains(&(i / grid_size)))
                .collect(),
            grid_size,
        )
    }
}
