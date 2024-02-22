// Captures a bit matrix. The length of the vector should always be edge_length**2
// Order is x first left to right, then y down to up (to match coord space)
#[derive(Default, Clone)]
pub struct Blocks {
    pub blocks: Vec<bool>,
    pub edge_length: usize, // of type usize because we index with it a bunch of times
    pub origin: [f64; 2] // At which bitmatrix coords is the center of the circle? 
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
                    (i % self.edge_length) as f64 - self.origin[0], // Get integer x position (which is bot. left), then make origin center
                    (i / self.edge_length) as f64 - self.origin[1]
                    ])
            }
            i += 1;
        }
        
        output_vec
    }

    pub fn get_nr_blocks(&self) -> u64 {
        let mut running_total = 0;
        for b in &self.blocks {
            if *b {
                running_total += 1;
            }
        }

        running_total
    }

    pub fn get_interior(&self) -> Blocks {
        let mut i = 0usize;
        let mut output_vec = Vec::new();

        for b in &self.blocks {
            
            if *b == false {
                // if not a point in the set of blocks, then certainly not a point in the interior
                output_vec.push(false);
            } else if (i % self.edge_length == 0) 
            || (i % self.edge_length == self.edge_length-1
            || (i / self.edge_length == 0)
            || (i / self.edge_length == self.edge_length-1)) {
                // If on the boundary, output not-interior (valid via judicious padding of the interesting structure)
                output_vec.push(false);
            } else if (self.blocks[i+1] == false)
            || (self.blocks[i-1] == false)
            || (self.blocks[i+self.edge_length] == false)
            || (self.blocks[i-self.edge_length] == false) { // actually check the neighbors
                output_vec.push(false);
            } else {
                output_vec.push(true);
            }

            i += 1;
        }

        Blocks {
            blocks: output_vec,
            edge_length: self.edge_length,
            origin: self.origin,
        }
    }
}