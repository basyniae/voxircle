use eframe::egui;
use eframe::egui::Ui;
pub fn ui_generation(
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
