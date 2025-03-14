use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::squircle_params::SquircleParams;
use crate::app::generation::centerpoint::generate_alg_centerpoint;
use crate::app::generation::conservative::generate_alg_conservative;
use crate::app::generation::contained::generate_alg_contained;
use crate::app::generation::empty::generate_alg_empty;
use crate::app::generation::percentage::generate_alg_percentage;
use crate::app::generation::shape::Shape;
use crate::app::generation::squircle::SquircleAlgorithm::{
    Centerpoint, Conservative, Contained, Empty, Percentage,
};
use crate::app::math::linear_algebra::Vec2;
use crate::app::param_field::ParamField;
use egui::Ui;
use std::f64::consts::{PI, TAU};
use std::fmt::{Display, Formatter};

pub struct Squircle {}

impl Shape<SquircleAlgorithm, SquircleParams> for Squircle {
    fn describe(&self, alg: &SquircleAlgorithm) -> String {
        match alg {
            Centerpoint => {"Include a particular block iff its centerpoint is in the ellipse".to_string()}
            Conservative => {"Include a particular block in the voxelization iff it has nonempty intersection with the ellipse".to_string()}
            Contained => {"Include a particular block iff it is fully contained in the ellipse".to_string()}
            Percentage(percentage) => {format!(
                "Include a particular block in the voxelization iff more than {:.0}% of it is contained in the circle. Ellipses and squircles not implemented.",
                100.0 * percentage
            )}
            Empty => {"Include no blocks in the voxelization".to_string()}
        }
    }

    fn name(&self, alg: &SquircleAlgorithm) -> String {
        match alg {
            Centerpoint => "Centerpoint".to_string(),
            Conservative => "Conservative".to_string(),
            Contained => "Contained".to_string(),
            Percentage(_) => "Percentage".to_string(),
            Empty => "Empty".to_string(),
        }
    }

    fn all_algs(&self) -> Vec<SquircleAlgorithm> {
        vec![Centerpoint, Conservative, Contained, Percentage(0.5)]
    }

    fn generate(alg: &SquircleAlgorithm, params: &SquircleParams, grid_size: usize) -> Blocks {
        let center_offset = Vec2::from([params.center_offset_x, params.center_offset_y]);
        let sqrt_quad_form = params.get_sqrt_quad_form();

        match alg {
            Centerpoint => generate_alg_centerpoint(
                center_offset,
                sqrt_quad_form,
                params.squircle_parameter,
                grid_size,
            ),
            Conservative => generate_alg_conservative(
                center_offset,
                sqrt_quad_form,
                params.squircle_parameter,
                grid_size,
            ),
            Contained => generate_alg_contained(
                center_offset,
                sqrt_quad_form,
                params.squircle_parameter,
                grid_size,
            ),
            Percentage(percentage) => generate_alg_percentage(
                f64::max(params.radius_a, params.radius_b),
                center_offset,
                *percentage,
                grid_size,
            ),
            Empty => generate_alg_empty(grid_size),
        }
    }

    fn get_new_param_fields(&self) -> Vec<ParamField> {
        vec![
            ParamField::new(
                true,
                true,
                "Radius A".to_string(),
                [0.0, 30.0],
                0.03,
                vec![],
            ),
            ParamField::new(
                true,
                true,
                "Radius B".to_string(),
                [0.0, 30.0],
                0.03,
                vec![],
            ),
            ParamField::new(
                true,
                false,
                "Tilt".to_string(),
                [-TAU, TAU],
                0.01,
                vec![
                    ("0°".to_string(), 0.0),
                    ("30°".to_string(), PI / 6.0),
                    ("45°".to_string(), PI / 4.0),
                    ("1:2".to_string(), 0.5_f64.atan()),
                    ("1:3".to_string(), 0.33333333333333_f64.atan()),
                    ("2:3".to_string(), 0.66666666666666_f64.atan()),
                    ("1:4".to_string(), 0.25_f64.atan()),
                ],
            ),
            ParamField::new(
                true,
                false,
                "x offset".to_string(),
                [-1.0, 1.0],
                0.03,
                vec![],
            ),
            ParamField::new(
                true,
                false,
                "y offset".to_string(),
                [-1.0, 1.0],
                0.03,
                vec![],
            ),
            ParamField::new_param_func(
                false,
                true,
                "Squircleness".to_string(),
                [0.0, 1.0],
                0.01,
                vec![
                    ("Circle".to_string(), 2.0),              // Squircle parameter 2
                    ("Astroid".to_string(), 0.6666666666666), // "" "" 2/3
                    ("Diamond".to_string(), 1.0),             // "" "" 1
                    ("Square".to_string(), f64::INFINITY),    // "" "" infinity
                ],
                |x| 1.0 / (1.0 - x) - 1.0,
                |p| 1.0 - 1.0 / (p + 1.0),
            ),
        ]
    }
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum SquircleAlgorithm {
    #[default]
    Centerpoint,
    Conservative,
    Contained,
    Percentage(f64),
    Empty,
}

impl SquircleAlgorithm {
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
            Empty => {"Include no blocks in the voxelization".to_string()}
        }
    }

    /// Name of algorithms (for combobox display)
    fn name(&self) -> String {
        match self {
            Centerpoint => "Centerpoint".to_string(),
            Conservative => "Conservative".to_string(),
            Contained => "Contained".to_string(),
            Percentage(_) => "Percentage".to_string(),
            Empty => "Empty".to_string(),
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

impl Display for SquircleAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SquircleAlgorithm::Centerpoint => {
                write!(f, "Centerpoint")
            }
            SquircleAlgorithm::Conservative => {
                write!(f, "Conservative")
            }
            SquircleAlgorithm::Contained => {
                write!(f, "Contained")
            }
            SquircleAlgorithm::Percentage(percentage) => {
                write!(f, "Percentage, {:.0}%", percentage * 100.0)
            }
            SquircleAlgorithm::Empty => {
                write!(f, "Empty")
            }
        }
    }
}
