use crate::app::colors::{
    COLOR_CENTER_DOT, COLOR_TILTED_X_AXIS, COLOR_TILTED_Y_AXIS, COLOR_X_AXIS, COLOR_Y_AXIS,
};
use crate::app::plotting;
use eframe::emath::Align;
use egui::{Layout, Ui};
use egui_plot::{HLine, PlotUi, Points, VLine};
use std::f64::consts::PI;

/// A percentage slider
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

/// Even and odd buttons for the center
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

/// Draw horizontal and vertical lines, as well as diagonal variants and a central dot
pub fn draw_axes(plot_ui: &mut PlotUi, x: f64, y: f64, tilt: f64) {
    // Plot x and y axes through the center of the shape
    plot_ui.hline(HLine::new(y).color(COLOR_X_AXIS).width(2.0));
    plot_ui.vline(VLine::new(x).color(COLOR_Y_AXIS).width(2.0));

    // Plot rotated x and y axes for nonzero tilt (dark orange and purple)
    if tilt != 0.0 {
        plot_ui.line(
            plotting::tilted_line_in_bounds(plot_ui.plot_bounds(), tilt, x, y)
                .color(COLOR_TILTED_X_AXIS),
        );
        plot_ui.line(
            plotting::tilted_line_in_bounds(plot_ui.plot_bounds(), tilt + PI / 2.0, x, y)
                .color(COLOR_TILTED_Y_AXIS),
        );
    }

    // Plot center dot
    plot_ui.points(
        Points::new(vec![[x, y]])
            .radius(5.0)
            .color(COLOR_CENTER_DOT),
    );
}
