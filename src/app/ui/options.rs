use crate::app::control::Control;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation;
use crate::app::generation::line::Line;
use crate::app::generation::shape_type::ShapeType;
use crate::app::generation::squircle::Squircle;
use crate::app::generation::{AllAlgs, AllFields, AllParams};
use crate::app::param_config::ParamConfig;
use egui::Ui;

pub fn ui_show_options(
    ui: &mut Ui,
    alg: &mut AllAlgs,
    params: &mut AllParams,
    fields: &mut AllFields,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
    sampling_points: &ZVec<Vec<f64>>,
    code_enabled: bool,
    shape_type: &ShapeType,
    param_config: &mut ParamConfig,
) {
    // pick algorithm here
    let old_alg = alg.clone();

    generation::algorithm_combobox(ui, alg, shape_type);
    if old_alg != *alg {
        parameters_current_layer_control.set_outdated();
        parameters_all_layers_control.set_outdated();
    }

    // algorithm description
    ui.label(alg.describe());

    ui.separator(); // Match first over the algorithm so that type is checked by the compiler at least a little bit
                    //  (we will get a reminder to add a new entry in here if we add a variant to the AllAlgs struct)
    match alg {
        AllAlgs::Null => panic!("impossible"),
        AllAlgs::Squircle(alg) => {
            if let (AllParams::Squircle(params), AllFields::Squircle(fields)) = (params, fields) {
                Squircle::show_options(
                    ui,
                    params,
                    fields,
                    alg,
                    parameters_current_layer_control,
                    parameters_all_layers_control,
                    sampling_points,
                    code_enabled,
                    param_config,
                )
            } else {
                panic!("ui_show_options called with arguments that do not match or have not been implemented")
            }
        }
        AllAlgs::Line(alg) => {
            if let (AllParams::Line(params), AllFields::Line(fields)) = (params, fields) {
                Line::show_options(
                    ui,
                    params,
                    fields,
                    alg,
                    parameters_current_layer_control,
                    parameters_all_layers_control,
                    sampling_points,
                    code_enabled,
                )
            } else {
                panic!("ui_show_options called with arguments that do not match or have not been implemented")
            }
        }
    }
}
