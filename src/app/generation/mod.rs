// For outputting the bitmatrices + size. Always solid, we do interior removal in preprocessing. Bunch of algorithms

mod centerpoint;

use std::default;

use crate::data_structures::Blocks;

use self::centerpoint::generate_alg_centerpoints;

#[derive(Debug, PartialEq, Default)]
pub enum Algorithm {
    Conservative,
    Percentage(f64),
    Contained,
    #[default]
    CenterPoint, // Easiest algorithm so take as default
    Square,
    Diamond
}

// Switch between algorithms
pub fn generate_all_blocks(algorithm: &Algorithm, radius: f64, center_offset_x: f64, center_offset_y: f64) -> Blocks {
    match algorithm {
        Algorithm::Conservative => todo!(),
        Algorithm::Percentage(_) => todo!(),
        Algorithm::Contained => todo!(),
        Algorithm::CenterPoint => {
            generate_alg_centerpoints(radius, center_offset_x, center_offset_y)
        },
        Algorithm::Square => todo!(),
        Algorithm::Diamond => todo!(),
    }
}