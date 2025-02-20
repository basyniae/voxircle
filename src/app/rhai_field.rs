use crate::app::colors::{COLOR_RHAI_CHANGED, COLOR_RHAI_INVALID, COLOR_RHAI_RUN_SUCCESS};
use crate::app::data_structures::zvec::ZVec;
use eframe::egui;
use eframe::egui::Ui;
use egui::{Align, Color32, Layout};
use rhai::{Engine, Scope};

#[derive(Clone, Eq, PartialEq)]
enum FieldState {
    RunSuccess,
    Changed,
    Invalid,
    Empty,
}

pub struct RhaiField {
    code: String,
    field_state: FieldState,
    req_finite: bool,
    req_nonnegative: bool,
}

impl RhaiField {
    pub fn new(req_finite: bool, req_nonnegative: bool) -> Self {
        Self {
            code: "".parse().unwrap(),
            field_state: FieldState::Empty,
            req_finite,
            req_nonnegative,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, sampling_points: &ZVec<Vec<f64>>) {
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            // code text field
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.code)
                    .code_editor()
                    .background_color(match self.field_state {
                        FieldState::RunSuccess => COLOR_RHAI_RUN_SUCCESS,
                        FieldState::Changed => COLOR_RHAI_CHANGED,
                        FieldState::Invalid => COLOR_RHAI_INVALID,
                        FieldState::Empty => ui.style().visuals.extreme_bg_color,
                    })
                    .text_color(Color32::BLACK), // black text for readability on light backgrounds
            );
            if response.changed() {
                self.update_field_state(sampling_points);
            }

            // clear code field button
            if ui
                .add_enabled(!self.code.is_empty(), egui::Button::new("🗙"))
                .clicked()
            {
                self.code = "".to_string();
                self.update_field_state(sampling_points);
            }
        });
    }

    pub fn update_field_state(&mut self, sampling_points: &ZVec<Vec<f64>>) {
        if self.code.is_empty() {
            self.field_state = FieldState::Empty
        } else if !self.is_valid_expression(sampling_points) {
            self.field_state = FieldState::Invalid
        } else {
            self.field_state = FieldState::Changed
        }
    }

    pub fn is_valid_expression(&self, sampling_points: &ZVec<Vec<f64>>) -> bool {
        let mut engine = Engine::new();
        let mut scope = Scope::new();
        let code = self.code.clone();

        // Check if the expression is valid at all sampling points. First unpack layers, then unpack
        // sampling points
        sampling_points
            .clone()
            .data
            .into_iter()
            .map(|layer| {
                layer
                    .into_iter()
                    .map(|sample| {
                        // is the expression valid for this sample?

                        scope.push_constant("layer", sample);
                        scope.push_constant("l", sample);

                        engine
                            .eval_expression_with_scope(&mut scope, &code)
                            .is_ok_and(|x: f64| {
                                !x.is_nan()
                                    && (!self.req_finite || x.is_finite())
                                    && (!self.req_nonnegative || x >= 0.0)
                            })
                    })
                    // is the expression valid for this particular layer?
                    .all(|a| a)
            })
            .all(|a| a) // is it finally valid for all layers
    }

    pub fn eval(&mut self, sample: &f64) -> Option<f64> {
        // Only change the parameter if the code is valid and has changed
        if self.field_state == FieldState::Changed || self.field_state == FieldState::RunSuccess {
            let engine = Engine::new();
            let mut scope = Scope::new();

            scope.push_constant("layer", *sample);
            scope.push_constant("l", *sample);

            let parameter = engine
                .eval_expression_with_scope(&mut scope, &self.code)
                .unwrap();
            Some(parameter)
        } else {
            None
        }
    }

    /// Has the field changed (since the last time it was run) to a valid expression?
    pub fn has_changed(&self) -> bool {
        self.field_state == FieldState::Changed
    }

    pub fn register_success(&mut self) {
        if self.field_state == FieldState::Changed {
            self.field_state = FieldState::RunSuccess;
        }
    }
}
