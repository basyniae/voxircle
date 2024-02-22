// For outputting the bitmatrices + size. Always solid, we do interior removal in preprocessing. Bunch of algorithms

mod centerpoint;
mod contained;
mod conservative;
pub mod percentage;
mod square;

use crate::data_structures::Blocks;

use self::{centerpoint::generate_alg_centerpoint, conservative::generate_alg_conservative, contained::generate_alg_contained, percentage::generate_alg_percentage, square::generate_alg_square};

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
        Algorithm::Conservative => {
            generate_alg_conservative(radius, center_offset_x, center_offset_y)
        },
        Algorithm::Percentage(param) => {
            generate_alg_percentage(radius, center_offset_x, center_offset_y, *param)
        },
        Algorithm::Contained => {
            generate_alg_contained(radius, center_offset_x, center_offset_y)
        },
        Algorithm::CenterPoint => {
            generate_alg_centerpoint(radius, center_offset_x, center_offset_y)
        },
        Algorithm::Square => {
            generate_alg_square(radius, center_offset_x, center_offset_y)
        },
        Algorithm::Diamond => todo!(),
    }
}