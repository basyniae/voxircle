use crate::app::data_structures::blocks::Blocks;
use egui::Color32;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem::swap;

/// Sparse representation of a blocks object (that forgets about the origin)
/// `indices` contains x iff there is a block at index x
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparseBlocks {
    indices: HashSet<[isize; 2]>, //todo: make private
}

// todo:
//  routine function which uses this to create a building thing...
//  (but must not forget placement on grid!)

impl From<Blocks> for SparseBlocks {
    fn from(blocks: Blocks) -> Self {
        SparseBlocks {
            indices: HashSet::from_iter(blocks.get_all_block_coords_usize().into_iter()),
        }
    }
}

impl From<&Blocks> for SparseBlocks {
    fn from(blocks: &Blocks) -> Self {
        SparseBlocks {
            indices: HashSet::from_iter(blocks.get_all_block_coords_usize().into_iter()),
        }
    }
}

impl Hash for SparseBlocks {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // sort coordinates into a vector and then hash
        let mut indices: Vec<&[isize; 2]> = self.indices.iter().collect();
        indices.sort_by(|a, b| {
            if a[0] != b[0] {
                a[0].partial_cmp(&b[0]).unwrap()
            } else {
                a[1].partial_cmp(&b[1]).unwrap()
            }
        });
        indices.hash(state)
    }
}

/// For build sequence
impl Display for SparseBlocks {
    /// The length of the segment if it is a segment, otherwise a random (from hash) letter
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dim = self.get_rotated_dimensions();
        match dim {
            [length, 1] => write!(f, "{length}"),
            dim => {
                let mut hasher = DefaultHasher::new();
                dim.hash(&mut hasher);
                let int = hasher.finish() % 26 + 65; // mod out the alphabet length, add offset 65 to land in uppercase range.
                println!("{dim:?} hashes to {int}");
                let chr: char = char::from_u32(int as u32).unwrap();
                write!(f, "{chr}",)
            }
        }
    }
}

impl SparseBlocks {
    pub fn get_coords(&self) -> std::collections::hash_set::Iter<'_, [isize; 2]> {
        self.indices.iter()
    }

    /// Return a vector of the connected components of the input
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
            (bounds[1][0] - bounds[0][0] + 1) as usize,
            (bounds[1][1] - bounds[0][1] + 1) as usize,
        ]
    }

    /// Get the dimensions rotated such that the long size is the first coordinate
    /// For now serves as "canonical form" on which we base coloring and lettering
    fn get_rotated_dimensions(&self) -> [usize; 2] {
        let mut dim = self.get_dimensions();
        if dim[0] < dim[1] {
            dim = [dim[1], dim[0]]
        }
        dim
    }

    // todo: try to influence the colors for the small dimension like [1,1], [2,1], etc., as they're
    //  very common and influence the look a lot
    // todo: give the colors some theming...
    /// Get color from the rotated dimension of a shape (by a hash function)
    pub fn hash_color(&self) -> Color32 {
        let mut hasher = DefaultHasher::new();
        let normal_form = self.normal_form();
        normal_form.hash(&mut hasher);
        let int = hasher.finish();
        println!("normal form: {:?}, hash: {}", normal_form, int);
        // get first 3 sets of 8 bits as rgb
        let r = (int & 255) as u8;
        let g = ((int & (255 * 256)) / 256) as u8;
        let b = ((int & (255 * 256 * 256)) / (256 * 256)) as u8;
        Color32::from_rgb(r, g, b)
    }

    /// Put the input in "normal form".
    /// Left-bottom coordinate is (0,0)
    /// More mass is below the x=y line than above it
    pub fn normal_form(&self) -> Self {
        let bounds = self.get_bounds();
        // Make left bottom coordinate [0,0]
        let mut indices: Vec<_> = self
            .indices
            .iter()
            .map(|[x, y]| [(x - bounds[0][0]), (y - bounds[0][1])])
            .collect();

        let [x, y] = self.get_dimensions();
        let mut x = x as isize;
        let mut y = y as isize;
        // Make long side the x coordinate
        if y < x {
            swap(&mut x, &mut y);
            indices = indices.iter().map(|[x, y]| [*y, *x]).collect()
        }

        // println!("x: {x}, y: {y}");
        // println!("translated indices {:?}", indices);

        // candidate orderings
        #[derive(Debug)]
        enum Candidates {
            LRB, // ordinary x y ordering (left to right, then top to bottom)
            RLB, // flipped x coordinate
            LRT, // flipped y coordinate
            RLT, // flipped both x and y coordinates
            BTR, // bottom to top, then left to right (only for squares)
            BTL,
            TBR,
            TBL,
        }
        impl Candidates {
            fn is_member(&self, indices: &Vec<[isize; 2]>, x: isize, y: isize, i: isize) -> bool {
                indices.contains(&self.flip(
                    x,
                    y,
                    &Candidates::get_coord_nonflipped_from_index(x, y, i),
                ))
            }

            fn get_coord_nonflipped_from_index(x: isize, y: isize, i: isize) -> [isize; 2] {
                [i % x, i / y]
            }

            fn flip(&self, x: isize, y: isize, coord: &[isize; 2]) -> [isize; 2] {
                match self {
                    Candidates::LRB => [coord[0], coord[1]],
                    Candidates::RLB => [(x - 1) - coord[0], coord[1]],
                    Candidates::LRT => [coord[0], (y - 1) - coord[1]],
                    Candidates::RLT => [(x - 1) - coord[0], (y - 1) - coord[1]],
                    Candidates::BTR => [coord[1], coord[0]],
                    Candidates::BTL => [coord[1], (y - 1) - coord[0]],
                    Candidates::TBR => [(x - 1) - coord[0], coord[1]],
                    Candidates::TBL => [(x - 1) - coord[0], (y - 1) - coord[1]],
                }
            }
        }

        let mut current_candidates = if x != y {
            // Rectangle case
            vec![
                Candidates::LRB,
                Candidates::RLB,
                Candidates::LRT,
                Candidates::RLT,
            ]
        } else {
            // Square case, there are now 8 symmetries instead of 4
            vec![
                Candidates::LRB,
                Candidates::RLB,
                Candidates::LRT,
                Candidates::RLT,
                Candidates::BTR,
                Candidates::BTL,
                Candidates::TBR,
                Candidates::TBL,
            ]
        };

        // Filter the candidates
        for i in 0..(x * y) {
            // println!(
            //     "{:?}",
            //     current_candidates
            //         .iter()
            //         .map(|c| c.is_member(&indices, x, y, i))
            //         .collect::<Vec<_>>()
            // );
            // println!("{:?}", current_candidates);

            if current_candidates
                .iter()
                .any(|c| c.is_member(&indices, x, y, i))
            {
                // if any ordering does contain i as a member, delete all candidates that
                //  do not contain i from the list
                current_candidates = current_candidates
                    .into_iter()
                    .filter(|c| c.is_member(&indices, x, y, i))
                    .collect();
            }

            if current_candidates.len() == 1 {
                break;
            }
        }
        // the right ordering is now current_candidates[0] (if there are multiple
        //  candidates left this indicates some sort of symmetry)

        Self {
            indices: HashSet::from_iter(
                indices
                    .iter()
                    .map(|coord| current_candidates[0].flip(x, y, coord)),
            ),
        }
    }
}
