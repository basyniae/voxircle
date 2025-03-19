use crate::app::colors::{COLOR_RHAI_CHANGED, COLOR_RHAI_INVALID, COLOR_RHAI_RUN_SUCCESS};
use crate::app::data_structures::zvec::ZVec;
use eframe::egui;
use eframe::egui::Ui;
use egui::{Align, Color32, Layout, SliderClamping};
use rhai::{Engine, Scope};

#[derive(Clone, Eq, PartialEq)]
enum FieldState {
    RunSuccess,
    Changed,
    Invalid,
    Empty,
}

pub struct ParamField {
    code: String,
    field_state: FieldState,
    req_finite: bool,
    req_nonnegative: bool,
    name: String,
    slider_range: [f64; 2],
    slider_drag_speed: f64,
    default_buttons: Vec<(String, f64)>,
    /// Maps the slider parameter to the actual parameter value
    param_func: Box<dyn Fn(f64) -> f64>,
    /// Inverse of the above
    inv_param_func: Box<dyn Fn(f64) -> f64>,
}

impl ParamField {
    pub fn new_param_func(
        req_finite: bool,
        req_nonnegative: bool,
        name: String,
        slider_range: [f64; 2],
        slider_drag_speed: f64,
        default_buttons: Vec<(String, f64)>,
        param_fun: impl Fn(f64) -> f64 + 'static,
        inv_param_fun: impl Fn(f64) -> f64 + 'static,
    ) -> Self {
        Self {
            code: "".parse().unwrap(),
            field_state: FieldState::Empty,
            req_finite,
            req_nonnegative,
            name,
            slider_range,
            slider_drag_speed,
            default_buttons,
            param_func: Box::new(param_fun),
            inv_param_func: Box::new(inv_param_fun),
        }
    }

    pub fn new(
        req_finite: bool,
        req_nonnegative: bool,
        name: String,
        slider_range: [f64; 2],
        slider_drag_speed: f64,
        default_buttons: Vec<(String, f64)>,
    ) -> Self {
        Self {
            code: "".parse().unwrap(),
            field_state: FieldState::Empty,
            req_finite,
            req_nonnegative,
            name,
            slider_range,
            slider_drag_speed,
            default_buttons,
            param_func: Box::new(|x| x),
            inv_param_func: Box::new(|x| x),
        }
    }

    pub fn show(
        &mut self,
        param: &mut f64,
        ui: &mut Ui,
        code_enabled: &bool,
        sampling_points: &ZVec<Vec<f64>>,
        changed: &mut bool,
        name_override: Option<&String>,
    ) {
        // Slider
        {
            // Read the value of the slider parameter from the old parameter value
            let mut slider_value = (self.inv_param_func)(*param);
            if ui
                .add(
                    egui::Slider::new(
                        &mut slider_value,
                        self.slider_range[0]..=self.slider_range[1],
                    )
                    .text(name_override.unwrap_or(&self.name))
                    .drag_value_speed(self.slider_drag_speed)
                    // How the slider value is displayed (x is in slider space, the output is in
                    // parameter space)
                    .custom_formatter(|x, _| format!("{:.02}", (self.param_func)(x)))
                    // How a text-input value should be interpreted as a slider parameter
                    // s is (a string) in parameter space, t is (a f64) in parameter space,
                    //  the output is in slider space
                    .custom_parser(|s| s.parse::<f64>().map(|t| (self.inv_param_func)(t)).ok())
                    .clamping(SliderClamping::Never),
                )
                .changed()
            {
                self.update_field_state(sampling_points);
                *changed = true;
            };
            // Write the parameter value coming from the slider
            *param = (self.param_func)(slider_value);
        }

        // Default values
        ui.allocate_ui_with_layout(
            egui::Vec2::from([100.0, 200.0]),
            Layout::left_to_right(Align::Min),
            |ui| {
                // todo: think about this clone a bit (performance should be fine though)
                for (name, value) in self.default_buttons.clone().iter() {
                    if ui.button(name).clicked() {
                        *param = value.clone();
                        self.update_field_state(sampling_points);
                        *changed = true
                    };
                }
            },
        );

        // Code field
        if *code_enabled {
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
        let engine = Engine::new();
        let mut scope = Scope::new();
        let code = self.code.clone();

        // Check if the expression is valid at all sampling points. First unpack layers, then unpack
        // sampling points
        sampling_points
            .data
            .clone()
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
