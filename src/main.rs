#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
                                                                   // (ripped straight from the example)

use std::default::Default;

use eframe::egui::ViewportBuilder;
use eframe::{HardwareAcceleration, Theme};

mod app;

fn main() {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default(),
        vsync: false,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        hardware_acceleration: HardwareAcceleration::Preferred,
        renderer: Default::default(),
        follow_system_theme: false,
        default_theme: Theme::Dark,
        run_and_return: false,
        event_loop_builder: None,
        shader_version: None,
        centered: false,
        window_builder: Default::default(),
        persist_window: false,
    };

    // Use `let _`  to ignore the result properly...
    let _ = eframe::run_native(
        "Voxircle",
        options,
        Box::new(|cc| Box::new(app::App::new(cc))),
    );
}
