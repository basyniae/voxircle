use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::squircle_params::SquircleParams;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::centerpoint::generate_alg_centerpoint;
use crate::app::generation::conservative::generate_alg_conservative;
use crate::app::generation::contained::generate_alg_contained;
use crate::app::generation::empty::generate_alg_empty;
use crate::app::generation::percentage::generate_alg_percentage;
use crate::app::generation::shape::Shape;
use crate::app::generation::squircle::SquircleAlgorithm::{
    Centerpoint, Conservative, Contained, Empty, Percentage,
};
use crate::app::math::exact_squircle_bounds::exact_squircle_bounds;
use crate::app::math::linear_algebra::Vec2;
use crate::app::param_field::ParamField;
use eframe::emath::Align;
use egui::{Layout, Ui};
use std::f64::consts::{PI, TAU};
use std::fmt::{Display, Formatter};

/// Squircle shape struct. It's values are globally constant options for how a squircle can be made
///  from how the parameters are displayed
pub struct Squircle {
    pub single_radius: bool,
}

impl Default for Squircle {
    fn default() -> Self {
        Self {
            single_radius: true,
        }
    }
}

impl Shape<SquircleAlgorithm, SquircleParams> for Squircle {
    fn describe(alg: &SquircleAlgorithm) -> String {
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

    fn name(alg: &SquircleAlgorithm) -> String {
        match alg {
            Centerpoint => "Centerpoint".to_string(),
            Conservative => "Conservative".to_string(),
            Contained => "Contained".to_string(),
            Percentage(_) => "Percentage".to_string(),
            Empty => "Empty".to_string(),
        }
    }

    fn all_algs() -> Vec<SquircleAlgorithm> {
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

    fn get_new_param_fields() -> Vec<ParamField> {
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

    fn bounds(params: &SquircleParams, pad_factor: f64) -> [[f64; 2]; 2] {
        exact_squircle_bounds(params, pad_factor)
    }

    fn show_options(
        &mut self,
        ui: &mut Ui,
        params: &mut SquircleParams,
        param_fields: &mut Vec<ParamField>,
        parameters_current_layer_control: &mut Control,
        parameters_all_layers_control: &mut Control,
        sampling_points: &ZVec<Vec<f64>>,
        code_enabled: bool,
    ) {
        // order:
        //  [0] <-> radius_a
        //  [1] <-> radius_b
        //  [2] <-> tilt
        //  [3] <-> center_offset_x
        //  [4] <-> center_offset_y
        //  [5] <-> squircle parameter

        ui.checkbox(&mut self.single_radius, "Single radius");

        if self.single_radius {
            param_fields[0].show(
                &mut params.radius_a,
                ui,
                &code_enabled,
                sampling_points,
                parameters_current_layer_control,
                parameters_all_layers_control,
                Some(&"Radius".to_string()),
            );
            params.radius_b = params.radius_a;
        } else {
            // radius a
            param_fields[0].show(
                &mut params.radius_a,
                ui,
                &code_enabled,
                sampling_points,
                parameters_current_layer_control,
                parameters_all_layers_control,
                None,
            );

            // radius b
            param_fields[1].show(
                &mut params.radius_b,
                ui,
                &code_enabled,
                sampling_points,
                parameters_current_layer_control,
                parameters_all_layers_control,
                None,
            );

            //longterm: Make circular slider for more intuitive controls (need to build this myapp probably)
        }

        //tilt
        param_fields[2].show(
            &mut params.tilt,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        // Squircle parameter
        param_fields[5].show(
            &mut params.squircle_parameter,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        // Centerpoint
        ui.separator();
        param_fields[3].show(
            &mut params.center_offset_x,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        param_fields[4].show(
            &mut params.center_offset_y,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        // Add odd and even buttons (also good so people understand what the abstraction "offset center" actually means)
        // todo: fix. how to make modular (linked parameters x and y?)
        ui.allocate_ui_with_layout(
            egui::Vec2::from([100.0, 200.0]),
            Layout::left_to_right(Align::Min),
            |ui| {
                [("Even center", 0.0, 0.0), ("Odd center", 0.5, 0.5)].map(|(name, x, y)| {
                    if ui.button(name).clicked() {
                        params.center_offset_x = x;
                        params.center_offset_y = y;

                        parameters_current_layer_control.set_outdated();
                        parameters_all_layers_control.set_outdated();
                    }
                });
            },
        );

        if param_fields[0].has_changed()
            || param_fields[1].has_changed()
            || param_fields[2].has_changed()
            || param_fields[3].has_changed()
            || param_fields[4].has_changed()
            || param_fields[5].has_changed()
        {
            parameters_current_layer_control.set_outdated();
            parameters_all_layers_control.set_outdated()
        }
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
