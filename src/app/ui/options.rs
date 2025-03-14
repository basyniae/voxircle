use crate::app::control::Control;
use crate::app::data_structures::squircle_params::SquircleParams;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::shape::Shape;
use crate::app::generation::squircle::{Squircle, SquircleAlgorithm};
use crate::app::param_field::ParamField;
use eframe::egui;
use eframe::egui::Ui;

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
    current_layer_shape: &mut SquircleParams,
    current_layer_alg: &mut SquircleAlgorithm,
    code_enabled: bool,
    param_fields: &mut Vec<ParamField>,
    sampling_points: &ZVec<Vec<f64>>,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
    squircle: &mut Squircle,
) {
    // TODO: easily change algorithm for all layers
    // Select algorithm (the storage is for checking changed(), this is necessary
    //  as https://github.com/emilk/egui/discussions/923)
    if SquircleAlgorithm::combo_box(ui, current_layer_alg) {
        outdate!(
            parameters_current_layer_control,
            parameters_all_layers_control
        );
    }

    // algorithm description
    ui.label(current_layer_alg.describe());

    // algorithm-specific options
    match current_layer_alg {
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
                *current_layer_alg = SquircleAlgorithm::Percentage(perc_slider);
            };
        }
        _ => {}
    }

    // Radius
    ui.separator();

    squircle.show_options(
        ui,
        current_layer_shape,
        param_fields,
        parameters_current_layer_control,
        parameters_all_layers_control,
        sampling_points,
        code_enabled,
    )
}
