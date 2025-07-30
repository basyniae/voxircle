use crate::app::metrics::symmetry_type::SymmetryType;
use crate::app::view::View;
use eframe::egui;
use eframe::egui::Ui;
use egui::Align::Min;
use egui::{Checkbox, Layout};

pub fn ui_viewport_options(
    ui: &mut Ui,
    layers_enabled: bool,
    symmetry_type: &SymmetryType,
    nr_connected_comp_bdry_2d: usize,
    view: &mut View,
) {
    ui.checkbox(&mut view.blocks, "Blocks");
    ui.columns(2, |columns| {
        // Better to fully hide 3d settings
        if layers_enabled {
            columns[0].with_layout(Layout::left_to_right(Min), |ui| {
                ui.checkbox(&mut view.boundary_2d, "Layer Boundary");
                ui.add_enabled(
                    view.boundary_2d,
                    Checkbox::new(&mut view.boundary_2d_colorful, "Colorful"),
                )
            });
            columns[0].checkbox(&mut view.interior_2d, "Layer Interior");
            columns[1].checkbox(&mut view.boundary_3d, "3D Boundary");
            columns[1].checkbox(&mut view.interior_3d, "3D Interior");
        } else {
            columns[0].with_layout(Layout::left_to_right(Min), |ui| {
                ui.checkbox(&mut view.boundary_2d, "Boundary");
                ui.add_enabled(
                    view.boundary_2d,
                    Checkbox::new(&mut view.boundary_2d_colorful, "Colorful"),
                )
            });
            columns[0].checkbox(&mut view.interior_2d, "Interior");
        }
    });

    ui.collapsing("Symmetry & Building", |ui| {
        ui.label(format!("Symmetry type: {:}", symmetry_type));
        ui.checkbox(&mut view.center_blocks, "Center blocks");
        ui.checkbox(&mut view.bounds, "Bounds");
        ui.checkbox(&mut view.mirrors, "Mirrors");
    });

    ui.collapsing("Technical", |ui| {
        ui.label(format!(
            "Nr. connected components of 2D boundary: {}",
            nr_connected_comp_bdry_2d
        ));
        ui.checkbox(&mut view.complement, "Complement");
        ui.checkbox(&mut view.convex_hull, "Convex hull");
        ui.checkbox(&mut view.outer_corners, "Outer corners");
        ui.checkbox(
            &mut view.intersect_area,
            "Intersect area (not in this version)",
        );
        ui.checkbox(&mut view.boundary_2d_graph, "Graph of boundary components")
    });
}
