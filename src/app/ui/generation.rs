use eframe::egui;
use eframe::egui::Ui;
pub fn ui_generation(
    ui: &mut Ui,
    blocks_current_layer_generate_once: &mut bool,
    blocks_current_layer_generate_auto: &mut bool,
    blocks_all_layers_generate_once: &mut bool,
    blocks_all_layers_generate_auto: &mut bool,
    parameters_current_layer_sample_once: &mut bool,
    parameters_current_layer_sample_auto: &mut bool,
    parameters_all_layers_sample_once: &mut bool,
    parameters_all_layers_sample_auto: &mut bool,
    layers_enabled: bool,
    code_enabled: bool,
    sampling_enabled: bool,
) {
    if layers_enabled {
        ui.checkbox(
            blocks_current_layer_generate_auto,
            "Auto-generate blocks on current layer",
        );
        ui.checkbox(
            blocks_all_layers_generate_auto,
            "Auto-generate blocks on all layers",
        );
    } else {
        ui.checkbox(blocks_current_layer_generate_auto, "Auto-generate blocks");
    }

    if sampling_enabled {
        ui.checkbox(
            parameters_current_layer_sample_auto,
            "Auto-sample parameters on current layer",
        );
        ui.checkbox(
            parameters_all_layers_sample_auto,
            "Auto-sample parameters on all layers",
        );
    }

    ui_generation_buttons(
        ui,
        blocks_current_layer_generate_once,
        blocks_all_layers_generate_once,
        parameters_current_layer_sample_once,
        parameters_all_layers_sample_once,
        layers_enabled,
        code_enabled,
    );
}

fn ui_generation_buttons(
    ui: &mut Ui,
    blocks_current_layer_generate_once: &mut bool,
    blocks_all_layers_generate_once: &mut bool,
    parameters_current_layer_sample_once: &mut bool,
    parameters_all_layers_sample_once: &mut bool,
    layers_enabled: bool,
    code_enabled: bool,
) {
    egui::Grid::new("generation_grid").show(ui, |ui| {
        if code_enabled {
            let response = ui.add_sized(
                [150.0, 50.0],
                egui::Button::new({
                    if layers_enabled {
                        "Sample parameters for current layer"
                    } else {
                        "Sample parameters"
                    }
                })
                .wrap(true),
            );

            *parameters_current_layer_sample_once |= response.clicked();
        }

        if code_enabled && layers_enabled {
            let response = ui.add_sized(
                [150.0, 50.0],
                egui::Button::new("Sample parameters for all layers").wrap(true),
            );

            *parameters_all_layers_sample_once |= response.clicked();
        }

        if code_enabled {
            ui.end_row();
        }

        let response = ui.add_sized(
            [150.0, 50.0],
            egui::Button::new({
                if layers_enabled {
                    "Generate blocks on current layer"
                } else {
                    "Generate blocks"
                }
            })
            .wrap(true),
        );
        *blocks_current_layer_generate_once = response.clicked();

        if layers_enabled {
            let response = ui.add_sized(
                [150.0, 50.0],
                egui::Button::new("Generate blocks on all layers").wrap(true),
            );
            *blocks_all_layers_generate_once = response.clicked();
        };
    });
}
