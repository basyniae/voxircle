use eframe::emath::Align;
use egui::{Layout, Ui};

pub fn percentage_slider(ui: &mut Ui, parameter: &mut f64) -> bool {
    ui.add(
        egui::Slider::new(parameter, 0.0..=1.0)
            .text("")
            .fixed_decimals(2)
            .custom_formatter(|n, _| {
                format!("{:.0}%", n * 100.0) //  formatting of percentage slider
            }),
    )
    .changed()
}

pub fn even_odd_buttons(ui: &mut Ui, x: &mut f64, y: &mut f64) -> bool {
    let mut clicked = false;
    ui.allocate_ui_with_layout(
        egui::Vec2::from([100.0, 200.0]),
        Layout::left_to_right(Align::Min),
        |ui| {
            [("Even center", 0.0, 0.0), ("Odd center", 0.5, 0.5)].map(|(name, x_0, y_0)| {
                if ui.button(name).clicked() {
                    *x = x_0;
                    *y = y_0;
                    clicked = true
                }
            });
        },
    );
    clicked
}

// todo: parts of draw_widgets can go here (HLine, VLine, tilted lines)
