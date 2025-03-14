// For outputting the bitmatrices + size. Always solid, we do interior removal in preprocessing. Bunch of algorithms

use self::{
    centerpoint::generate_alg_centerpoint, conservative::generate_alg_conservative,
    contained::generate_alg_contained, empty::generate_alg_empty,
    percentage::generate_alg_percentage,
};
use crate::app::data_structures::blocks::Blocks;
use crate::app::generation::Algorithm::{Centerpoint, Conservative, Contained, Percentage};
use crate::app::math::linear_algebra::{Mat2, Vec2};
use egui::Ui;
use std::fmt::{Display, Formatter};

mod centerpoint;
mod conservative;
mod contained;
mod empty;
pub mod percentage; // want it public because we use the circle intersection area as a widget
mod shape;
mod square;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum Algorithm {
    #[default]
    Centerpoint,
    Conservative,
    Contained,
    Percentage(f64),
    Empty,
}

impl Algorithm {
    /// Description for info display
    pub fn describe(&self) -> String {
        match self {
            Centerpoint => {"Include a particular block iff its centerpoint is in the ellipse".to_string()}
            Conservative => {"Include a particular block in the voxelization iff it has nonempty intersection with the ellipse".to_string()}
            Contained => {"Include a particular block iff it is fully contained in the ellipse".to_string()}
            Percentage(percentage) => {format!(
                "Include a particular block in the voxelization iff more than {:.0}% of it is contained in the circle. Ellipses and squircles not implemented.",
                100.0 * percentage
            )}
            Algorithm::Empty => {"Include no blocks in the voxelization".to_string()}
        }
    }

    /// Name of algorithms (for combobox display)
    fn name(&self) -> String {
        match self {
            Centerpoint => "Centerpoint".to_string(),
            Conservative => "Conservative".to_string(),
            Contained => "Contained".to_string(),
            Percentage(_) => "Percentage".to_string(),
            Algorithm::Empty => "Empty".to_string(),
        }
    }

    /// List of all algorithms that we want to make selectable
    fn all_algs() -> Vec<Self> {
        vec![Centerpoint, Conservative, Contained, Percentage(0.5)]
    }

    /// Generate a combo box to pick from all the algorithms. Return true if
    /// the value has changed (this frame).
    pub fn combo_box(ui: &mut Ui, alg: &mut Self) -> bool {
        let old_alg = alg.clone();
        egui::ComboBox::from_label("Algorithm")
            .selected_text(format!("{:}", alg))
            .show_ui(ui, |ui| {
                for i in Self::all_algs() {
                    ui.selectable_value(alg, i, i.name());
                }
            });
        old_alg != *alg
    }
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
