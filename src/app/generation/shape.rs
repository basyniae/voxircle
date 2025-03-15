use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::zvec::ZVec;
use crate::app::sampling::layer_parameters::LayerParameters;
use egui::{Color32, Ui};
use egui_plot::PlotUi;
use std::fmt::{Debug, Display};

/// Abstraction for a class of algorithms, which all use the same parameters (e.g., squircles, lines).
/// The parameters are Param, these fully describe the shape to be approximated
/// Instances of Alg are (pointers to) the algorithm, Alg is some enum.
pub trait Shape<
    Alg: PartialEq + Default + Clone + Copy,
    Params: Default + Clone,
    Fields: Default + ShapeFields,
>
{
    /// Description for info display
    fn describe(alg: &Alg) -> String;

    /// Name of algorithms (for combobox display)
    fn name(alg: &Alg) -> String;

    /// List of all algorithms that we want to make selectable
    fn all_algs() -> Vec<Alg>;

    /// Generate a combo box to pick from all the algorithms. Return true if
    /// the value has changed (this frame).
    fn combo_box(ui: &mut Ui, alg: &mut Alg) -> bool {
        let old_alg = alg.clone();
        egui::ComboBox::from_label("Algorithm")
            .selected_text(format!("{:}", Self::name(alg)))
            .show_ui(ui, |ui| {
                for i in Self::all_algs() {
                    ui.selectable_value(alg, i, Self::name(&i));
                }
            });
        old_alg != *alg
    }

    fn grid_size(all_params: &Vec<Params>) -> usize;

    /// Generate the blocks with the given algorithm and parameters
    fn generate(alg: &Alg, params: &Params, grid_size: usize) -> Blocks;

    fn bounds(params: &Params, pad_factor: f64) -> [[f64; 2]; 2];

    /// Captures layout of the options (editing *only* the parameters, not the algorithm)
    fn show_options(
        &mut self,
        ui: &mut Ui,
        params: &mut Params,
        param_fields: &mut Fields,
        alg: &mut Alg,
        parameters_current_layer_control: &mut Control,
        parameters_all_layers_control: &mut Control,
        sampling_points: &ZVec<Vec<f64>>,
        code_enabled: bool,
    );

    /// Draw the shape
    fn draw(plot_ui: &mut PlotUi, params: Params, color: Color32);

    /// Draw widgets horizontal and vertical line through the center of the shape, and diagonals
    fn draw_widgets(plot_ui: &mut PlotUi, params: Params);

    fn set_parameters(
        &self,
        layer_parameters: &mut LayerParameters<Alg, Params, Fields, Self>,
        sampling_points: &Vec<f64>,
        default_shape: &Params,
        algorithm: Alg,
        fields: &mut Fields,
    ) where
        Self: Clone + Default;

    fn update_slider_parameters(
        &self,
        current_layer_shape: &mut Params,
        layer: isize,
        fields: &mut Fields,
    );
}

pub trait ShapeFields {
    fn all_register_success(&mut self);

    fn has_any_changed(&mut self) -> bool;
}
