use eframe::egui;
use eframe::egui::Ui;

/// Output: old_layer (isize), has_layer_stack_changed, has_layer_changed
pub fn ui_layer_navigation(
    ui: &mut Ui,
    current_layer: &mut isize,
    layer_lowest: &mut isize,
    layer_highest: &mut isize,
    lock_stack_size: bool,
) -> (isize, bool, bool) {
    // bookkeeping for updating the configuration
    let old_layer = *current_layer;
    let prev_layer_lowest = *layer_lowest;
    let prev_layer_highest = *layer_highest;

    ui.centered_and_justified(|ui| {
        // Ui finicky due to not being able to know the size of the widget in advance
        // so do a pretty good prediction
        let height = ui.style().spacing.interact_size.y;
        let controls_width = height + 10.0;
        let main_width = ui.style().spacing.interact_size.x; // Incorrect for >4 digits (which is unlikely to occur)
        let padding = ui.style().spacing.button_padding.x; //Button size is text size plus this on each side

        let (rect, response) = ui.allocate_exact_size(
            [
                4.0 * controls_width + 3.0 * main_width + padding * 12.0,
                height,
            ]
                .into(),
            egui::Sense::click(),
        );
        ui.put(rect, |ui: &mut Ui| {
            ui.horizontal(|ui| {
                ui.add_enabled(
                    !lock_stack_size,
                    egui::DragValue::new(layer_lowest).speed(0.05),
                );
                if ui
                    .add(
                        egui::Button::new("|<")
                            .min_size(egui::Vec2::from([controls_width, height])),
                    )
                    .clicked()
                {
                    *current_layer = *layer_lowest;
                }
                if ui
                    .add(
                        egui::Button::new("<").min_size(egui::Vec2::from([controls_width, height])),
                    )
                    .clicked()
                {
                    if !lock_stack_size || *current_layer - 1 >= *layer_lowest {
                        *current_layer -= 1;
                    }
                }
                if lock_stack_size {
                    ui.add(
                        egui::DragValue::new(current_layer)
                            .speed(0.05)
                            .range(*layer_lowest..=*layer_highest),
                    );
                } else {
                    ui.add(egui::DragValue::new(current_layer).speed(0.05));
                }

                if ui
                    .add(
                        egui::Button::new(">").min_size(egui::Vec2::from([controls_width, height])),
                    )
                    .clicked()
                {
                    if !lock_stack_size || *current_layer + 1 <= *layer_highest {
                        *current_layer += 1;
                    }
                }
                if ui
                    .add(
                        egui::Button::new(">|")
                            .min_size(egui::Vec2::from([controls_width, height])),
                    )
                    .clicked()
                {
                    *current_layer = *layer_highest;
                }
                ui.add_enabled(
                    !lock_stack_size,
                    egui::DragValue::new(layer_highest).speed(0.05),
                );
            });
            response
        });
    });

    // Update the stack bounds to include the current layer
    *layer_lowest = *layer_lowest.min(current_layer);
    *layer_highest = *layer_highest.max(current_layer);

    (
        old_layer,
        prev_layer_lowest != *layer_lowest || prev_layer_highest != *layer_highest, // has layer stack changed?
        old_layer != *current_layer, // has current layer changed?
    )

    // Expression for if the stack has grown (might be useful later):
    // prev_layer_lowest > *layer_lowest || prev_layer_highest < *layer_highest
}
