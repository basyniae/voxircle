// For outputting the bitmatrices + size. Always solid, we do interior removal in preprocessing. Bunch of algorithms

mod centerpoint;
mod conservative;
mod contained;
pub mod percentage; // want it public because we use the circle intersection area as a widget

use crate::app::helpers::lin_alg::{Vec2, Mat2};
use crate::data_structures::Blocks;

use self::{
    centerpoint::generate_alg_centerpoint, conservative::generate_alg_conservative,
    contained::generate_alg_contained,
    percentage::generate_alg_percentage,
};

#[derive(Debug, PartialEq, Default)]
pub enum Algorithm {
    Conservative,
    Percentage(f64),
    Contained,
    #[default]
    CenterPoint, // Easiest algorithm so take as default
}

// Switch between algorithms
pub fn generate_all_blocks(
    algorithm: &Algorithm,
    center_offset: Vec2,
    sqrt_quad_form: Mat2,
    radius_major: f64,
    squircle_parameter: f64,
) -> Blocks {
    match algorithm {
        Algorithm::Conservative => {
            generate_alg_conservative(
                radius_major,
                center_offset,
                sqrt_quad_form,
            ) //
        }
        Algorithm::Percentage(percentage) => {
            generate_alg_percentage(radius_major, center_offset, *percentage)
            //
        }
        Algorithm::Contained => {
            generate_alg_contained(
                center_offset,
                sqrt_quad_form,
                radius_major,
                squircle_parameter,
            ) //
        }
        Algorithm::CenterPoint => {
            generate_alg_centerpoint(
                center_offset,
                sqrt_quad_form,
                radius_major,
                squircle_parameter,
            ) //
        }
    }
}
