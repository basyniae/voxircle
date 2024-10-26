// For outputting the bitmatrices + size. Always solid, we do interior removal in preprocessing. Bunch of algorithms

use crate::app::data_structures::blocks::Blocks;
use crate::app::math::linear_algebra::{Mat2, Vec2};
use std::fmt::{Display, Formatter};

use self::{
    centerpoint::generate_alg_centerpoint, conservative::generate_alg_conservative,
    contained::generate_alg_contained, empty::generate_alg_empty,
    percentage::generate_alg_percentage,
};

mod centerpoint;
mod conservative;
mod contained;
mod empty;
pub mod percentage;
// want it public because we use the circle intersection area as a widget

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum Algorithm {
    #[default]
    Centerpoint,
    Conservative,
    Contained,
    Percentage(f64),
    Empty,
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
) -> Blocks {
    match algorithm {
        Algorithm::Centerpoint => {
            generate_alg_centerpoint(center_offset, sqrt_quad_form, squircle_parameter, grid_size)
        }
        Algorithm::Conservative => {
            generate_alg_conservative(center_offset, sqrt_quad_form, squircle_parameter, grid_size)
        }
        Algorithm::Contained => {
            generate_alg_contained(center_offset, sqrt_quad_form, squircle_parameter, grid_size)
        }
        Algorithm::Percentage(percentage) => generate_alg_percentage(
            f64::max(radius_a, radius_b),
            center_offset,
            *percentage,
            grid_size,
        ),
        Algorithm::Empty => generate_alg_empty(grid_size),
    }
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Algorithm::Centerpoint => {
                write!(f, "Centerpoint")
            }
            Algorithm::Conservative => {
                write!(f, "Conservative")
            }
            Algorithm::Contained => {
                write!(f, "Contained")
            }
            Algorithm::Percentage(percentage) => {
                write!(f, "Percentage, {:.0}%", percentage * 100.0)
            }
            Algorithm::Empty => {
                write!(f, "Empty")
            }
        }
    }
}
