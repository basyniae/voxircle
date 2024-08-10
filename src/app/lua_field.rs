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

    pub fn show(&mut self, ui: &mut Ui, lua: &mut Lua, layer_min: isize, layer_max: isize) {
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
            self.update_field_state(lua, layer_min, layer_max);
        }

        ui.set_style(original_style);
    }

    pub fn update_field_state(&mut self, lua: &mut Lua, layer_min: isize, layer_max: isize) {
        if self.code.is_empty() {
            self.field_state = FieldState::Empty
        } else if !self.is_valid_expression(lua, layer_min, layer_max) {
            self.field_state = FieldState::Invalid
        } else {
            self.field_state = FieldState::Changed
        }
    }

    pub fn is_valid_expression(&self, lua: &mut Lua, layer_min: isize, layer_max: isize) -> bool {
        let mut running = true;
        for layer in layer_min..=layer_max {
            lua.globals().set("layer", layer).unwrap();
            running &= lua.load(self.code.clone()).eval::<f64>().is_ok_and(|x| {
                !x.is_nan()
                    && (!self.req_finite || x.is_finite())
                    && (!self.req_nonnegative || x >= 0.0)
            });
        }
        running
    }

    pub fn eval(&mut self, lua: &mut Lua, parameter: &mut f64) {
        // Only change the parameter if the code is valid and has changed
        if self.field_state == FieldState::Changed {
            *parameter = lua.load(self.code.clone()).eval().unwrap();
        }
    }

    pub fn set_success(&mut self) {
        self.field_state = FieldState::RunSuccess;
    }
}
