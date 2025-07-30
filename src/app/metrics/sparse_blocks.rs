use crate::app::data_structures::blocks::Blocks;
use crate::app::math::sparse_graph::SparseGraph;
use angular_units::Turns;
use egui::Color32;
use prisma::{FromColor, Hsl, Rgb};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem::swap;

static SMALL_SHAPE_HASHES: [u64; 10] = [
    6420002640197838707,  // 1x1
    2915436035146535286,  // 2x1
    6444618775187902610,  // 3x1
    7776891254995127166,  // 4x1
    11950804322075204449, // 5x1
    8096643921108516772,  // Triomino Bracket
    17590444270234595342, // Tetromino Skew
    8477383144400419704,  // Tetromino T
    7757481152709758788,  // Tetromino Square
    13159953227328387611, // Tetromino L
];

// From matplotlib qualitative Set 3 (colorpicked from site so not exact)
// https://matplotlib.org/stable/users/explain/colors/colormaps.html#qualitative
// HSL ranges:
//  hue: any
//  saturation: 0-66%
//  lightness: 49-76%
static SMALL_SHAPE_COLORS: [Color32; 11] = [
    Color32::from_rgb(109, 176, 164),
    Color32::from_rgb(219, 219, 145),
    Color32::from_rgb(156, 151, 182),
    Color32::from_rgb(216, 97, 82),
    Color32::from_rgb(97, 144, 176),
    Color32::from_rgb(217, 146, 68),
    Color32::from_rgb(145, 188, 74),
    Color32::from_rgb(216, 170, 193),
    // Color32::from_rgb(182, 182, 182), // Too close to block gray
    Color32::from_rgb(154, 97, 156),
    Color32::from_rgb(170, 199, 162),
    Color32::from_rgb(219, 201, 80),
];

/// Sparse representation of a blocks object (that forgets about the origin)
/// `indices` contains x iff there is a block at index x
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparseBlocks {
    indices: HashSet<[isize; 2]>,
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

    /// Get the center of the sparse blocks (mean of the coordinates)
    pub fn get_center(&self) -> [f64; 2] {
        let mut x = 0.0;
        let mut y = 0.0;
        let nr_blocks = self.indices.len() as f64;

        for i in self.indices.iter() {
            x += i[0] as f64;
            y += i[1] as f64;
        }

        [x / nr_blocks + 0.5, y / nr_blocks + 0.5]
    }
}

impl SparseBlocks {
    /// Return true iff the two sparse blocks have overlap
    fn is_overlapping(comp_a: &SparseBlocks, comp_b: &SparseBlocks) -> bool {
        comp_a.indices.iter().any(|x| comp_b.indices.contains(x))
    }

    /// Return true iff the two sparse blocks are strongly connected (share an edge)
    fn is_strongly_connected(comp_a: &SparseBlocks, comp_b: &SparseBlocks) -> bool {
        comp_a.indices.iter().any(|p| {
            // loop over points around p
            [
                [p[0] + 1, p[1]],
                [p[0] - 1, p[1]],
                [p[0], p[1] + 1],
                [p[0], p[1] - 1],
                [p[0], p[1]],
            ]
            .iter()
            .any(|p_offset| comp_b.indices.contains(p_offset))
        })
    }

    /// Return true iff the two sparse blocks are weakly connected (share a vertex)
    fn is_weakly_connected(comp_a: &SparseBlocks, comp_b: &SparseBlocks) -> bool {
        comp_a.indices.iter().any(|p| {
            // loop over 3x3 grid around p
            [
                [p[0] + 1, p[1] + 1],
                [p[0] + 1, p[1] - 1],
                [p[0] + 1, p[1]],
                [p[0] - 1, p[1] + 1],
                [p[0] - 1, p[1] - 1],
                [p[0] - 1, p[1]],
                [p[0], p[1] + 1],
                [p[0], p[1] - 1],
                [p[0], p[1]],
            ]
            .iter()
            .any(|p_offset| comp_b.indices.contains(p_offset))
        })
    }
}
impl SparseBlocks {
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

    // todo: found tour in this graph that is as efficient as possible.
    //  at least the graph is planar.
    //  try: identify largest cycle in the graph, expand that to include the components which
    //  are only connected to one other (the 'leaves').
    //  does this generally visit all points? no.
    //  We can decompose into graph-connected components
    //  throw explicit error when the algorithm fails
    /// For a vector of sparse_blocks, check which ones are weakly connected (so also diagonally)
    /// If the input has length n, the output looks as follows.
    /// `weak_connection_graph(vec_sparse_blocks)[i][n-1 - j] == true` if and only if component i and j
    /// are connected (i,j=0,...,n-1).
    /// As convention, we pick that blocks aren't connected to themselves.
    pub fn weak_connection_graph(all_comps: &Vec<Self>) -> SparseGraph {
        let n = all_comps.len();
        let mut running_edges = vec![];
        for i in 0..n {
            for j in i..n {
                if i != j && SparseBlocks::is_weakly_connected(&all_comps[i], &all_comps[j]) {
                    running_edges.push([i, j])
                }
            }
        }

        SparseGraph::new(n, running_edges)
    }

    /// Get color from the rotated dimension of a shape (by a hash function)
    /// Assuming the input is in normal form already, allows slight optimization
    pub fn hash_color_from_normal_form(&self) -> Color32 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();
        // for getting small shapes:
        // println!("Shape: {:?}, Hash: {}", normal_form, int);
        SMALL_SHAPE_HASHES
            .iter()
            .position(|&h| h == hash)
            .map(|h| SMALL_SHAPE_COLORS[h]) // if the shape is small, look up a hardcoded color
            .unwrap_or_else(|| {
                // random float between 0.0 and 1.0 for hue
                let random1 = (hash & 255) as f64 / 256.0;
                // generate 2 random floats between 0.0 and 1.0 from the hash
                let random2 = ((hash >> 8) & 255) as f64 / 256.0;
                let random3 = ((hash >> 16) & 255) as f64 / 256.0;

                // make range match those of SMALL_SHAPE_COLORS
                let hsl = Hsl::new(
                    Turns(random1),
                    0.66 * random3 + 0.34,
                    (0.76 - 0.49) * random2 + 0.49,
                );

                let rgb = Rgb::from_color(&hsl);
                let rgb_u8: Rgb<u8> = rgb.color_cast();

                // debug
                // println!("Hash: {hash}");
                // println!("Normal form: {self:?}");
                // println!(
                //     "Color: {}, {}, {}",
                //     rgb_u8.red(),
                //     rgb_u8.green(),
                //     rgb_u8.blue()
                // );

                Color32::from_rgb(rgb_u8.red(), rgb_u8.green(), rgb_u8.blue())
            })
    }

    // fixme: there are some small errors in this code, namely the symmetry does not work 100% of the
    //  time. Have to fiddle around with special cases
    /// Put the input in "normal form".
    /// Left-bottom coordinate is (0,0)
    /// Rotated/flipped so that the associated binary number is lowest. This is computed as,
    /// scanning from right to left, bottom to top starting at (0,0), adding 2^i if there is a block
    /// at scan index i.
    pub fn normal_form(&self) -> Self {
        let bounds = self.get_bounds();
        // Make left bottom coordinate [0,0]
        let mut indices: Vec<_> = self
            .indices
            .iter()
            .map(|[x, y]| [x - bounds[0][0], y - bounds[0][1]])
            .collect();

        let [x, y] = self.get_dimensions();
        let mut x = x as isize;
        let mut y = y as isize;
        // Make long side the x coordinate
        if y > x {
            swap(&mut x, &mut y);
            indices = indices.iter().map(|[x, y]| [*y, *x]).collect()
        }

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
