use crate::app::helpers::linear_algebra::{Mat2, Vec2};
use crate::app::helpers::square::Square;
use std::ops::Not;

/// Captures a bit matrix. The length of the vector should always be edge_length**2

#[derive(Default, Clone)]
pub struct Blocks {
    pub blocks: Vec<bool>,
    pub grid_size: usize, // of type usize because we index with it a bunch of times
    // the total number of cells where there can be blocks is edge_length**2
    // Order is x first left to right, then y down to up (to match coord space)
    pub origin: Vec2, // At which bitmatrix coords is the center of the circle?
                      // For a block-diameter 4 circle it would be [2.0,2.0], for a block-diameter 5 circle it would be [2.5,2.5]
                      // but how does this work paradigmatically
}

impl Blocks {
    /// Centered block coordinates, i.e., where the origin lies at (0,0)
    pub fn get_block_coords(&self) -> Vec<[f64; 2]> {
        let mut i = 0;
        let mut output_vec = Vec::new();

        for b in &self.blocks {
            if *b {
                output_vec.push([
                    ((i % self.grid_size) as f64) - self.origin.x, // Get integer x position (which is bot. left), then make origin center
                    ((i / self.grid_size) as f64) - self.origin.y,
                ]);
            }
            i += 1;
        }

        output_vec
    }

    /// Get number of blocks
    pub fn get_nr_blocks(&self) -> u64 {
        (*self.blocks).into_iter().filter(|b| **b).count() as u64
    }

    /// Get (thin-walled) interior, i.e., blocks in self which have no neighbors which share a side
    /// with an air block
    pub fn get_interior(&self) -> Blocks {
        // let mut output_vec = Vec::new();

        let blocks = (0..self.grid_size.pow(2))
            .map(|i| {
                self.blocks[i] == true
                    // has to be a block in self
                    && i % self.grid_size != 0
                    && i % self.grid_size != self.grid_size - 1
                    && i / self.grid_size != 0
                    && i / self.grid_size != self.grid_size - 1
                    // cannot lie on the boundary of the grid
                    && self.blocks[i + 1] == true
                    && self.blocks[i - 1] == true
                    && self.blocks[i + self.grid_size] == true
                    && self.blocks[i - self.grid_size] == true
                // all direct neighbors should also be blocks
            })
            .collect();

        Blocks { blocks, ..*self }
    }

    /// Complement of blocks (for interiors)
    pub fn get_complement(&self) -> Blocks {
        let blocks = (0..self.grid_size.pow(2))
            .map(|i| self.blocks[i].not())
            .collect();
        Blocks { blocks, ..*self }
    }

    // From a shape (which is assumed to be non-pathological, TODO: make checks for this)
    // find the sequence of side lengths.
    // Goal: easy construction in minecraft (it's how i think of circles)
    pub fn get_build_sequence(&self) -> Vec<usize> {
        // We scan horizontal lines bottom to top (which is just repeated addition of the edge length)
        // Then subtract the nr blocks in each successive layer and divide by two

        let mut nr_blocks_per_layer = vec![];

        // loop over rows
        'row: for i in 0..self.grid_size {
            let mut nr_blocks_in_ith_layer: usize = 0;
            let mut prev_block_had_block_below = false; // detect if we're accross the symmetry axis
            let mut prev_location_had_block = false; // detect if all blocks on a row have been scanned
            let mut last_row = false; // detect if this is the last row we need to include in the build list

            // loop over elements of rows
            for j in 0..self.grid_size {
                // If there is a block at the current position
                if self.blocks[i * self.grid_size + j] {
                    prev_location_had_block = true;
                    // And there is no block directly below it (note that padding takes care of positivity)
                    if !self.blocks[(i - 1) * self.grid_size + j] {
                        if prev_block_had_block_below {
                            // If the previous block had a block below it and this one doesn't,
                            //  we must be past the symmetry axis. So no contribution and stop
                            //  scanning this line.
                            nr_blocks_per_layer.push(nr_blocks_in_ith_layer);
                            if last_row {
                                break 'row;
                            }
                            break;
                        }
                        // Then this block contributes to the build sequence
                        nr_blocks_in_ith_layer += 1;
                    } else {
                        prev_block_had_block_below = true;
                    }

                    // If there is no block left-diagonally above the current block
                    //  we must be past the 45° point. (this assumes "niceness" of the input)
                    if !self.blocks[(i + 1) * self.grid_size + j - 1] {
                        // so break the outer loop as soon as we've scanned the full row
                        // (this to avoid error in case of the final in build sequence being >1 in length)
                        last_row = true;
                    }
                } else {
                    // Stop scanning the line at the end of the line
                    if prev_location_had_block {
                        nr_blocks_per_layer.push(nr_blocks_in_ith_layer);
                        if last_row {
                            break 'row;
                        }
                        break;
                    }
                }
            }
        }

        nr_blocks_per_layer
    }

    // Need to get the x and y diameters separately by assymetry.
    // Also diameters are preferred over radii since we always get an integer this way
    pub fn get_diameters(&self) -> [u64; 2] {
        // x diameter: loop over all columns, count how many have at least one block
        let mut x_diameter = 0;
        for i in 0..self.grid_size {
            for j in 0..self.grid_size {
                if self.blocks[i + j * self.grid_size] {
                    x_diameter += 1;
                    break;
                }
            }
        }

        // y diameter: loop over all rows, count how many have at least one block
        let mut y_diameter = 0;
        for i in 0..self.grid_size {
            for j in 0..self.grid_size {
                if self.blocks[i * self.grid_size + j] {
                    y_diameter += 1;
                    break;
                }
            }
        }

        [x_diameter, y_diameter]
    }

    // Returns a (unordered) list of all corners of the blocks (so get_block_coords (±0.5,±0.5))
    // which are the corner of exactly 1 block (note that exactly 2 blocks corresponds to a corner
    //  which is an edge of the whole block structure, and exactly 2 blocks correponds to an inner corner)
    // The convex hull only depends on points of this type (so it should be a large reduction).
    pub fn get_outer_corners(&self) -> Vec<[f64; 2]> {
        // loop over all 2x2s in the grid, which we think of as points between the blocks
        // then count the number of blocks

        let mut output = vec![];

        // For an n×n grid there are (n-1)×(n-1) points inbetween 4 points... (vaguely speaking)
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
                    ((corner_point % self.grid_size) as f64) - self.origin.x + 1.0,
                    ((corner_point / self.grid_size) as f64) - self.origin.y + 1.0,
                ]);
            }
        }

        output
    }

    pub fn get_strict_bounds(&self) -> [[f64; 2]; 2] {
        let mut min_x = 0.0_f64;
        let mut max_x = 0.0_f64;
        let mut min_y = 0.0_f64;
        let mut max_y = 0.0_f64;

        for i in 0..self.grid_size.pow(2) {
            let square = Square::new(
                i,
                self.grid_size,
                Vec2::from([0.0, 0.0]),
                self.origin,
                Mat2::from([0.0, 0.0, 0.0, 0.0]),
            );

            if self.blocks[i] {
                if square.lb.x <= min_x {
                    min_x = square.lb.x
                }
                if square.lb.y <= min_y {
                    min_y = square.lb.y
                }
                if square.rt.x >= max_x {
                    max_x = square.rt.x
                }
                if square.lb.y <= max_y {
                    max_y = square.rt.y
                }
            }
        }

        [[min_x, min_y], [max_x, max_y]]
    }

    pub fn get_padded_bounds(&self, pad: f64) -> [[f64; 2]; 2] {
        let strict_bounds = self.get_strict_bounds();
        [
            [strict_bounds[0][0] - pad, strict_bounds[0][1] - pad],
            [strict_bounds[1][0] + pad, strict_bounds[1][1] + pad],
        ]
    }
}
