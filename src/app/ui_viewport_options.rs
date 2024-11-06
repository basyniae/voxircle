use crate::app::data_structures::blocks::SymmetryType;
use eframe::egui;
use eframe::egui::Ui;

pub fn ui_viewport_options(
    ui: &mut Ui,
    layers_enabled: bool,
    single_radius: bool,
    symmetry_type: &SymmetryType,

    view_blocks: &mut bool,
    view_boundary_2d: &mut bool,
    view_interior_2d: &mut bool,
    view_complement: &mut bool,
    view_intersect_area: &mut bool,
    view_boundary_3d: &mut bool,
    view_interior_3d: &mut bool,
    view_convex_hull: &mut bool,
    view_outer_corners: &mut bool,

    // viewport options for symmetry & building
    view_center_blocks: &mut bool,
    view_bounds: &mut bool,
    view_mirrors: &mut bool,
) {
    ui.checkbox(view_blocks, "Blocks");
    ui.columns(2, |columns| {
        // Better to fully hide 3d settings
        if layers_enabled {
            columns[0].checkbox(view_boundary_2d, "Layer Boundary");
            columns[0].checkbox(view_interior_2d, "Layer Interior");
            columns[1].checkbox(view_boundary_3d, "3D Boundary");
            columns[1].checkbox(view_interior_3d, "3D Interior");
        } else {
            columns[0].checkbox(view_boundary_2d, "Boundary");
            columns[0].checkbox(view_interior_2d, "Interior");
        }
    });

    ui.collapsing("Symmetry & Building", |ui| {
        ui.label(format!("Symmetry type: {:}", symmetry_type));
        ui.checkbox(view_center_blocks, "Center blocks");
        ui.checkbox(view_bounds, "Bounds");
        ui.checkbox(view_mirrors, "Mirrors");
    });

    ui.collapsing("Technical", |ui| {
        ui.checkbox(view_complement, "Complement");
        ui.checkbox(view_convex_hull, "Convex hull");
        ui.checkbox(view_outer_corners, "Outer corners");
        ui.add_enabled(
            single_radius,
            egui::Checkbox::new(view_intersect_area, "Intersect area (only for circles)"),
        );
    });
}
