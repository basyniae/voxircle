use crate::app::data_structures::symmetry_type::SymmetryType;
use crate::app::view::View;
use eframe::egui;
use eframe::egui::Ui;

pub fn ui_viewport_options(
    ui: &mut Ui,
    layers_enabled: bool,
    single_radius: bool,
    symmetry_type: &SymmetryType,
    view: &mut View,
) {
    ui.checkbox(&mut view.blocks, "Blocks");
    ui.columns(2, |columns| {
        // Better to fully hide 3d settings
        if layers_enabled {
            columns[0].checkbox(&mut view.boundary_2d, "Layer Boundary");
            columns[0].checkbox(&mut view.interior_2d, "Layer Interior");
            columns[1].checkbox(&mut view.boundary_3d, "3D Boundary");
            columns[1].checkbox(&mut view.interior_3d, "3D Interior");
        } else {
            columns[0].checkbox(&mut view.boundary_2d, "Boundary");
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
        ui.checkbox(&mut view.complement, "Complement");
        ui.checkbox(&mut view.convex_hull, "Convex hull");
        ui.checkbox(&mut view.outer_corners, "Outer corners");
        ui.add_enabled(
            single_radius,
            egui::Checkbox::new(
                &mut view.intersect_area,
                "Intersect area (only for circles)",
            ),
        );
    });
}