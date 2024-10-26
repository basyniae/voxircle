use crate::app::sampling::{SampleCombineMethod, SampleDistributeMethod};
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
    sampling_points_compute_once: &mut bool,
    sampling_points_compute_auto: &mut bool,
    sampling_points_is_outdated: &mut bool,
) {
    ui.label("Vertical sampling of the code. Requires code mode to be on.");

    ui.add_enabled_ui(sampling_enabled, |ui| {
        if egui::ComboBox::from_label("Sample combination method")
            // TODO: change formatting to text
            .selected_text(format!("{:?}", sample_combine_method))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    sample_combine_method,
                    SampleCombineMethod::AnySamples,
                    "Any samples (for Contained)",
                );
                ui.selectable_value(
                    sample_combine_method,
                    SampleCombineMethod::AllSamples,
                    "All samples (for Conservative)",
                );
                ui.selectable_value(
                    sample_combine_method,
                    SampleCombineMethod::Percentage(0.5),
                    "Given number of percentage of all samples",
                );
            })
            .response
            .changed()
        {
            *sampling_points_is_outdated = true;
        };

        match sample_combine_method {
            SampleCombineMethod::Percentage(percentage) => {
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
                    *sample_combine_method = SampleCombineMethod::Percentage(perc_slider);
                    *sampling_points_is_outdated = true;
                };
            }
            _ => {}
        }

        if egui::ComboBox::from_label("Sample distribution method")
            .selected_text(format!("{:?}", sample_distribute_method))
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
            *sampling_points_is_outdated = true;
        };

        if ui
            .checkbox(
                only_sample_half_of_bottom_layer,
                "Only sample half of the bottom layer",
            )
            .changed()
        {
            *sampling_points_is_outdated = true;
        };

        if ui
            .checkbox(
                only_sample_half_of_top_layer,
                "Only sample half of the top layer",
            )
            .changed()
        {
            *sampling_points_is_outdated = true;
        };

        ui.checkbox(
            sampling_points_compute_auto,
            "Auto recompute sampling points",
        );
        if ui.button("Recompute sampling points").clicked() {
            *sampling_points_compute_once = true
        }

        if ui
            .add(egui::Slider::new(nr_samples_per_layer, 1..=20).text("Nr. samples per layer"))
            .changed()
        {
            *sampling_points_is_outdated = true;
            if !sampling_enabled {
                *nr_samples_per_layer = 1; // if sampling is off, don't allow changing this value
            }
        };
    });
}
