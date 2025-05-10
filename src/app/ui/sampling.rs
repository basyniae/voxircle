use crate::app::control::Control;
use crate::app::sampling::{SampleCombineMethod, SampleDistributeMethod, SamplingOptions};
use crate::app::ui::bits::percentage_slider;
use eframe::egui;
use eframe::egui::Ui;

/// document
pub fn ui_sampling(
    ui: &mut Ui,
    sampling_enabled: bool,
    sampling_options: &mut SamplingOptions,
    sampling_points_control: &mut Control,
) {
    ui.label("Vertical sampling of the code. Requires code mode to be on.");

    ui.add_enabled_ui(sampling_enabled, |ui| {
        if egui::ComboBox::from_label("Sample combination method")
            .selected_text(format!("{:}", &mut sampling_options.sample_combine_method))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut sampling_options.sample_combine_method,
                    SampleCombineMethod::AnySamples,
                    "Any samples",
                );
                ui.selectable_value(
                    &mut sampling_options.sample_combine_method,
                    SampleCombineMethod::AllSamples,
                    "All samples",
                );
                ui.selectable_value(
                    &mut sampling_options.sample_combine_method,
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
        match sampling_options.sample_combine_method {
            SampleCombineMethod::Percentage(percentage) => {
                let mut perc_slider = percentage.clone();
                if percentage_slider(ui, &mut perc_slider) {
                    sampling_options.sample_combine_method =
                        SampleCombineMethod::Percentage(perc_slider);
                    sampling_points_control.set_outdated();
                };
            }
            _ => {}
        }

        if egui::ComboBox::from_label("Sample distribution method")
            .selected_text(format!("{:}", sampling_options.sample_distribute_method))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut sampling_options.sample_distribute_method,
                    SampleDistributeMethod::IncludeEndpoints,
                    "Include endpoints",
                );
                ui.selectable_value(
                    &mut sampling_options.sample_distribute_method,
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
                &mut sampling_options.only_sample_half_of_bottom_layer,
                "Only sample half of the bottom layer",
            )
            .changed()
        {
            sampling_points_control.set_outdated();
        };

        if ui
            .checkbox(
                &mut sampling_options.only_sample_half_of_top_layer,
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
            .add(
                egui::Slider::new(&mut sampling_options.nr_samples_per_layer, 1..=20)
                    .text("Nr. samples per layer"),
            )
            .changed()
        {
            sampling_points_control.set_outdated();
            if !sampling_enabled {
                sampling_options.nr_samples_per_layer = 1; // if sampling is off, don't allow changing this value
            }
        };
    });
}
