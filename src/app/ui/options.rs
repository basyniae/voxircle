use crate::app::control::Control;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::line::Line;
use crate::app::generation::shape::{ui_show_options, AllAlgs, AllFields, AllParams};
use crate::app::generation::shape_type::ShapeType;
use crate::app::generation::squircle::Squircle;
use crate::app::param_config::ParamConfig;
use eframe::egui::Ui;

/// Draw ui for algorithm selection, parameters fields for describing the shape.
/// Update
pub fn ui_options(
    ui: &mut Ui,
    current_layer_shape: &mut AllParams,
    current_layer_alg: &mut AllAlgs,
    code_enabled: bool,
    fields: &mut AllFields,
    sampling_points: &ZVec<Vec<f64>>,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
    shape_type: &ShapeType,
    param_config: &mut ParamConfig,
) {
    ui.separator();

    // TODO: easily change algorithm for all layers
    // pick algorithm here
    let old_shape_type = current_layer_alg.clone();

    egui::ComboBox::from_label("Algorithm")
        .selected_text(format!("{:}", current_layer_alg.name()))
        .show_ui(ui, |ui| {
            for i in match shape_type {
                ShapeType::Squircle => Squircle::combobox_all_algs(),
                ShapeType::Line => Line::combobox_all_algs(),
            } {
                ui.selectable_value(current_layer_alg, i, i.name());
            }
        });

    if old_shape_type != *current_layer_alg {
        parameters_current_layer_control.set_outdated();
        parameters_all_layers_control.set_outdated();
    }

    // algorithm description
    ui.label(current_layer_alg.describe());

    // Radius
    ui.separator();

    ui_show_options(
        ui,
        current_layer_alg,
        current_layer_shape,
        fields,
        parameters_current_layer_control,
        parameters_all_layers_control,
        sampling_points,
        code_enabled,
        param_config,
    )
}
