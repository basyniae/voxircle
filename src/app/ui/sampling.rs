use crate::app::control::Control;
use crate::app::sampling::{SampleCombineMethod, SampleDistributeMethod};
use crate::app::ui::bits::percentage_slider;
use eframe::egui;
use eframe::egui::Ui;

/// document
pub fn ui_sampling(
    ui: &mut Ui,
    sampling_enabled: bool,
    only_sample_half_of_bottom_layer: &mut bool,
    only_sample_half_of_top_layer: &mut bool,
    nr_samples_per_layer: &mut usize,
    sample_combine_method: &mut SampleCombineMethod,
    sample_distribute_method: &mut SampleDistributeMethod,
    sampling_points_control: &mut Control,
) {
    ui.label("Vertical sampling of the code. Requires code mode to be on.");

    ui.add_enabled_ui(sampling_enabled, |ui| {
        if egui::ComboBox::from_label("Sample combination method")
            .selected_text(format!("{:}", sample_combine_method))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    sample_combine_method,
                    SampleCombineMethod::AnySamples,
                    "Any samples",
                );
                ui.selectable_value(
                    sample_combine_method,
                    SampleCombineMethod::AllSamples,
                    "All samples",
                );
                ui.selectable_value(
                    sample_combine_method,
                    SampleCombineMethod::Percentage(0.5),
                    "Percentage of samples",
                );
            })
            .response
            .changed()
        {
            sampling_points_control.set_outdated();
        };

        // Extra sampling method-specific options
        match sample_combine_method {
            SampleCombineMethod::Percentage(percentage) => {
                let mut perc_slider = percentage.clone();
                if percentage_slider(ui, &mut perc_slider) {
                    *sample_combine_method = SampleCombineMethod::Percentage(perc_slider);
                    sampling_points_control.set_outdated();
                };
            }
            _ => {}
        }

        if egui::ComboBox::from_label("Sample distribution method")
            .selected_text(format!("{:}", sample_distribute_method))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    sample_distribute_method,
                    SampleDistributeMethod::IncludeEndpoints,
                    "Include endpoints",
                );
                ui.selectable_value(
                    sample_distribute_method,
                    SampleDistributeMethod::ExcludeEndpoints,
                    "Exclude endpoints",
                );
            })
            .response
            .changed()
        {
            sampling_points_control.set_outdated();
        };

        if ui
            .checkbox(
                only_sample_half_of_bottom_layer,
                "Only sample half of the bottom layer",
            )
            .changed()
        {
            sampling_points_control.set_outdated();
        };

        if ui
            .checkbox(
                only_sample_half_of_top_layer,
                "Only sample half of the top layer",
            )
            .changed()
        {
            sampling_points_control.set_outdated();
        };

        ui.checkbox(
            sampling_points_control.auto(),
            "Auto recompute sampling points",
        );
        if ui.button("Recompute sampling points").clicked() {
            sampling_points_control.set_outdated();
        }

        if ui
            .add(egui::Slider::new(nr_samples_per_layer, 1..=20).text("Nr. samples per layer"))
            .changed()
        {
            sampling_points_control.set_outdated();
            if !sampling_enabled {
                *nr_samples_per_layer = 1; // if sampling is off, don't allow changing this value
            }
        };
    });
}
