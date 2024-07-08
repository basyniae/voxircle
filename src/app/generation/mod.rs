// For outputting the bitmatrices + size. Always solid, we do interior removal in preprocessing. Bunch of algorithms

mod centerpoint;
mod conservative;
mod contained;
mod diamond;
pub mod percentage; // want it public because we use the circle intersection area as a widget
mod square;

use crate::app::helpers::lin_alg::Mat2;
use crate::data_structures::Blocks;

use self::{
    centerpoint::generate_alg_centerpoint, conservative::generate_alg_conservative,
    contained::generate_alg_contained, diamond::generate_alg_diamond,
    percentage::generate_alg_percentage, square::generate_alg_square,
};

#[derive(Debug, PartialEq, Default)]
pub enum Algorithm {
    Conservative,
    Percentage(f64),
    Contained,
    #[default]
    CenterPoint, // Easiest algorithm so take as default
    Square,
    Diamond,
}

// Switch between algorithms
pub fn generate_all_blocks(
    algorithm: &Algorithm,
    center_offset_x: f64,
    center_offset_y: f64,
    sqrt_quad_form: Mat2,
    radius_major: f64,
) -> Blocks {
    match algorithm {
        Algorithm::Conservative => {
            generate_alg_conservative(
                radius_major,
                center_offset_x,
                center_offset_y,
                sqrt_quad_form,
            ) //
        }
        Algorithm::Percentage(percentage) => {
            generate_alg_percentage(radius_major, center_offset_x, center_offset_y, *percentage)
            //
        }
        Algorithm::Contained => {
            generate_alg_contained(
                center_offset_x,
                center_offset_y,
                sqrt_quad_form,
                radius_major,
            ) //
        }
        Algorithm::CenterPoint => {
            generate_alg_centerpoint(
                center_offset_x,
                center_offset_y,
                sqrt_quad_form,
                radius_major,
            ) //
        }
        Algorithm::Square => {
            generate_alg_square(radius_major, center_offset_x, center_offset_y) //
        }
        Algorithm::Diamond => {
            generate_alg_diamond(radius_major, center_offset_x, center_offset_y)
            //
        }
    }
}
