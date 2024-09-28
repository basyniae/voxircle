use crate::app::data_structures::zvec::ZVec;
use eframe::egui;
use eframe::egui::{Color32, Ui};
use mlua::Lua;

#[derive(Clone, Eq, PartialEq)]
enum FieldState {
    RunSuccess,
    Changed,
    Invalid,
    Empty,
}

pub struct LuaField {
    code: String,
    field_state: FieldState,
    req_finite: bool,
    req_nonnegative: bool,
}

impl LuaField {
    pub fn new(req_finite: bool, req_nonnegative: bool) -> Self {
        Self {
            code: "".parse().unwrap(),
            field_state: FieldState::Empty,
            req_finite,
            req_nonnegative,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, lua: &mut Lua, sampling_points: &ZVec<Vec<f64>>) {
        let original_style = ui.style().clone();

        match self.field_state {
            FieldState::RunSuccess => {
                ui.visuals_mut().extreme_bg_color = Color32::LIGHT_GREEN;
                ui.visuals_mut().override_text_color = Some(Color32::BLACK)
            }
            FieldState::Changed => {
                ui.visuals_mut().extreme_bg_color = Color32::from_rgb(227, 197, 103);
                ui.visuals_mut().override_text_color = Some(Color32::BLACK)
            }
            FieldState::Invalid => {
                ui.visuals_mut().extreme_bg_color = Color32::LIGHT_RED;
                ui.visuals_mut().override_text_color = Some(Color32::BLACK)
            }
            FieldState::Empty => {}
        }

        let response = ui.add(egui::TextEdit::singleline(&mut self.code).code_editor());
        if response.changed() {
            self.update_field_state(lua, sampling_points);
        }

        ui.set_style(original_style);
    }

    pub fn update_field_state(&mut self, lua: &mut Lua, sampling_points: &ZVec<Vec<f64>>) {
        if self.code.is_empty() {
            self.field_state = FieldState::Empty
        } else if !self.is_valid_expression(lua, sampling_points) {
            self.field_state = FieldState::Invalid
        } else {
            self.field_state = FieldState::Changed
        }
    }

    pub fn is_valid_expression(&self, lua: &mut Lua, sampling_points: &ZVec<Vec<f64>>) -> bool {
        // Check if the expression is valid at all sampling points. First unpack layers, then unpack
        // sampling points
        sampling_points
            .data
            .iter()
            .map(|layer| {
                layer
                    .iter()
                    .map(|sample| {
                        // is the expression valid for this sample?
                        lua.globals().set("layer", *sample).unwrap();
                        lua.globals().set("l", *sample).unwrap();
                        lua.load(self.code.clone()).eval::<f64>().is_ok_and(|x| {
                            !x.is_nan()
                                && (!self.req_finite || x.is_finite())
                                && (!self.req_nonnegative || x >= 0.0)
                        })
                    })
                    // is the expression valid for this particular layer
                    .fold(true, |a, b| a && b)
            })
            // is the expression valid for all layers?
            .fold(true, |a, b| a && b)
    }

    pub fn eval(&mut self, lua: &mut Lua) -> Option<f64> {
        // Only change the parameter if the code is valid and has changed
        if self.field_state == FieldState::Changed {
            let parameter = lua.load(self.code.clone()).eval().unwrap();
            self.field_state = FieldState::RunSuccess;
            // Simplest to register success here
            Some(parameter)
        } else {
            None
        }
    }
}
