use crate::app::colors::{
    COLOR_CENTER_DOT, COLOR_TILTED_X_AXIS, COLOR_TILTED_Y_AXIS, COLOR_X_AXIS, COLOR_Y_AXIS,
};
use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::shape::{Shape, ShapeFields};
use crate::app::math::linear_algebra::Vec2;
use crate::app::param_field::ParamField;
use crate::app::plotting;
use crate::app::sampling::layer_parameters::LayerParameters;
use centerpoint::generate_alg_centerpoint;
use conservative::generate_alg_conservative;
use contained::generate_alg_contained;
use eframe::emath::Align;
use egui::{Color32, Layout, Ui};
use egui_plot::{HLine, PlotUi, Points, VLine};
use empty::generate_alg_empty;
use exact_squircle_bounds::exact_squircle_bounds;
use percentage::generate_alg_percentage;
use squircle_params::SquircleParams;
use std::f64::consts::{PI, TAU};
use std::fmt::Display;
use SquircleAlgorithm::{Centerpoint, Conservative, Contained, Empty, Percentage};

mod centerpoint;
mod conservative;
mod contained;
mod empty;
// want it public because we use the circle intersection area as a widget
pub mod exact_squircle_bounds;
pub mod percentage;
pub mod squircle_params;

/// Squircle shape struct. It's values are globally constant options for how a squircle can be made
///  from how the parameters are displayed
#[derive(Clone, Copy)]
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

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum SquircleAlgorithm {
    #[default]
    Centerpoint,
    Conservative,
    Contained,
    Percentage(f64),
    Empty,
}

pub struct SquircleFields {
    pub radius_a: ParamField,
    pub radius_b: ParamField,
    pub tilt: ParamField,
    pub offset_x: ParamField,
    pub offset_y: ParamField,
    pub squircle_parameter: ParamField,
}

impl Default for SquircleFields {
    fn default() -> Self {
        Self {
            radius_a: ParamField::new(
                true,
                true,
                "Radius A".to_string(),
                [0.0, 30.0],
                0.03,
                vec![],
            ),
            radius_b: ParamField::new(
                true,
                true,
                "Radius B".to_string(),
                [0.0, 30.0],
                0.03,
                vec![],
            ),
            tilt: ParamField::new(
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
                    ("1:3".to_string(), (1.0_f64 / 3.0).atan()),
                    ("2:3".to_string(), (2.0_f64 / 3.0).atan()),
                    ("1:4".to_string(), 0.25_f64.atan()),
                ],
            ),
            offset_x: ParamField::new(
                true,
                false,
                "x offset".to_string(),
                [-1.0, 1.0],
                0.01,
                vec![],
            ),
            offset_y: ParamField::new(
                true,
                false,
                "y offset".to_string(),
                [-1.0, 1.0],
                0.01,
                vec![],
            ),
            squircle_parameter: ParamField::new_param_func(
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
        }
    }
}

impl ShapeFields for SquircleFields {
    fn all_register_success(&mut self) {
        self.radius_a.register_success();
        self.radius_b.register_success();
        self.tilt.register_success();
        self.offset_x.register_success();
        self.offset_y.register_success();
        self.squircle_parameter.register_success();
    }

    fn has_any_changed(&mut self) -> bool {
        self.radius_a.has_changed()
            || self.radius_b.has_changed()
            || self.tilt.has_changed()
            || self.offset_x.has_changed()
            || self.offset_y.has_changed()
            || self.squircle_parameter.has_changed()
    }
}

impl Shape<SquircleAlgorithm, SquircleParams, SquircleFields> for Squircle {
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

    fn grid_size(all_params: &Vec<SquircleParams>) -> usize {
        // Determine grid size
        // The major radius should be included, for some metrics we need at least one layer of padding
        //  around the generated figure. Assuming a square figure (squircle parameter infinity), we
        //  need an x side length of 2.0 * sqrt(2) * radius_major. Add 4 for a padding of at least 2
        //  on each side.

        // Compute the largest radii for all shapes on this layer
        let largest_radius_a = all_params
            .iter()
            .fold(f64::NEG_INFINITY, |a, b| a.max(b.radius_a));
        let largest_radius_b = all_params
            .iter()
            .fold(f64::NEG_INFINITY, |a, b| a.max(b.radius_b));

        // Compute the largest offset for all shapes on this layer
        let largest_offset_x = all_params
            .iter()
            .fold(f64::NEG_INFINITY, |a, b| a.max(b.offset_x));
        let largest_offset_y = all_params
            .iter()
            .fold(f64::NEG_INFINITY, |a, b| a.max(b.offset_y));

        // Note that this method works but is kind of stupid. Ideally we'd want to have a grid that's
        //  a lot smaller but still contains all the shapes (it can't be centered at the origin then anymore)
        let grid_size = ((2.0 * 1.42 * f64::max(largest_radius_a, largest_radius_b)).ceil()
            + 2.0 * largest_offset_x.abs().max(largest_offset_y.abs()).ceil())
            as usize
            + 4;

        grid_size
    }

    fn generate(alg: &SquircleAlgorithm, params: &SquircleParams, grid_size: usize) -> Blocks {
        let center_offset = Vec2::from([params.offset_x, params.offset_y]);
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

    fn bounds(params: &SquircleParams, pad_factor: f64) -> [[f64; 2]; 2] {
        exact_squircle_bounds(params, pad_factor)
    }

    fn show_options(
        &mut self,
        ui: &mut Ui,
        params: &mut SquircleParams,
        param_fields: &mut SquircleFields,
        alg: &mut SquircleAlgorithm,
        parameters_current_layer_control: &mut Control,
        parameters_all_layers_control: &mut Control,
        sampling_points: &ZVec<Vec<f64>>,
        code_enabled: bool,
    ) {
        // algorithm-specific options
        match alg {
            Percentage(percentage) => {
                let mut perc_slider = percentage.clone();
                if ui
                    .add(
                        egui::Slider::new(&mut perc_slider, 0.0..=1.0)
                            .text("")
                            .fixed_decimals(2)
                            .custom_formatter(|n, _| {
                                format!("{:.0}%", n * 100.0) //  formatting of percentage slider
                            }),
                    )
                    .changed()
                {
                    *alg = Percentage(perc_slider);
                };
            }
            _ => {}
        }

        ui.checkbox(&mut self.single_radius, "Single radius");

        if self.single_radius {
            param_fields.radius_a.show(
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
            param_fields.radius_a.show(
                &mut params.radius_a,
                ui,
                &code_enabled,
                sampling_points,
                parameters_current_layer_control,
                parameters_all_layers_control,
                None,
            );

            // radius b
            param_fields.radius_b.show(
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
        param_fields.tilt.show(
            &mut params.tilt,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        // Squircle parameter
        param_fields.squircle_parameter.show(
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
        param_fields.offset_x.show(
            &mut params.offset_x,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        param_fields.offset_y.show(
            &mut params.offset_y,
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
                        params.offset_x = x;
                        params.offset_y = y;

                        parameters_current_layer_control.set_outdated();
                        parameters_all_layers_control.set_outdated();
                    }
                });
            },
        );

        if param_fields.has_any_changed() {
            parameters_current_layer_control.set_outdated();
            parameters_all_layers_control.set_outdated()
        }
    }

    fn draw(plot_ui: &mut PlotUi, params: SquircleParams, color: Color32) {
        plot_ui.line(plotting::superellipse_at_coords(params).color(color))
    }

    fn draw_widgets(plot_ui: &mut PlotUi, params: SquircleParams) {
        // Plot x and y axes through the center of the shape
        plot_ui.hline(HLine::new(params.offset_y).color(COLOR_X_AXIS).width(2.0));
        plot_ui.vline(VLine::new(params.offset_x).color(COLOR_Y_AXIS).width(2.0));

        // Plot rotated x and y axes for nonzero tilt (dark orange and purple)
        if params.tilt != 0.0 {
            plot_ui.line(
                plotting::tilted_line_in_bounds(
                    plot_ui.plot_bounds(),
                    params.tilt,
                    params.offset_x,
                    params.offset_y,
                )
                .color(COLOR_TILTED_X_AXIS),
            );
            plot_ui.line(
                plotting::tilted_line_in_bounds(
                    plot_ui.plot_bounds(),
                    params.tilt + PI / 2.0,
                    params.offset_x,
                    params.offset_y,
                )
                .color(COLOR_TILTED_Y_AXIS),
            );
        }

        // todo: reimplement
        // // Plot intersect area
        // if view.intersect_area {
        //     let grid_size =
        //         (2.0 * 1.42 * f64::max(shape_parameters.radius_a, shape_parameters.radius_b))
        //             .ceil() as usize
        //             + 4;
        //
        //     let square = Blocks::new((0..grid_size.pow(2)).map(|_| true).collect(), grid_size);
        //
        //     for coord in square.get_all_block_coords() {
        //         let cell_center = [coord[0] + 0.5, coord[1] + 0.5];
        //         let mut x_center = cell_center[0] - shape_parameters.center_offset_x;
        //         let mut y_center = cell_center[1] - shape_parameters.center_offset_y;
        //
        //         // Dihedral symmetry swaps (see percentage.rs for explanation)
        //         if x_center < 0.0 {
        //             x_center = -x_center;
        //         }
        //         if y_center < 0.0 {
        //             y_center = -y_center;
        //         }
        //         if x_center > y_center {
        //             (y_center, x_center) = (x_center, y_center);
        //         }
        //
        //         plot_ui.text(Text::new(PlotPoint::from(cell_center), {
        //             let value = squircle::percentage::cell_disk_intersection_area(
        //                 shape_parameters.radius_a.max(shape_parameters.radius_b),
        //                 x_center,
        //                 y_center,
        //             );
        //
        //             if value == 0.0 {
        //                 // Don't show zero intersect area
        //                 "".to_string()
        //             } else {
        //                 format!("{:.2}", value)
        //             }
        //         }));
        //     }
        // }

        // Plot center dot
        plot_ui.points(
            Points::new(vec![[params.offset_x, params.offset_y]])
                .radius(5.0)
                .color(COLOR_CENTER_DOT),
        );
    }

    // todo: make more generic
    fn set_parameters(
        &self,
        layer_parameters: &mut LayerParameters<
            SquircleAlgorithm,
            SquircleParams,
            SquircleFields,
            Squircle,
        >,
        sampling_points: &Vec<f64>,
        default_shape: &SquircleParams,
        algorithm: SquircleAlgorithm,
        fields: &mut SquircleFields,
    ) {
        // Set the algorithm & nr. of samples
        layer_parameters.algorithm = algorithm;
        layer_parameters.nr_samples = sampling_points.len();

        // If the code evaluation failed (returned None) resort to using the default_parameters (supplied by sliders)
        layer_parameters.parameters = sampling_points
            .iter()
            .map(|layer| SquircleParams {
                radius_a: fields
                    .radius_a
                    .eval(layer)
                    .unwrap_or(default_shape.radius_a),
                radius_b: if self.single_radius {
                    fields
                        .radius_a
                        .eval(layer)
                        .unwrap_or(default_shape.radius_a)
                } else {
                    fields
                        .radius_b
                        .eval(layer)
                        .unwrap_or(default_shape.radius_b)
                },
                tilt: fields.tilt.eval(layer).unwrap_or(default_shape.tilt),
                offset_x: fields
                    .offset_x
                    .eval(layer)
                    .unwrap_or(default_shape.offset_x),
                offset_y: fields
                    .offset_y
                    .eval(layer)
                    .unwrap_or(default_shape.offset_y),
                squircle_parameter: fields
                    .squircle_parameter
                    .eval(layer)
                    .unwrap_or(default_shape.squircle_parameter),
            })
            .collect()
    }

    fn update_slider_parameters(
        &self,
        current_layer_shape: &mut SquircleParams,
        layer: isize,
        fields: &mut SquircleFields,
    ) {
        // evaluate the rhai field at the layer
        if let Some(radius_a) = fields.radius_a.eval(&(layer as f64)) {
            current_layer_shape.radius_a = radius_a
        }

        if self.single_radius {
            if let Some(radius_a) = fields.radius_a.eval(&(layer as f64)) {
                current_layer_shape.radius_b = radius_a
            }
        } else {
            if let Some(radius_b) = fields.radius_b.eval(&(layer as f64)) {
                current_layer_shape.radius_b = radius_b
            }
        }

        if let Some(tilt) = fields.tilt.eval(&(layer as f64)) {
            current_layer_shape.tilt = tilt
        }
        if let Some(center_offset_x) = fields.offset_x.eval(&(layer as f64)) {
            current_layer_shape.offset_x = center_offset_x
        }
        if let Some(center_offset_y) = fields.offset_y.eval(&(layer as f64)) {
            current_layer_shape.offset_y = center_offset_y
        }

        if let Some(squircle_parameter) = fields.squircle_parameter.eval(&(layer as f64)) {
            current_layer_shape.squircle_parameter = squircle_parameter
        }
    }
}
