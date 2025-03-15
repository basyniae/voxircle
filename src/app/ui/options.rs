use crate::app::control::Control;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::shape::Shape;
use crate::app::param_field::ParamField;
use eframe::egui::Ui;
use std::fmt::Debug;

// my first macro!
/// Mark the inputted control variables as outdated
macro_rules! outdate {
    ( $( $x:expr ),+ ) => {{
        $($x.set_outdated();)+
    }};
}

/// Draw ui for algorithm selection, parameters fields for describing the shape.
/// Update
pub fn ui_options<
    Alg: Debug + PartialEq + Default + Clone + Copy,
    Params: Default + Clone,
    Sh: Shape<Alg, Params> + Default + Clone,
>(
    ui: &mut Ui,
    current_layer_shape: &mut Params,
    current_layer_alg: &mut Alg,
    code_enabled: bool,
    // todo: a *vector* of ParamField is not very safe! maybe (yet) another struct for it
    param_fields: &mut Vec<ParamField>,
    sampling_points: &ZVec<Vec<f64>>,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
    shape: &mut Sh,
) {
    // pick Shape here
    ui.separator();

    // TODO: easily change algorithm for all layers
    // Select algorithm (the storage is for checking changed(), this is necessary
    //  as https://github.com/emilk/egui/discussions/923)
    if Sh::combo_box(ui, current_layer_alg) {
        outdate!(
            parameters_current_layer_control,
            parameters_all_layers_control
        );
    }

    // algorithm description
    ui.label(Sh::describe(current_layer_alg));

    // Radius
    ui.separator();

    shape.show_options(
        ui,
        current_layer_shape,
        param_fields,
        current_layer_alg,
        parameters_current_layer_control,
        parameters_all_layers_control,
        sampling_points,
        code_enabled,
    )
}
