use crate::app::colors::{
    COLOR_CENTER_DOT, COLOR_TILTED_X_AXIS, COLOR_TILTED_Y_AXIS, COLOR_X_AXIS, COLOR_Y_AXIS,
};
use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::line::centerpoint::generate_line_centerpoint;
use crate::app::generation::line::line_params::LineParams;
use crate::app::generation::line::LineAlg::Centerpoint;
use crate::app::generation::{AllAlgs, AllParams};
use crate::app::math::linear_algebra::Vec2;
use crate::app::param_field::ParamField;
use crate::app::plotting;
use crate::app::sampling::layer_parameters::LayerParameters;
use crate::app::ui::bits::even_odd_buttons;
use egui::{Align, Layout, Ui};
use egui_plot::{HLine, PlotUi, Points, VLine};
use std::f64::consts::PI;

mod centerpoint;
pub mod line_params;

#[derive(Clone)]
pub struct Line {}

impl Default for Line {
    fn default() -> Self {
        Line {}
    }
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum LineAlg {
    #[default]
    Centerpoint,
}

pub struct LineFields {
    pub rise: ParamField,
    pub run: ParamField,
    pub offset_x: ParamField,
    pub offset_y: ParamField,
    pub thickness: ParamField,
    pub length: ParamField,
}

impl Default for LineFields {
    fn default() -> Self {
        Self {
            rise: ParamField::new(true, false, "Rise".to_string(), [-1.0, 1.0], 0.01, vec![]),
            run: ParamField::new(true, false, "Run".to_string(), [-1.0, 1.0], 0.01, vec![]),
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
            thickness: ParamField::new(
                true,
                true,
                "Thickness".to_string(),
                [0.0, 3.0],
                0.01,
                vec![],
            ),
            length: ParamField::new(true, true, "Length".to_string(), [5.0, 30.0], 0.01, vec![]),
        }
    }
}

impl LineFields {
    pub fn all_fields_mut(&mut self) -> Vec<&mut ParamField> {
        vec![
            &mut self.rise,
            &mut self.run,
            &mut self.offset_x,
            &mut self.offset_y,
            &mut self.thickness,
            &mut self.length,
        ]
    }
}

impl Line {
    pub(crate) fn grid_size(all_params: Vec<&LineParams>) -> usize {
        all_params
            .iter()
            .map(|param| param.length + param.thickness)
            .fold(f64::NEG_INFINITY, |a, b| a.max(b))
            .ceil() as usize
            + 2
    }

    pub fn generate(alg: &LineAlg, params: &LineParams, grid_size: usize) -> Blocks {
        let rise_run = Vec2::from([params.run, params.rise]);
        let offset = Vec2::from([params.offset_x, params.offset_y]);

        match alg {
            Centerpoint => generate_line_centerpoint(
                rise_run,
                offset,
                params.thickness,
                params.length,
                grid_size,
            ),
        }
    }

    pub(crate) fn bounds(params: &LineParams, pad_factor: f64) -> [[f64; 2]; 2] {
        let center = Vec2::from([params.offset_x, params.offset_y]);
        let rr = Vec2::from([params.run, params.rise]).normalize();
        let rr_orth = rr.rot_90_CCW();
        let offset_long = params.length / 2.0 * rr;
        let offset_short = params.thickness / 2.0 * rr_orth;

        // the corners are now +- offset_long +- offset_short (assuming that center=0)
        let corners = [
            offset_long + offset_short,
            offset_long - offset_short,
            -offset_long + offset_short,
            -offset_long - offset_short,
        ];

        let x_max = corners
            .iter()
            .map(|pt| pt.x)
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));
        let y_max = corners
            .iter()
            .map(|pt| pt.y)
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));

        let lb = center + pad_factor * Vec2::from([-x_max, -y_max]);
        let rt = center + pad_factor * Vec2::from([x_max, y_max]);
        [[lb.x, lb.y], [rt.x, rt.y]]
    }

    pub fn show_options(
        ui: &mut Ui,
        params: &mut LineParams,
        fields: &mut LineFields,
        _: &mut LineAlg,
        parameters_current_layer_control: &mut Control,
        parameters_all_layers_control: &mut Control,
        sampling_points: &ZVec<Vec<f64>>,
        code_enabled: bool,
    ) {
        macro_rules! show_field {
            ($x:ident) => {
                fields.$x.show(
                    &mut params.$x,
                    ui,
                    &code_enabled,
                    sampling_points,
                    parameters_current_layer_control,
                    parameters_all_layers_control,
                    None,
                )
            };
        }
        show_field!(thickness);
        show_field!(length);
        show_field!(rise);
        show_field!(run);
        show_field!(thickness);

        ui.allocate_ui_with_layout(
            egui::Vec2::from([100.0, 200.0]),
            Layout::left_to_right(Align::Min),
            |ui| {
                [
                    ("0°", 1.0, 0.0),
                    ("30°", (PI / 6.0).cos(), (PI / 6.0).sin()),
                    ("45°", (PI / 4.0).cos(), (PI / 4.0).sin()),
                    ("60°", (PI / 3.0).cos(), (PI / 3.0).sin()),
                    ("1:1", 1.0, 1.0),
                    ("1:2", 1.0, 2.0),
                    ("1:3", 1.0, 3.0),
                    ("2:3", 2.0, 3.0),
                ]
                .map(|(name, x, y)| {
                    if ui.button(name).clicked() {
                        params.run = x;
                        params.rise = y;

                        parameters_current_layer_control.set_outdated();
                        parameters_all_layers_control.set_outdated();
                    }
                });
            },
        );
        ui.label(format!(
            "Angle: {:01}°",
            (params.rise.atan2(params.run) * 1800.0 / PI).round() / 10.0
        ));
        ui.separator();

        show_field!(offset_x);
        show_field!(offset_y);

        if even_odd_buttons(ui, &mut params.offset_x, &mut params.offset_y) {
            parameters_current_layer_control.set_outdated();
            parameters_all_layers_control.set_outdated();
        }

        if fields
            .all_fields_mut()
            .iter()
            .any(|field| field.has_changed())
        {
            parameters_current_layer_control.set_outdated();
            parameters_all_layers_control.set_outdated()
        }
    }

    pub fn draw_widgets(plot_ui: &mut PlotUi, params: &LineParams) {
        // Plot x and y axes through the center of the shape
        plot_ui.hline(HLine::new(params.offset_y).color(COLOR_X_AXIS).width(2.0));
        plot_ui.vline(VLine::new(params.offset_x).color(COLOR_Y_AXIS).width(2.0));

        // Plot rotated x and y axes for nonzero tilt (dark orange and purple)
        let tilt = params.rise.atan2(params.run);
        if tilt != 0.0 {
            plot_ui.line(
                plotting::tilted_line_in_bounds(
                    plot_ui.plot_bounds(),
                    tilt,
                    params.offset_x,
                    params.offset_y,
                )
                .color(COLOR_TILTED_X_AXIS),
            );
            plot_ui.line(
                plotting::tilted_line_in_bounds(
                    plot_ui.plot_bounds(),
                    PI / 2.0,
                    params.offset_x,
                    params.offset_y,
                )
                .color(COLOR_TILTED_Y_AXIS),
            );
        }

        // Plot center dot
        plot_ui.points(
            Points::new(vec![[params.offset_x, params.offset_y]])
                .radius(5.0)
                .color(COLOR_CENTER_DOT),
        );
    }

    pub fn set_parameters(
        layer_parameters: &mut LayerParameters,
        sampling_points: &Vec<f64>,
        default_shape: &LineParams,
        algorithm: &LineAlg,
        fields: &mut LineFields,
    ) where
        Self: Clone + Default,
    {
        layer_parameters.algorithm = AllAlgs::Line(*algorithm);
        layer_parameters.nr_samples = sampling_points.len();

        layer_parameters.parameters = sampling_points
            .iter()
            .map(|layer| AllParams::Line(Self::eval_param(layer, fields, default_shape)))
            .collect()
    }

    pub fn eval_param(layer: &f64, fields: &mut LineFields, default: &LineParams) -> LineParams {
        LineParams {
            rise: fields.rise.eval(layer).unwrap_or(default.rise),
            run: fields.run.eval(layer).unwrap_or(default.run),
            offset_x: fields.offset_x.eval(layer).unwrap_or(default.offset_x),
            offset_y: fields.offset_y.eval(layer).unwrap_or(default.offset_y),
            thickness: fields.thickness.eval(layer).unwrap_or(default.thickness),
            length: fields.length.eval(layer).unwrap_or(default.length),
        }
    }

    pub fn all_algs() -> Vec<AllAlgs> {
        vec![AllAlgs::Line(Centerpoint)]
    }
}
