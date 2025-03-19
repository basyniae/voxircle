use crate::app::data_structures::blocks::Blocks;
use crate::app::generation::line::line_params::LineParams;
use crate::app::generation::line::{Line, LineAlg, LineFields};
use crate::app::generation::shape_type::ShapeType;
use crate::app::generation::squircle::squircle_params::SquircleParams;
use crate::app::generation::squircle::{Squircle, SquircleAlg, SquircleFields};
use crate::app::math::linear_algebra::Vec2;
use crate::app::plotting;
use eframe::epaint::Color32;
use egui::Ui;
use egui_plot::PlotUi;
use std::fmt::{Debug, Formatter};

pub mod line;
pub mod shape_type;
pub mod squircle;

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
    pub fn all_register_success(&mut self) {
        match self {
            Self::Null => panic!("impossible"),
            AllFields::Squircle(squircle) => {
                for field in squircle.all_fields_mut() {
                    field.register_success()
                }
            }
            AllFields::Line(line) => {
                for field in line.all_fields_mut() {
                    field.register_success()
                }
            }
        }
    }
}

impl AllAlgs {
    pub fn describe(&self) -> String {
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

    pub fn name(&self) -> String {
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

pub fn algorithm_combobox(ui: &mut Ui, current_layer_alg: &mut AllAlgs, shape_type: &ShapeType) {
    egui::ComboBox::from_label("Algorithm")
        .selected_text(format!("{:}", current_layer_alg.name()))
        .show_ui(ui, |ui| {
            for i in match shape_type {
                ShapeType::Squircle => SquircleAlg::all_algs(),
                ShapeType::Line => LineAlg::all_algs(),
            } {
                ui.selectable_value(current_layer_alg, i, i.name());
            }
        });
}
