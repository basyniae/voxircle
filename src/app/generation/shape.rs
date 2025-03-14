use crate::app::data_structures::blocks::Blocks;
use egui::Ui;
use std::fmt::{Debug, Display};

/// Abstraction for a class of algorithms, which all use the same parameters (e.g., squircles, lines).
/// The parameters are Param, these fully describe the shape to be approximated
/// Instances of Alg are (pointers to) the algorithm, Alg is some enum.
pub trait Shape<Alg: Debug + PartialEq + Default + Clone + Copy + Display, Param> {
    /// Description for info display
    fn describe(&self, alg: &Alg) -> String;

    /// Name of algorithms (for combobox display)
    fn name(&self, alg: &Alg) -> String;

    /// List of all algorithms that we want to make selectable
    fn all_algs(&self) -> Vec<Alg>;

    /// Generate a combo box to pick from all the algorithms. Return true if
    /// the value has changed (this frame).
    fn combo_box(&self, ui: &mut Ui, alg: &mut Alg) -> bool {
        let old_alg = alg.clone();
        egui::ComboBox::from_label("Algorithm")
            .selected_text(format!("{:}", alg))
            .show_ui(ui, |ui| {
                for i in self.all_algs() {
                    ui.selectable_value(alg, i, self.name(&i));
                }
            });
        old_alg != *alg
    }

    /// Generate the blocks with the given algorithm and parameters
    fn generate(&self, alg: &Alg, params: &Param) -> Blocks;

    // Need some rhai field here (as the type of Rhai field depends on the parameters)
}
