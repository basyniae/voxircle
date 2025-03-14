use crate::app::control::Control;
use crate::app::data_structures::squircle_params::SquircleParams;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::squircle::SquircleAlgorithm;
use crate::app::param_field::ParamField;
use eframe::egui;
use eframe::egui::{Align, Layout, Ui};

// my first macro!
/// Mark the inputted control variables as outdated
macro_rules! outdate {
    ( $( $x:expr ),+ ) => {{
        $($x.set_outdated();)+
    }};
}

/// Draw ui for algorithm selection, parameters fields for describing the shape.
/// Update
pub fn ui_options(
    ui: &mut Ui,
    current_layer_config: &mut SquircleParams,
    single_radius: &mut bool,
    code_enabled: bool,
    param_field_radius_a: &mut ParamField,
    param_field_radius_b: &mut ParamField,
    param_field_tilt: &mut ParamField,
    param_field_center_offset_x: &mut ParamField,
    param_field_center_offset_y: &mut ParamField,
    param_field_squircle_parameter: &mut ParamField,
    sampling_points: &ZVec<Vec<f64>>,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
) {
    // TODO: easily change algorithm for all layers
    // Select algorithm (the storage is for checking changed(), this is necessary
    //  as https://github.com/emilk/egui/discussions/923)
    if SquircleAlgorithm::combo_box(ui, &mut current_layer_config.algorithm) {
        outdate!(
            parameters_current_layer_control,
            parameters_all_layers_control
        );
    }

    // algorithm description
    ui.label(current_layer_config.algorithm.describe());

    // algorithm-specific options
    match current_layer_config.algorithm {
        SquircleAlgorithm::Percentage(percentage) => {
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
                current_layer_config.algorithm = SquircleAlgorithm::Percentage(perc_slider);
            };
        }
        _ => {}
    }

    // Radius
    ui.separator();

    ui.checkbox(single_radius, "Single radius");

    if *single_radius {
        param_field_radius_a.show(
            &mut current_layer_config.radius_a,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            Some(&"Radius".to_string()),
        );
        current_layer_config.radius_b = current_layer_config.radius_a;
    } else {
        // radius a
        param_field_radius_a.show(
            &mut current_layer_config.radius_a,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        // radius b
        param_field_radius_b.show(
            &mut current_layer_config.radius_b,
            ui,
            &code_enabled,
            sampling_points,
            parameters_current_layer_control,
            parameters_all_layers_control,
            None,
        );

        //longterm: Make circular slider for more intuitive controls (need to build this myapp probably)
    }

    //tilt
    param_field_tilt.show(
        &mut current_layer_config.tilt,
        ui,
        &code_enabled,
        sampling_points,
        parameters_current_layer_control,
        parameters_all_layers_control,
        None,
    );

    // Squircle parameter
    param_field_squircle_parameter.show(
        &mut current_layer_config.squircle_parameter,
        ui,
        &code_enabled,
        sampling_points,
        parameters_current_layer_control,
        parameters_all_layers_control,
        None,
    );

    // Centerpoint
    ui.separator();
    param_field_center_offset_x.show(
        &mut current_layer_config.center_offset_x,
        ui,
        &code_enabled,
        sampling_points,
        parameters_current_layer_control,
        parameters_all_layers_control,
        None,
    );

    param_field_center_offset_y.show(
        &mut current_layer_config.center_offset_y,
        ui,
        &code_enabled,
        sampling_points,
        parameters_current_layer_control,
        parameters_all_layers_control,
        None,
    );

    // Add odd and even buttons (also good so people understand what the abstraction "offset center" actually means)
    // todo: fix. how to make modular (linked parameters x and y?)
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

    if param_field_radius_a.has_changed()
        || param_field_radius_b.has_changed()
        || param_field_tilt.has_changed()
        || param_field_center_offset_x.has_changed()
        || param_field_center_offset_y.has_changed()
        || param_field_squircle_parameter.has_changed()
    {
        outdate!(
            parameters_current_layer_control,
            parameters_all_layers_control
        );
    }
}
