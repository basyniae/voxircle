use crate::app::colors::{
    COLOR_CENTER_DOT, COLOR_TILTED_X_AXIS, COLOR_TILTED_Y_AXIS, COLOR_X_AXIS, COLOR_Y_AXIS,
};
use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::line::centerpoint::generate_line_centerpoint;
use crate::app::generation::line::line_params::LineParams;
use crate::app::generation::line::LineAlg::Centerpoint;
use crate::app::generation::shape::{TraitAlgorithm, TraitFields, TraitParameters, TraitShape};
use crate::app::math::linear_algebra::Vec2;
use crate::app::param_field::ParamField;
use crate::app::plotting;
use crate::app::sampling::layer_parameters::LayerParameters;
use eframe::epaint::Color32;
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

impl TraitAlgorithm for LineAlg {}

impl TraitParameters for LineParams {}

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

impl TraitFields for LineFields {
    fn all_register_success(&mut self) {
        self.rise.register_success();
        self.run.register_success();
        self.offset_x.register_success();
        self.offset_y.register_success();
        self.thickness.register_success();
        self.length.register_success()
    }

    fn has_any_changed(&mut self) -> bool {
        self.rise.has_changed()
            || self.run.has_changed()
            || self.offset_x.has_changed()
            || self.offset_y.has_changed()
            || self.thickness.has_changed()
            || self.length.has_changed()
    }
}

impl TraitShape<LineAlg, LineParams, LineFields> for Line {
    fn describe(alg: &LineAlg) -> String {
        match alg {
            Centerpoint => {
                "Include a particular block iff its centerpoint is in the line".to_string()
            }
        }
    }

    fn name(alg: &LineAlg) -> String {
        match alg {
            Centerpoint => "Centerpoint".to_string(),
        }
    }

    fn all_algs() -> Vec<LineAlg> {
        vec![Centerpoint]
    }

    fn grid_size(all_params: &Vec<LineParams>) -> usize {
        all_params
            .iter()
            .map(|param| param.length + param.thickness)
            .fold(f64::NEG_INFINITY, |a, b| a.max(b))
            .ceil() as usize
    }

    fn generate(alg: &LineAlg, params: &LineParams, grid_size: usize) -> Blocks {
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

    fn bounds(params: &LineParams, pad_factor: f64) -> [[f64; 2]; 2] {
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

    fn show_options(
        &mut self,
        ui: &mut Ui,
        params: &mut LineParams,
        param_fields: &mut LineFields,
        alg: &mut LineAlg,
        parameters_current_layer_control: &mut Control,
        parameters_all_layers_control: &mut Control,
        sampling_points: &ZVec<Vec<f64>>,
        code_enabled: bool,
    ) {
        param_fields.thickness.show(
            &mut params.thickness,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        param_fields.length.show(
            &mut params.length,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );
        ui.separator();
        param_fields.rise.show(
            &mut params.rise,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );
        param_fields.run.show(
            &mut params.run,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

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

    fn draw(plot_ui: &mut PlotUi, params: LineParams, color: Color32) {
        let center = Vec2::from([params.offset_x, params.offset_y]);
        let rr = Vec2::from([params.run, params.rise]).normalize();
        let rr_orth = rr.rot_90_CCW();
        let offset_long = params.length / 2.0 * rr;
        let offset_short = params.thickness / 2.0 * rr_orth;

        // the corners are now +- offset_long +- offset_short (assuming that center=0)
        let x = vec![
            offset_long + offset_short + center,
            offset_long - offset_short + center,
            -offset_long - offset_short + center,
            -offset_long + offset_short + center,
            offset_long + offset_short + center,
        ];
        let corners: Vec<[f64; 2]> = x.iter().map(|pt| pt.as_arr()).collect();

        plot_ui.line(
            egui_plot::Line::new(corners)
                .name("line".to_owned())
                .color(color),
        )
    }

    fn draw_widgets(plot_ui: &mut PlotUi, params: LineParams) {
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

    fn set_parameters(
        &self,
        layer_parameters: &mut LayerParameters<LineAlg, LineParams, LineFields, Self>,
        sampling_points: &Vec<f64>,
        default_shape: &LineParams,
        algorithm: LineAlg,
        fields: &mut LineFields,
    ) where
        Self: Clone + Default,
    {
        layer_parameters.algorithm = algorithm;
        layer_parameters.nr_samples = sampling_points.len();

        layer_parameters.parameters = sampling_points
            .iter()
            .map(|layer| LineParams {
                rise: fields.rise.eval(layer).unwrap_or(default_shape.rise),
                run: fields.run.eval(layer).unwrap_or(default_shape.run),
                offset_x: fields
                    .offset_x
                    .eval(layer)
                    .unwrap_or(default_shape.offset_x),
                offset_y: fields
                    .offset_y
                    .eval(layer)
                    .unwrap_or(default_shape.offset_y),
                thickness: fields
                    .thickness
                    .eval(layer)
                    .unwrap_or(default_shape.thickness),
                length: fields.length.eval(layer).unwrap_or(default_shape.length),
            })
            .collect()
    }

    fn update_slider_parameters(
        &self,
        current_layer_shape: &mut LineParams,
        layer: isize,
        fields: &mut LineFields,
    ) {
        if let Some(rise) = fields.rise.eval(&(layer as f64)) {
            current_layer_shape.rise = rise
        }
        if let Some(run) = fields.run.eval(&(layer as f64)) {
            current_layer_shape.run = run
        }
        if let Some(offset_x) = fields.offset_x.eval(&(layer as f64)) {
            current_layer_shape.offset_x = offset_x
        }
        if let Some(offset_y) = fields.offset_y.eval(&(layer as f64)) {
            current_layer_shape.offset_y = offset_y
        }
        if let Some(thickness) = fields.thickness.eval(&(layer as f64)) {
            current_layer_shape.thickness = thickness
        }
        if let Some(length) = fields.length.eval(&(layer as f64)) {
            current_layer_shape.thickness = length
        }
    }
}
