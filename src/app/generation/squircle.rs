use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::{AllAlgs, AllParams};
use crate::app::math::linear_algebra::Vec2;
use crate::app::param_config::ParamConfig;
use crate::app::param_field::ParamField;
use crate::app::sampling::layer_parameters::LayerParameters;
use crate::app::ui::bits::{draw_axes, even_odd_buttons};
use centerpoint::generate_alg_centerpoint;
use conservative::generate_alg_conservative;
use contained::generate_alg_contained;
use egui::Ui;
use egui_plot::PlotUi;
use empty::generate_alg_empty;
use exact_squircle_bounds::exact_squircle_bounds;
use percentage::generate_alg_percentage;
use squircle_params::SquircleParams;
use std::f64::consts::{PI, TAU};
use SquircleAlg::{Centerpoint, Conservative, Contained, Empty, Percentage};

mod centerpoint;
mod conservative;
mod contained;
mod empty;
mod exact_squircle_bounds;
// want it public because we use the circle intersection area as a widget
mod percentage;
pub mod squircle_params;

/// Squircle shape struct. It's values are globally constant options for how a squircle can be made
///  from how the parameters are displayed
#[derive(Clone, Copy)]
pub struct Squircle {}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum SquircleAlg {
    #[default]
    Centerpoint,
    Conservative,
    Contained,
    Percentage(f64),
    Empty,
}

impl SquircleAlg {
    pub fn all_algs() -> Vec<AllAlgs> {
        vec![
            AllAlgs::Squircle(Centerpoint),
            AllAlgs::Squircle(Conservative),
            AllAlgs::Squircle(Contained),
            AllAlgs::Squircle(Percentage(0.5)),
            AllAlgs::Squircle(Empty),
        ]
    }
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

impl SquircleFields {
    pub fn all_fields_mut(&mut self) -> Vec<&mut ParamField> {
        vec![
            &mut self.radius_a,
            &mut self.radius_b,
            &mut self.tilt,
            &mut self.offset_x,
            &mut self.offset_y,
            &mut self.squircle_parameter,
        ]
    }

    pub fn all_register_success(&mut self) {
        for i in self.all_fields_mut() {
            i.register_success()
        }
    }
}

impl Squircle {
    pub fn grid_size(all_params: Vec<&SquircleParams>) -> usize {
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

    pub fn generate(alg: &SquircleAlg, params: &SquircleParams, grid_size: usize) -> Blocks {
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

    pub fn bounds(params: &SquircleParams, pad_factor: f64) -> [[f64; 2]; 2] {
        exact_squircle_bounds(params, pad_factor)
    }

    pub fn show_options(
        ui: &mut Ui,
        params: &mut SquircleParams,
        fields: &mut SquircleFields,
        alg: &mut SquircleAlg,
        sampling_points: &ZVec<Vec<f64>>,
        code_enabled: bool,
        param_config: &mut ParamConfig,
    ) -> bool {
        let mut changed = false;
        macro_rules! show_field {
            ($x:ident) => {
                fields.$x.show(
                    &mut params.$x,
                    ui,
                    &code_enabled,
                    sampling_points,
                    &mut changed,
                    None,
                )
            };
        }

        // algorithm-specific options
        match alg {
            Percentage(percentage) => {
                let mut perc_slider = percentage.clone();
                if crate::app::ui::bits::percentage_slider(ui, &mut perc_slider) {
                    *alg = Percentage(perc_slider);
                };
            }
            _ => {}
        }

        ui.checkbox(&mut param_config.single_radius, "Single radius");

        if param_config.single_radius {
            fields.radius_a.show(
                &mut params.radius_a,
                ui,
                &code_enabled,
                sampling_points,
                &mut changed,
                Some(&"Radius".to_string()),
            );
            params.radius_b = params.radius_a;
        } else {
            show_field!(radius_a);
            show_field!(radius_b);
        }
        //longterm: Make circular slider for more intuitive controls (need to build this myapp probably)

        ui.separator();
        show_field!(tilt);
        show_field!(squircle_parameter);
        ui.separator();
        show_field!(offset_x);
        show_field!(offset_y);

        // Add odd and even buttons (also good so people understand what the abstraction "offset center" actually means)
        if even_odd_buttons(ui, &mut params.offset_x, &mut params.offset_y) {
            changed = true
        }

        if fields
            .all_fields_mut()
            .iter()
            .any(|field| field.has_changed())
        {
            changed = true
        }

        changed
    }

    pub fn draw_widgets(plot_ui: &mut PlotUi, params: &SquircleParams) {
        draw_axes(plot_ui, params.offset_x, params.offset_y, params.tilt);

        // todo: reimplement intersect area
    }

    pub fn set_parameters(
        layer_parameters: &mut LayerParameters,
        sampling_points: &Vec<f64>,
        default_shape: &SquircleParams,
        algorithm: &SquircleAlg,
        fields: &mut SquircleFields,
        param_config: &ParamConfig,
    ) {
        // Set the algorithm & nr. of samples
        layer_parameters.algorithm = AllAlgs::Squircle(*algorithm);

        // If the code evaluation failed (returned None) resort to using the default_parameters (supplied by sliders)
        layer_parameters.parameters = sampling_points
            .iter()
            .map(|layer| {
                AllParams::Squircle(Self::eval_param(layer, fields, default_shape, param_config))
            })
            .collect()
    }

    /// Overwrite params with values coming from the fields at the specified layer if the code
    /// can be evaluated. Otherwise, overwrite with
    pub fn eval_param(
        layer: &f64,
        fields: &mut SquircleFields,
        default: &SquircleParams,
        param_config: &ParamConfig,
    ) -> SquircleParams {
        SquircleParams {
            radius_a: fields.radius_a.eval(layer).unwrap_or(default.radius_a),
            radius_b: if param_config.single_radius {
                fields.radius_a.eval(layer).unwrap_or(default.radius_a)
            } else {
                fields.radius_b.eval(layer).unwrap_or(default.radius_b)
            },
            tilt: fields.tilt.eval(layer).unwrap_or(default.tilt),
            offset_x: fields.offset_x.eval(layer).unwrap_or(default.offset_x),
            offset_y: fields.offset_y.eval(layer).unwrap_or(default.offset_y),
            squircle_parameter: fields
                .squircle_parameter
                .eval(layer)
                .unwrap_or(default.squircle_parameter),
        }
    }
}
