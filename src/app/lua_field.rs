use crate::app::data_structures::zvec::ZVec;
use eframe::egui;
use eframe::egui::{Color32, Ui};
use rhai::{Engine, EvalAltResult, run, Scope};

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

    // TODO: button to clear the lua field
    pub fn show(&mut self, ui: &mut Ui, sampling_points: &ZVec<Vec<f64>>) {
        let original_style = ui.style().clone();

        match self.field_state {
            FieldState::RunSuccess => {
                ui.visuals_mut().extreme_bg_color = Color32::LIGHT_GREEN;
                ui.visuals_mut().override_text_color = Some(Color32::BLACK)
            }
            FieldState::Changed => {
                ui.visuals_mut().extreme_bg_color = Color32::from_rgb(227, 197, 103); //todo: make color
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
            self.update_field_state(sampling_points);
        }

        ui.set_style(original_style);
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
        sampling_points.clone().data.into_iter().map(|layer| {
            layer
                .into_iter()
                .map(|sample| {
                    // is the expression valid for this sample?

                    scope.push_constant("layer", sample);
                    scope.push_constant("l", sample);

                    println!("l = {}", sample.to_string());
                    engine.eval_expression_with_scope(&mut scope, &code).is_ok_and(|x: f64| {
                        !x.is_nan()
                            && (!self.req_finite || x.is_finite())
                            && (!self.req_nonnegative || x >= 0.0)
                    })
                })
                // is the expression valid for this particular layer?
                .fold(true, |a, b| a && b)
        }).fold(true, |a, b| a && b)
    }


    pub fn eval(&mut self, sample: &f64) -> Option<f64> {
        // Only change the parameter if the code is valid and has changed
        // longterm: should not rerun code if there has been a success (assuming layer hasn't changed)
        if self.field_state == FieldState::Changed || self.field_state == FieldState::RunSuccess {
            let mut engine = Engine::new();
            let mut scope = Scope::new();

            scope.push_constant("layer", sample.clone());
            scope.push_constant("l", sample.clone());

            let parameter = engine.eval_expression_with_scope(&mut scope, &*self.code).unwrap();
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
