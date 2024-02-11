
mod app;
pub mod data_structures;

use std::default::Default;
use eframe::egui::ViewportBuilder;
use eframe::{Theme, HardwareAcceleration};

fn main() {
    println!("Hello, world!");
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
        "Voxel circles", 
        options, 
        Box::new(|cc|  Box::new(crate::app::App::new(cc)))
    );
}

