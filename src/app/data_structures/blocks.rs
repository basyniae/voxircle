use crate::app::math::linear_algebra::Vec2;
use crate::app::sampling::SampleCombineMethod;
use itertools::Itertools;

/// Captures a bit matrix. The length of the vector should always be edge_length**2
#[derive(Default, Debug, Clone)]
pub struct Blocks {
    pub blocks: Vec<bool>,
    pub grid_size: usize, // of type usize because we index with it a bunch of times
    // the total number of cells where there can be blocks is edge_length**2
    // Order is x first left to right, then y down to up (to match coord space)
    origin_float: Vec2, // In bitmatrix coordinates, where is the point (0,0)? (Note that it has integer coordinates)
    origin_usize: [usize; 2], //  same as origin_float but with usize coordinates
}

// TODO: make intersect and complement methods for easier computation
// longterm: Symmetry type detection, then build sequences should be within reach
// TODO: make everything use is_block_on_global_coord
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
    // fixme: how to handle empty inputs?
    // todo: think about if this would be better make as global coords (pairs of isizes).
    /// Get the smallest box that contains all blocks in the input.
    /// The output is in coordinates relative to the grid system (so usizes)
    /// Presented in the format ((x_1, y_1), (x_2, y_2)), where
    /// 0 <= x_1 <= x_2 < grid_size,   0 <= y_1 <= y_2 < grid_size
    pub fn get_bounds(&self) -> [[usize; 2]; 2] {
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

        [[x_1, y_1], [x_2, y_2]]
    }

    pub fn get_bounds_floats(&self) -> [[f64; 2]; 2] {
        let [[x_1, y_1], [x_2, y_2]] = self.get_bounds();
        let [x_1, y_1] = self.get_global_coord_usize_from_index(x_1 + y_1 * self.grid_size);
        let [x_2, y_2] = self.get_global_coord_usize_from_index(x_2 + y_2 * self.grid_size);

        [
            [x_1 as f64, y_1 as f64],
            [x_2 as f64 + 1.0, y_2 as f64 + 1.0],
        ]
    }

    /// Get the diameters of the bounding box of the input
    /// Need to get the x and y diameters separately by assymetry.
    /// Diameters are preferred over radii since we always get an integer this way
    pub fn get_diameters(&self) -> [usize; 2] {
        let [[x_1, y_1], [x_2, y_2]] = self.get_bounds();
        [x_2 - x_1 + 1, y_2 - y_1 + 1]
    }

    // Return the blocks object which contains the center 1, 2, or 4 blocks (depending on parities)
    pub fn get_center_blocks(&self) -> Blocks {
        let [[x_1, y_1], [x_2, y_2]] = self.get_bounds();
        let [diameter_x, diameter_y] = self.get_diameters();

        // get indices, 1 or 2 depending on parity
        let x_indices = if diameter_x % 2 == 0 {
            vec![x_1 + diameter_x / 2 - 1, x_1 + diameter_x / 2]
        } else {
            vec![x_1 + diameter_x / 2]
        };

        let y_indices = if diameter_y % 2 == 0 {
            vec![y_1 + diameter_y / 2 - 1, y_1 + diameter_y / 2]
        } else {
            vec![y_1 + diameter_y / 2]
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

    pub fn combine(sample_combine_method: SampleCombineMethod, stack: Vec<Self>) -> Self {
        match sample_combine_method {
            SampleCombineMethod::AllSamples => Self::combine_all(stack),
            SampleCombineMethod::AnySamples => Self::combine_any(stack),
            SampleCombineMethod::Percentage(percentage) => {
                Self::combine_percentage(stack, percentage)
            }
        }
    }
}

/// Methods for modifying blocks
impl Blocks {
    fn flip_horizontal(&self, bounds: [[usize; 2]; 2]) -> Self {
        todo!()
    }
}

/// Methods for symmetry detection & building helpers TODO
impl Blocks {
    // From a shape (which is assumed to be non-pathological,
    // find the sequence of side lengths.
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

    // Blocks that have x and y coords contained in the provided vectors
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
