use std::f64::consts::PI;
use std::f64::consts::TAU;

use crate::app::control::Control;
use crate::app::data_structures::slice_parameters::SliceParameters;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::Algorithm;
use crate::app::rhai_field::RhaiField;
use eframe::egui;
use eframe::egui::{Align, Layout, Ui};
use egui::SliderClamping;

// my first macro!
// todo: there should be a way to simplify this further, there's still a lot of repetition in the code
//  (esp. if you take into account the other ui files also)
/// Mark the inputted control variables as outdated
macro_rules! outdate {
    ( $( $x:expr ),+ ) => {{
        $($x.set_outdated();)+
    }};
}

/// Draw ui for algorithm selection, parameters of the generation (radius etc.), and rhai fields.
/// Update
pub fn ui_options(
    ui: &mut Ui,
    current_layer_config: &mut SliceParameters,
    single_radius: &mut bool,
    code_enabled: bool,
    rhai_field_radius_a: &mut RhaiField,
    rhai_field_radius_b: &mut RhaiField,
    rhai_field_tilt: &mut RhaiField,
    rhai_field_center_offset_x: &mut RhaiField,
    rhai_field_center_offset_y: &mut RhaiField,
    rhai_field_squircle_parameter: &mut RhaiField,
    sampling_points: &ZVec<Vec<f64>>,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
) {
    // TODO: easily change algorithm for all layers
    // Select algorithm (the storage is for checking changed(), this is necessary
    //  as https://github.com/emilk/egui/discussions/923)
    let old_alg = current_layer_config.algorithm.clone();
    Algorithm::combo_box(ui, &mut current_layer_config.algorithm);
    if old_alg != current_layer_config.algorithm {
        outdate!(
            parameters_current_layer_control,
            parameters_all_layers_control
        );
    }

    // algorithm description
    ui.label(current_layer_config.algorithm.describe());

    // algorithm-specific options
    match current_layer_config.algorithm {
        Algorithm::Percentage(percentage) => {
            let mut perc_slider = percentage.clone();
            if ui
                .add(
                    egui::Slider::new(&mut perc_slider, 0.0..=1.0)
                        .text("")
                        .fixed_decimals(2)
                        .custom_formatter(|n, _| {
                            format!("{:.0}%", n * 100.0) //  formatting of percentage slider
                        }),
                )
                .changed()
            {
                current_layer_config.algorithm = Algorithm::Percentage(perc_slider);
            };
        }
        _ => {}
    }

    // Radius
    ui.separator();

    ui.checkbox(single_radius, "Single radius");

    if *single_radius {
        if ui
            .add(
                egui::Slider::new(&mut current_layer_config.radius_a, 0.0..=30.0)
                    .text("Radius")
                    .clamping(SliderClamping::Never)
                    .custom_formatter(|param, _| format!("{:.02}", param))
                    .drag_value_speed(0.03),
            )
            .changed()
        {
            // the code is now invalid
            rhai_field_radius_a.update_field_state(sampling_points);
            rhai_field_radius_b.update_field_state(sampling_points);

            outdate!(
                parameters_current_layer_control,
                parameters_all_layers_control
            );
        };

        // rhai
        if code_enabled {
            rhai_field_radius_a.show(ui, sampling_points);
        }

        current_layer_config.radius_b = current_layer_config.radius_a;
    } else {
        // radius a
        if ui
            .add(
                egui::Slider::new(&mut current_layer_config.radius_a, 0.0..=30.0)
                    .text("Radius A")
                    .clamping(SliderClamping::Never)
                    .custom_formatter(|param, _| format!("{:.02}", param))
                    .drag_value_speed(0.03),
            )
            .changed()
        {
            rhai_field_radius_a.update_field_state(sampling_points);
            outdate!(
                parameters_current_layer_control,
                parameters_all_layers_control
            );
        }
        if code_enabled {
            rhai_field_radius_a.show(ui, sampling_points);
        }

        // radius b
        if ui
            .add(
                egui::Slider::new(&mut current_layer_config.radius_b, 0.0..=30.0)
                    .text("Radius B")
                    .clamping(SliderClamping::Never)
                    .custom_formatter(|param, _| format!("{:.02}", param))
                    .drag_value_speed(0.03),
            )
            .changed()
        {
            rhai_field_radius_b.update_field_state(sampling_points);
            outdate!(
                parameters_current_layer_control,
                parameters_all_layers_control
            );
        }
        if code_enabled {
            rhai_field_radius_b.show(ui, sampling_points);
        }

        //longterm: Make circular slider for more intuitive controls (need to build this myapp probably)
    }

    //tilt
    if ui
        .add(
            egui::Slider::new(&mut current_layer_config.tilt, -TAU..=TAU)
                .text("Tilt (radians)")
                .fixed_decimals(2)
                .drag_value_speed(0.01),
        )
        .changed()
    {
        rhai_field_tilt.update_field_state(sampling_points);
        outdate!(
            parameters_current_layer_control,
            parameters_all_layers_control
        );
    };

    // Particular values
    ui.allocate_ui_with_layout(
        egui::Vec2::from([100.0, 200.0]),
        Layout::left_to_right(Align::Min),
        |ui| {
            [
                ("0°", 0.0),
                ("30°", PI / 6.0),
                ("45°", PI / 4.0),
                ("1:2", 0.5_f64.atan()),
                ("1:3", 0.33333333333333_f64.atan()),
                ("2:3", 0.66666666666666_f64.atan()),
                ("1:4", 0.25_f64.atan()),
            ]
            .map(|(name, value)| {
                if ui.button(name).clicked() {
                    current_layer_config.tilt = value;
                    rhai_field_tilt.update_field_state(sampling_points);
                    outdate!(
                        parameters_current_layer_control,
                        parameters_all_layers_control
                    )
                }
            });
        },
    );
    if code_enabled {
        rhai_field_tilt.show(ui, sampling_points);
    }

    // Squircle parameter
    // due to the scale of the parameter this is all a bit awkward... Introduce a temporary variable for controlling it
    {
        let mut squircle_ui_parameter = current_layer_config.get_squircle_ui_parameter();
        ui.separator();
        if ui
            .add(
                egui::Slider::new(&mut squircle_ui_parameter, 0.0..=1.0)
                    .text("Squircicity")
                    .custom_formatter(|param, _| format!("{:.02}", 1.0 / (1.0 - param) - 1.0))
                    .custom_parser(|s| s.parse::<f64>().map(|t| 1.0 - 1.0 / (t + 1.0)).ok()),
            )
            .changed()
        {
            rhai_field_squircle_parameter.update_field_state(sampling_points);
            outdate!(
                parameters_current_layer_control,
                parameters_all_layers_control
            );
        };

        // Default values

        // Aim: Make choice of squircle parameter easy. there are distinct values at 2/3 and 1/3 we want to be exact
        // From the implementation, squircle_parameter = 1.0 / (1.0 - squircle_ui_parameter) - 1.0,
        //  so squircle_ui_parameter = squircle_parameter / (squircle_parameter + 1.0).
        ui.allocate_ui_with_layout(
            egui::Vec2::from([100.0, 200.0]),
            Layout::left_to_right(Align::Min),
            |ui| {
                [
                    ("Circle", 0.66666666666666), // Squircle parameter 2
                    ("Astroid", 0.4),             // "" "" 2/3
                    ("Diamond", 0.5),             // "" "" 1
                    ("Square", 1.0),              // "" "" infinity
                ]
                .map(|(name, value)| {
                    if ui.button(name).clicked() {
                        squircle_ui_parameter = value;
                        rhai_field_squircle_parameter.update_field_state(sampling_points);
                        outdate!(
                            parameters_current_layer_control,
                            parameters_all_layers_control
                        );
                    }
                });
            },
        );
        current_layer_config.squircle_parameter = 1.0 / (1.0 - squircle_ui_parameter) - 1.0;
    }
    // now kill the temporary variable
    if code_enabled {
        rhai_field_squircle_parameter.show(ui, sampling_points);
    }

    // Centerpoint
    ui.separator();
    if ui
        .add(
            egui::Slider::new(&mut current_layer_config.center_offset_x, -1.0..=1.0)
                .text("x offset")
                .clamping(SliderClamping::Never),
        )
        .changed()
    {
        rhai_field_center_offset_x.update_field_state(sampling_points);
        outdate!(
            parameters_current_layer_control,
            parameters_all_layers_control
        );
    };
    if code_enabled {
        rhai_field_center_offset_x.show(ui, sampling_points);
    }

    if ui
        .add(
            egui::Slider::new(&mut current_layer_config.center_offset_y, -1.0..=1.0)
                .text("y offset")
                .clamping(SliderClamping::Never),
        )
        .changed()
    {
        rhai_field_center_offset_y.update_field_state(sampling_points);
        outdate!(
            parameters_current_layer_control,
            parameters_all_layers_control
        );
    };
    if code_enabled {
        rhai_field_center_offset_y.show(ui, sampling_points);
    }

    // Add odd and even buttons (also good so people understand what the abstraction "offset center" actually means)
    ui.allocate_ui_with_layout(
        egui::Vec2::from([100.0, 200.0]),
        Layout::left_to_right(Align::Min),
        |ui| {
            [("Even center", 0.0, 0.0), ("Odd center", 0.5, 0.5)].map(|(name, x, y)| {
                if ui.button(name).clicked() {
                    current_layer_config.center_offset_x = x;
                    current_layer_config.center_offset_y = y;
                    outdate!(
                        parameters_current_layer_control,
                        parameters_all_layers_control
                    )
                }
            });
        },
    );

    if rhai_field_radius_a.has_changed()
        || rhai_field_radius_b.has_changed()
        || rhai_field_tilt.has_changed()
        || rhai_field_center_offset_x.has_changed()
        || rhai_field_center_offset_y.has_changed()
        || rhai_field_squircle_parameter.has_changed()
    {
        outdate!(
            parameters_current_layer_control,
            parameters_all_layers_control
        );
    }
}
