use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::line::centerpoint::generate_line_centerpoint;
use crate::app::generation::line::line_params::LineParams;
use crate::app::generation::line::LineAlgorithm::Centerpoint;
use crate::app::generation::shape::{Shape, ShapeFields};
use crate::app::param_field::ParamField;
use crate::app::sampling::layer_parameters::LayerParameters;
use eframe::epaint::Color32;
use egui::{Align, Layout, Ui};
use egui_plot::PlotUi;
use std::f64::consts::PI;
use std::fmt::Display;

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
pub enum LineAlgorithm {
    #[default]
    Centerpoint,
}

pub struct LineFields {
    pub rise: ParamField,
    pub run: ParamField,
    pub offset_y: ParamField,
    pub thickness: ParamField,
}

impl Default for LineFields {
    fn default() -> Self {
        Self {
            rise: ParamField::new(true, false, "Rise".to_string(), [-1.0, 1.0], 0.03, vec![]),
            run: ParamField::new(true, false, "Run".to_string(), [-1.0, 1.0], 0.03, vec![]),
            offset_y: ParamField::new(
                true,
                false,
                "y offset".to_string(),
                [-1.0, 1.0],
                0.03,
                vec![
                    ("0".to_string(), 0.0),
                    ("1/2".to_string(), 0.5),
                    ("1/3".to_string(), 1.0 / 3.0),
                    ("2/3".to_string(), 2.0 / 3.0),
                ],
            ),
            thickness: ParamField::new(
                true,
                true,
                "Thickness".to_string(),
                [0.0, 3.0],
                0.03,
                vec![],
            ),
        }
    }
}

impl ShapeFields for LineFields {
    fn all_register_success(&mut self) {
        self.rise.register_success();
        self.run.register_success();
        self.offset_y.register_success();
        self.thickness.register_success();
    }

    fn has_any_changed(&mut self) -> bool {
        self.rise.has_changed()
            || self.run.has_changed()
            || self.offset_y.has_changed()
            || self.thickness.has_changed()
    }
}

impl Shape<LineAlgorithm, LineParams, LineFields> for Line {
    fn describe(alg: &LineAlgorithm) -> String {
        match alg {
            Centerpoint => {
                "Include a particular block iff its centerpoint is in the line".to_string()
            }
        }
    }

    fn name(alg: &LineAlgorithm) -> String {
        match alg {
            Centerpoint => "Centerpoint".to_string(),
        }
    }

    fn all_algs() -> Vec<LineAlgorithm> {
        vec![Centerpoint]
    }

    fn grid_size(all_params: &Vec<LineParams>) -> usize {
        // todo actually estimate
        12
    }

    fn generate(alg: &LineAlgorithm, params: &LineParams, grid_size: usize) -> Blocks {
        match alg {
            Centerpoint => generate_line_centerpoint(
                params.rise,
                params.run,
                params.offset_y,
                params.thickness,
                grid_size,
            ),
        }
    }

    fn bounds(params: &LineParams, pad_factor: f64) -> [[f64; 2]; 2] {
        // todo: make better approximation (maybe possible to be clever about this)
        [[-7.0; 2], [7.0; 2]]
    }

    fn show_options(
        &mut self,
        ui: &mut Ui,
        params: &mut LineParams,
        param_fields: &mut LineFields,
        alg: &mut LineAlgorithm,
        parameters_current_layer_control: &mut Control,
        parameters_all_layers_control: &mut Control,
        sampling_points: &ZVec<Vec<f64>>,
        code_enabled: bool,
    ) {
        // order:
        //  [0] <-> rise
        //  [1] <-> run
        //  [2] <-> offset_y
        //  [3] <-> thickness

        param_fields.rise.show(
            &mut params.rise,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );
        param_fields.rise.show(
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

        param_fields.offset_y.show(
            &mut params.offset_y,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        param_fields.thickness.show(
            &mut params.thickness,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        if param_fields.has_any_changed() {
            parameters_current_layer_control.set_outdated();
            parameters_all_layers_control.set_outdated()
        }
    }

    fn draw(plot_ui: &mut PlotUi, params: LineParams, color: Color32) {
        // todo
    }

    fn draw_widgets(plot_ui: &mut PlotUi, params: LineParams) {
        // todo
    }

    fn set_parameters(
        &self,
        layer_parameters: &mut LayerParameters<LineAlgorithm, LineParams, LineFields, Self>,
        sampling_points: &Vec<f64>,
        default_shape: &LineParams,
        algorithm: LineAlgorithm,
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
                offset_y: fields
                    .offset_y
                    .eval(layer)
                    .unwrap_or(default_shape.offset_y),
                thickness: fields
                    .thickness
                    .eval(layer)
                    .unwrap_or(default_shape.thickness),
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
        if let Some(offset_y) = fields.offset_y.eval(&(layer as f64)) {
            current_layer_shape.offset_y = offset_y
        }
        if let Some(thickness) = fields.thickness.eval(&(layer as f64)) {
            current_layer_shape.thickness = thickness
        }
    }
}
