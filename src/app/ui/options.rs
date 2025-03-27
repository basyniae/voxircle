use crate::app::control::Control;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation;
use crate::app::generation::line::{Line, LineFields};
use crate::app::generation::shape_type::ShapeType;
use crate::app::generation::squircle::{Squircle, SquircleFields};
use crate::app::generation::{AllAlgs, AllParams};
use crate::app::param_config::ParamConfig;
use egui::Ui;

pub fn ui_options(
    ui: &mut Ui,
    alg: &mut AllAlgs,
    params: &mut AllParams,
    squircle_fields: &mut SquircleFields,
    line_fields: &mut LineFields,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
    sampling_points: &ZVec<Vec<f64>>,
    code_enabled: bool,
    shape_type: &mut ShapeType,
    param_config: &mut ParamConfig,
) {
    // Pick shape for this layer
    let old_shape = shape_type.clone();
    egui::ComboBox::from_label("Shape")
        .selected_text(format!("{:}", shape_type))
        .show_ui(ui, |ui| {
            for i in ShapeType::list_all_shape_types() {
                ui.selectable_value(shape_type, i, i.name());
            }
        });

    // if a new shape is picked, change the parameters &c
    if old_shape != *shape_type {
        parameters_current_layer_control.set_outdated();
        parameters_all_layers_control.set_outdated();
        match shape_type {
            // todo: figure out better defaults based on "last seen of this type" or something
            ShapeType::Squircle => {
                *alg = AllAlgs::Squircle(Default::default());
                *params = AllParams::Squircle(Default::default());
            }
            ShapeType::Line => {
                *alg = AllAlgs::Line(Default::default());
                *params = AllParams::Line(Default::default());
            }
        }
    }

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
    if match alg {
        AllAlgs::Null => panic!("impossible"),
        AllAlgs::Squircle(alg) => {
            if let AllParams::Squircle(params) = params {
                Squircle::show_options(
                    ui,
                    params,
                    squircle_fields,
                    alg,
                    sampling_points,
                    code_enabled,
                    param_config,
                )
            } else {
                panic!("ui_options called with arguments that do not match or have not been implemented")
            }
        }
        AllAlgs::Line(alg) => {
            if let AllParams::Line(params) = params {
                Line::show_options(ui, params, line_fields, alg, sampling_points, code_enabled)
            } else {
                panic!("ui_options called with arguments that do not match or have not been implemented")
            }
        }
    } {
        parameters_all_layers_control.set_outdated();
        parameters_current_layer_control.set_outdated();
    }
}
