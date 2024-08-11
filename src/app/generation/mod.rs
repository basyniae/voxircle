// For outputting the bitmatrices + size. Always solid, we do interior removal in preprocessing. Bunch of algorithms

mod centerpoint;
mod conservative;
mod contained;
pub mod percentage;
// want it public because we use the circle intersection area as a widget

use crate::app::helpers::blocks::Blocks;
use crate::app::helpers::linear_algebra::{Mat2, Vec2};

use self::{
    centerpoint::generate_alg_centerpoint, conservative::generate_alg_conservative,
    contained::generate_alg_contained, percentage::generate_alg_percentage,
};

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum Algorithm {
    #[default]
    CenterPoint,
    Conservative,
    Contained,
    Percentage(f64),
}

// Switch between algorithms
pub fn generate_all_blocks(
    algorithm: &Algorithm,
    center_offset: Vec2,
    sqrt_quad_form: Mat2,
    squircle_parameter: f64,
    radius_a: f64,
    radius_b: f64,
    grid_size: usize,
    origin: Vec2,
) -> Blocks {
    match algorithm {
        Algorithm::CenterPoint => generate_alg_centerpoint(
            center_offset,
            sqrt_quad_form,
            squircle_parameter,
            grid_size,
            origin,
        ),
        Algorithm::Conservative => generate_alg_conservative(
            center_offset,
            sqrt_quad_form,
            squircle_parameter,
            grid_size,
            origin,
        ),
        Algorithm::Contained => generate_alg_contained(
            center_offset,
            sqrt_quad_form,
            squircle_parameter,
            grid_size,
            origin,
        ),
        Algorithm::Percentage(percentage) => generate_alg_percentage(
            f64::max(radius_a, radius_b),
            center_offset,
            *percentage,
            grid_size,
            origin,
        ),
    }
}
