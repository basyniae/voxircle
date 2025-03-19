use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::line::line_params::LineParams;
use crate::app::generation::line::{Line, LineAlg, LineFields};
use crate::app::generation::squircle::squircle_params::SquircleParams;
use crate::app::generation::squircle::{Squircle, SquircleAlg, SquircleFields};
use crate::app::math::linear_algebra::Vec2;
use crate::app::param_config::ParamConfig;
use crate::app::plotting;
use crate::app::sampling::layer_parameters::LayerParameters;
use egui::{Color32, Ui};
use egui_plot::PlotUi;
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone, Debug)]
pub enum AllParams {
    Null,
    Squircle(SquircleParams),
    Line(LineParams),
}

pub enum AllFields {
    Null,
    Squircle(SquircleFields),
    Line(LineFields),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AllAlgs {
    Null,
    Squircle(SquircleAlg),
    Line(LineAlg),
}

// impl Debug for AllParams {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{}",
//             match self {
//                 Self::Null => {
//                     "Null"
//                 }
//                 Self::Squircle(_) => {
//                     "Squircle"
//                 }
//                 Self::Line(_) => {
//                     "Line"
//                 }
//             }
//         )
//     }
// }

impl Debug for AllFields {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Null => {
                    "Null"
                }
                Self::Squircle(_) => {
                    "Squircle"
                }
                Self::Line(_) => {
                    "Line"
                }
            }
        )
    }
}

// impl Debug for AllAlgs {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{}",
//             match self {
//                 Self::Null => {
//                     "Null"
//                 }
//                 Self::Squircle(_) => {
//                     "Squircle"
//                 }
//                 Self::Line(_) => {
//                     "Line"
//                 }
//             }
//         )
//     }
// }

impl AllParams {
    pub fn draw(&self, plot_ui: &mut PlotUi, color: Color32) {
        match self {
            AllParams::Null => {
                panic!("Impossible")
            }
            AllParams::Squircle(params) => {
                plot_ui.line(plotting::superellipse_at_coords(params.clone()).color(color))
            }
            AllParams::Line(params) => {
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
        }
    }

    pub fn draw_widgets(&self, plot_ui: &mut PlotUi) {
        match self {
            AllParams::Null => panic!("impossible"),
            AllParams::Squircle(params) => Squircle::draw_widgets(plot_ui, params),
            AllParams::Line(params) => Line::draw_widgets(plot_ui, params),
        }
    }

    pub fn bounds(&self, pad_factor: f64) -> [[f64; 2]; 2] {
        match self {
            AllParams::Null => panic!("impossible"),
            AllParams::Squircle(params) => Squircle::bounds(params, pad_factor),
            AllParams::Line(params) => Line::bounds(params, pad_factor),
        }
    }

    pub fn grid_size(layer_params: &Vec<Self>) -> usize {
        match layer_params[0] {
            AllParams::Null => panic!("impossible"),
            AllParams::Squircle(_) => {
                let mut running = vec![];
                for i in layer_params {
                    if let AllParams::Squircle(i) = i {
                        running.push(i)
                    } else {
                        panic!("grid_size called with arguments that do not match")
                    }
                }
                Squircle::grid_size(running)
            }
            AllParams::Line(_) => {
                let mut running = vec![];
                for i in layer_params {
                    if let AllParams::Line(i) = i {
                        running.push(i)
                    } else {
                        panic!("grid_size called with arguments that do not match")
                    }
                }
                Line::grid_size(running)
            }
        }
    }
}

impl AllFields {
    // there is probably some way to automate writing these definitions but i don't care for it at this moment
    pub(crate) fn all_register_success(&mut self) {
        match self {
            Self::Null => panic!("impossible"),
            AllFields::Squircle(squircle) => {
                squircle.radius_a.register_success();
                squircle.radius_b.register_success();
                squircle.tilt.register_success();
                squircle.offset_x.register_success();
                squircle.offset_y.register_success();
                squircle.squircle_parameter.register_success();
            }
            AllFields::Line(line) => {
                line.rise.register_success();
                line.run.register_success();
                line.offset_x.register_success();
                line.offset_y.register_success();
                line.thickness.register_success();
                line.length.register_success()
            }
        }
    }

    // todo: check usages
    pub(crate) fn has_any_changed(&self) -> bool {
        match self {
            Self::Null => panic!("impossible"),
            AllFields::Squircle(fields) => {
                fields.radius_a.has_changed()
                    || fields.radius_b.has_changed()
                    || fields.tilt.has_changed()
                    || fields.offset_x.has_changed()
                    || fields.offset_y.has_changed()
                    || fields.squircle_parameter.has_changed()
            }
            AllFields::Line(fields) => {
                fields.rise.has_changed()
                    || fields.run.has_changed()
                    || fields.offset_x.has_changed()
                    || fields.offset_y.has_changed()
                    || fields.thickness.has_changed()
                    || fields.length.has_changed()
            }
        }
    }
}

impl AllAlgs {
    pub(crate) fn describe(&self) -> String {
        match self {
            Self::Null => panic!("impossible"),
            AllAlgs::Squircle(alg) => {match alg {
                SquircleAlg::Centerpoint => {"Include a particular block iff its centerpoint is in the ellipse".to_string()}
                SquircleAlg::Conservative => {"Include a particular block in the voxelization iff it has nonempty intersection with the ellipse".to_string()}
                SquircleAlg::Contained => {"Include a particular block iff it is fully contained in the ellipse".to_string()}
                SquircleAlg::Percentage(percentage) => {format!(
                    "Include a particular block in the voxelization iff more than {:.0}% of it is contained in the circle. Ellipses and squircles not implemented.",
                    100.0 * percentage
                )}
                SquircleAlg::Empty => {"Include no blocks in the voxelization".to_string()}
            }}
            AllAlgs::Line(alg) => {match alg {
                LineAlg::Centerpoint => {
                    "Include a particular block iff its centerpoint is in the line".to_string()
                }
            }}
        }
    }

    pub(crate) fn name(&self) -> String {
        match self {
            Self::Null => panic!("impossible"),
            AllAlgs::Squircle(alg) => match alg {
                SquircleAlg::Centerpoint => "Centerpoint".to_string(),
                SquircleAlg::Conservative => "Conservative".to_string(),
                SquircleAlg::Contained => "Contained".to_string(),
                SquircleAlg::Percentage(_) => "Percentage".to_string(),
                SquircleAlg::Empty => "Empty".to_string(),
            },
            AllAlgs::Line(alg) => match alg {
                LineAlg::Centerpoint => "Centerpoint".to_string(),
            },
        }
    }
}

pub fn ui_show_options(
    ui: &mut Ui,
    alg: &mut AllAlgs,
    params: &mut AllParams,
    fields: &mut AllFields,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
    sampling_points: &ZVec<Vec<f64>>,
    code_enabled: bool,
    param_config: &mut ParamConfig,
) {
    // Match first over the algorithm so that type is checked by the compiler at least a little bit
    //  (we will get a reminder to add a new entry in here if we add a variant to the AllAlgs struct)
    match alg {
        AllAlgs::Null => panic!("impossible"),
        AllAlgs::Squircle(alg) => {
            if let (AllParams::Squircle(params), AllFields::Squircle(fields)) = (params, fields) {
                Squircle::show_options(
                    ui,
                    params,
                    fields,
                    alg,
                    parameters_current_layer_control,
                    parameters_all_layers_control,
                    sampling_points,
                    code_enabled,
                    param_config,
                )
            } else {
                panic!("ui_show_options called with arguments that do not match or have not been implemented")
            }
        }
        AllAlgs::Line(alg) => {
            if let (AllParams::Line(params), AllFields::Line(fields)) = (params, fields) {
                Line::show_options(
                    ui,
                    params,
                    fields,
                    alg,
                    parameters_current_layer_control,
                    parameters_all_layers_control,
                    sampling_points,
                    code_enabled,
                )
            } else {
                panic!("ui_show_options called with arguments that do not match or have not been implemented")
            }
        }
    }
}

pub fn generate(alg: &AllAlgs, params: &AllParams, grid_size: usize) -> Blocks {
    match alg {
        AllAlgs::Null => panic!("impossible"),
        AllAlgs::Squircle(alg) => {
            if let AllParams::Squircle(params) = params {
                Squircle::generate(alg, params, grid_size)
            } else {
                panic!(
                    "generate called with arguments that do not match or have not been implemented"
                )
            }
        }
        AllAlgs::Line(alg) => {
            if let AllParams::Line(params) = params {
                Line::generate(alg, params, grid_size)
            } else {
                panic!(
                    "generate called with arguments that do not match or have not been implemented"
                )
            }
        }
    }
}

pub fn set_parameters(
    layer_parameters: &mut LayerParameters,
    sampling_points: &Vec<f64>,
    default_shape: &AllParams,
    alg: &AllAlgs,
    fields: &mut AllFields,
    param_config: &ParamConfig,
) {
    match alg {
        AllAlgs::Null => panic!("impossible"),
        AllAlgs::Squircle(alg) => {
            if let (AllParams::Squircle(default_shape), AllFields::Squircle(fields)) =
                (default_shape, fields)
            {
                Squircle::set_parameters(
                    layer_parameters,
                    sampling_points,
                    default_shape,
                    alg,
                    fields,
                    param_config,
                )
            } else {
                panic!("set_parameters called with arguments that do not match")
            }
        }
        AllAlgs::Line(alg) => {
            if let (AllParams::Line(default_shape), AllFields::Line(fields)) =
                (default_shape, fields)
            {
                Line::set_parameters(
                    layer_parameters,
                    sampling_points,
                    default_shape,
                    alg,
                    fields,
                )
            } else {
                panic!("set_parameters called with arguments that do not match")
            }
        }
    }
}

pub fn update_slider_parameters(
    current_layer_shape: &mut AllParams,
    layer: isize,
    fields: &mut AllFields,
    param_config: &ParamConfig,
) {
    match current_layer_shape {
        AllParams::Null => panic!("impossible"),
        AllParams::Squircle(current_layer_shape) => {
            if let AllFields::Squircle(fields) = fields {
                Squircle::update_slider_parameters(current_layer_shape, layer, fields, param_config)
            } else {
                panic!("update_slider_parameters called with arguments that do not match")
            }
        }
        AllParams::Line(current_layer_shape) => {
            if let AllFields::Line(fields) = fields {
                Line::update_slider_parameters(current_layer_shape, layer, fields)
            } else {
                panic!("update_slider_parameters called with arguments that do not match")
            }
        }
    }
}
