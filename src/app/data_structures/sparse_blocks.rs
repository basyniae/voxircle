use crate::app::data_structures::blocks::Blocks;
use egui::Color32;
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Sparse representation of a blocks object (that forgets about the origin)
/// `indices` contains x iff there is a block at index x
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparseBlocks {
    pub indices: HashSet<[isize; 2]>, //todo: make private
}

// todo:
//  routine function which uses this to create a building thing...
//  (but must not forget placement on grid!)

// todo:
//  assign the same color consistently to similarly shaped blocks (translated & flipped & rotated versions)
//  first do only translates.

impl From<Blocks> for SparseBlocks {
    fn from(blocks: Blocks) -> Self {
        SparseBlocks {
            indices: HashSet::from_iter(blocks.get_all_block_coords_usize().into_iter()),
        }
    }
}

impl SparseBlocks {
    pub fn connected_components(&self) -> Vec<Self> {
        let mut a = self.indices.clone();
        let mut running_conn_components = Vec::new();

        // idea: remove blocks from a until it is empty. remove all connected blocks
        while !a.is_empty() {
            // a is not empty, select the first element. we will find the connected component of
            // this first element
            let first_random_index = *a.iter().next().unwrap();
            a.remove(&first_random_index);
            let mut running_component: HashSet<[isize; 2]> =
                vec![first_random_index].into_iter().collect();
            let mut moving_boundary = vec![first_random_index];

            // now iterate over the boundary neighbors of the moving_boundary, removing them from a and
            // adding them to this component
            while !moving_boundary.is_empty() {
                // check if the neighbors of [x,y] are in a, and if so add them to the running_component
                //  and the boundary
                let [x, y] = moving_boundary.pop().unwrap();
                for neighbor in [[x, y + 1], [x, y - 1], [x + 1, y], [x - 1, y]].iter() {
                    if a.contains(neighbor) {
                        // neighbor is in the same connected component. (it is a neighbor of the boundary)
                        a.remove(neighbor);
                        running_component.insert(neighbor.clone());
                        moving_boundary.push(neighbor.clone());
                    }
                }
            }

            running_conn_components.push(SparseBlocks {
                indices: running_component,
            })
        }

        running_conn_components
    }

    /// Get the maximal and minimal coordinates of the blocks (for pattern matching) in the form
    ///  [[min_x, min_y], [max_x, max_y]] (same as in the rest of the program)
    fn get_bounds(&self) -> [[isize; 2]; 2] {
        /// Return largest square (specified by min and max vector) (bottom left and top right)
        fn include_in_square(a: [[isize; 2]; 2], b: &[isize; 2]) -> [[isize; 2]; 2] {
            let low_x = a[0][0].min(b[0]);
            let high_x = a[1][0].max(b[0]);
            let low_y = a[0][1].min(b[1]);
            let high_y = a[1][1].max(b[1]);

            [[low_x, low_y], [high_x, high_y]]
        }
        self.indices.iter().fold(
            [[isize::MAX, isize::MAX], [isize::MIN, isize::MIN]],
            include_in_square,
        )
    }

    fn get_dimensions(&self) -> [usize; 2] {
        let bounds = self.get_bounds();
        [
            (bounds[1][0] - bounds[0][0]) as usize,
            (bounds[1][1] - bounds[0][1]) as usize,
        ]
    }

    // this is not the right way to go about the coloring process. rather put every shape in a canonical form that we can color/hash
    // /// Is self a translate of other? I.e., is there some offset of self which transforms it into other?
    // pub fn is_translate_of(&self, other: &Self) -> bool {
    //     // use bounds of the self and other to get a range for the indices to loop over
    //     // short circuit if self and other contain different amounts of blocks
    //     if self.indices.len() != other.indices.len() {
    //         return false;
    //     }
    //
    //     let [self_bounds_min, self_bounds_max] = self.get_bounds();
    //     let [other_bounds_min, other_bounds_max] = other.get_bounds();
    //
    //     // short circuit if the self and other have different profile sizes
    //     if self.get_dimensions() != other.get_dimensions() {
    //         return false;
    //     }
    //
    //     // we can calculate the necessary offset exactly from the bounds.
    //     let [x_offset, y_offset] = [
    //         other_bounds_min[0] - self_bounds_min[0],
    //         other_bounds_min[1] - self_bounds_min[1],
    //     ];
    //     // do a loop over the indices in self, add the offset, then check if they're all in the other.
    //     //  (this works since both sets have the same size.) short-circuiting by the use of .all()
    //     self.indices
    //         .iter()
    //         .all(|[x, y]| !other.indices.contains(&[x + x_offset, y + y_offset]))
    // }

    // todo: try to influence the colors for the small dimension like [1,1], [2,1], etc., as they're
    //  very common and influence the look a lot
    // todo: give the colors some theming...
    // todo: it is possible but not common for two shapes to have the same rotated dimensions but
    //  not being a translation/rotation/reflection of eachother, but then they still get the same
    //  color from this function.
    /// Get color from the rotated dimension of a shape (by a hash function)
    pub fn hash_color(&self) -> Color32 {
        let mut hasher = DefaultHasher::new();
        let mut first = self.get_dimensions();
        // rotate the dimensions so the long size is the first coordinate
        if first[0] < first[1] {
            first = [first[1], first[0]]
        }
        first.hash(&mut hasher);
        let mut int = hasher.finish();
        // get first 3 sets of 8 bits as rgb
        let r = (int & 255) as u8;
        let g = ((int & (255 * 256)) / 256) as u8;
        let b = ((int & (255 * 256 * 256)) / (256 * 256)) as u8;
        Color32::from_rgb(r, g, b)
    }
}
