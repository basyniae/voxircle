use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::line::centerpoint::generate_line_centerpoint;
use crate::app::generation::line::LineAlgorithm::Centerpoint;
use crate::app::generation::shape::Shape;
use crate::app::param_field::ParamField;
use eframe::emath::Align;
use egui::{Layout, Ui};
use line_params::LineParams;
use std::f64::consts::PI;
use std::fmt::Display;

mod centerpoint;
mod line_params;

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

impl Shape<LineAlgorithm, LineParams> for Line {
    fn describe(alg: &LineAlgorithm) -> String {
        match alg {
            Centerpoint => {
                "Include a particular block iff its centerpoint is in the line".to_string()
            }
        }
    }

    fn name(alg: &LineAlgorithm) -> String {
        match alg {
            LineAlgorithm::Centerpoint => "Centerpoint".to_string(),
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

    fn get_new_param_fields() -> Vec<ParamField> {
        vec![
            ParamField::new(true, false, "Rise".to_string(), [-1.0, 1.0], 0.03, vec![]),
            ParamField::new(true, false, "Run".to_string(), [-1.0, 1.0], 0.03, vec![]),
            ParamField::new(
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
            ParamField::new(
                true,
                true,
                "Thickness".to_string(),
                [0.0, 3.0],
                0.03,
                vec![],
            ),
        ]
    }

    fn bounds(params: &LineParams, pad_factor: f64) -> [[f64; 2]; 2] {
        // todo: make better approximation (maybe possible to be clever about this)
        [[-7.0; 2], [7.0; 2]]
    }

    fn show_options(
        &mut self,
        ui: &mut Ui,
        params: &mut LineParams,
        param_fields: &mut Vec<ParamField>,
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

        param_fields[0].show(
            &mut params.rise,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );
        param_fields[1].show(
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

        param_fields[2].show(
            &mut params.offset_y,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        param_fields[3].show(
            &mut params.thickness,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        if param_fields[0].has_changed()
            || param_fields[1].has_changed()
            || param_fields[2].has_changed()
            || param_fields[3].has_changed()
        {
            parameters_current_layer_control.set_outdated();
            parameters_all_layers_control.set_outdated()
        }
    }
}
