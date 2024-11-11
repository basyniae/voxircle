use crate::app::colors::{
    linear_gradient, COLOR_BACKGROUND, COLOR_DARK_BLUE, COLOR_DARK_GREEN, COLOR_DARK_ORANGE,
    COLOR_FACE, COLOR_LIGHT_BLUE, COLOR_LIME, COLOR_MUTED_ORANGE, COLOR_ORANGE, COLOR_PURPLE,
    COLOR_RED, COLOR_WIRE, COLOR_X_AXIS, COLOR_YELLOW, COLOR_Y_AXIS,
};
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::layer_config::LayerConfig;
use crate::app::data_structures::symmetry_type::SymmetryType;
use crate::app::metrics::convex_hull::line_segments_from_conv_hull;
use crate::app::plotting::bounds_from_square;
use crate::app::sampling::sampled_parameters::SampledParameters;
use crate::app::{generation, plotting};
use eframe::egui::{Stroke, Ui, Vec2b};
use egui_plot::{
    uniform_grid_spacer, HLine, Line, Plot, PlotBounds, PlotPoint, PlotPoints, Points, Text, VLine,
};
use itertools::izip;
use std::f64::consts::PI;

pub fn ui_viewport(
    ui: &mut Ui,
    global_bounding_box: [[f64; 2]; 2],
    layer_config: LayerConfig,
    sampled_parameters: SampledParameters,
    blocks: Blocks,
    // todo: factor out the zoom size determination for clearer access
    sampling_enabled: bool,

    // Viewport options
    view_blocks: bool,
    view_boundary_2d: bool,
    view_interior_2d: bool,
    view_complement: bool,
    view_intersect_area: bool,
    view_boundary_3d: bool,
    view_interior_3d: bool,
    view_convex_hull: bool,
    view_outer_corners: bool,

    view_center_blocks: bool,
    view_bounds: bool,
    view_mirrors: bool,
    symmetry_type: &SymmetryType,
    center_coord: &[f64; 2], //todo: rename

    // Zoom options (used for double click to reset zoom)
    reset_zoom_once: &mut bool,
    reset_zoom_continuous: &mut bool,

    // Metrics
    boundary_2d: Blocks,
    interior_2d: Blocks,
    complement_2d: Blocks,
    boundary_3d_slice: Option<Blocks>,
    interior_3d_slice: Option<Blocks>,
    convex_hull: &Vec<[f64; 2]>,
    outer_corners: &Vec<[f64; 2]>,
) {
    ui.visuals_mut().extreme_bg_color = COLOR_BACKGROUND;

    Plot::new("my_plot")
        .data_aspect(1.0) // so that squares in the rasterization always look square in the viewport
        // Grid lines of increasing thickness at distance 1.0, 5.0, 10.0 for counting
        .x_grid_spacer(uniform_grid_spacer(|_gridinput| [1.0, 5.0, 10.0]))
        .y_grid_spacer(uniform_grid_spacer(|_gridinput| [1.0, 5.0, 10.0]))
        .allow_boxed_zoom(false)
        // We don't need this, there's a maximal reasonable zoom in level and the reasonable zoom out level is only as big as the circle we're generating
        .auto_bounds(Vec2b::from([false, false]))
        .allow_double_click_reset(false) // we do this ourselves
        .label_formatter(move |_name, mouse_coord| {
            // if !name.is_empty() {  // Can condition formatting only on name of object! So if we want to have different tooltips for different objects this is what we must do
            //     format!("{}: {:.*}%", name, 1, value.y)
            // } else {
            //     "".to_owned()
            // } // longterm: think about integer coords for odd & even circles (no +/- zero for even circles)... ideally have it dep. only on...
            format!(
                "{0:.0}, {1:.0}",
                mouse_coord.x.trunc(),
                mouse_coord.y.trunc()
            ) // Use trunc instead of floor for symmetry preservation around the axis! Nasty but works
        })
        .show_axes([false, false]) // Don't show number axes
        .show(ui, |plot_ui| {
            // Reset zoom (approximates default behaviour, but we get to specify the action of automatic zooming
            if *reset_zoom_once || *reset_zoom_continuous {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                    global_bounding_box[0],
                    global_bounding_box[1],
                ));
                *reset_zoom_once = false
            }

            if plot_ui.response().clicked() || plot_ui.response().drag_started() {
                *reset_zoom_continuous = false
            }

            if plot_ui.response().double_clicked() {
                *reset_zoom_continuous = true // not sure if best to reset zoom once or reset zoom continuously
            }

            // * Viewport plotting * //
            // Draw order should be largest to smallest, so
            // 1. blocks
            // 2. complement
            // 3. 3d boundary
            // 4. 2d boundary
            // 5. 2d interior
            // 6. 3d interior
            // then geometric overlays like the target shape, center, etc.

            // First draw the blocks (the for loop is to avoid duplicate code)
            for (view, option_blocks, color) in izip!(
                [
                    view_blocks,
                    view_complement,
                    view_boundary_3d,
                    view_boundary_2d,
                    view_interior_2d,
                    view_interior_3d,
                    view_center_blocks,
                ],
                [
                    Some(blocks.clone()), //fixme: remove clone
                    Some(complement_2d),
                    boundary_3d_slice,
                    Some(boundary_2d),
                    Some(interior_2d),
                    interior_3d_slice,
                    Some(blocks.clone().get_center_blocks()), // fixme: update in proper order
                ],
                [
                    COLOR_FACE,
                    COLOR_ORANGE,
                    COLOR_PURPLE,
                    COLOR_LIGHT_BLUE,
                    COLOR_YELLOW,
                    COLOR_MUTED_ORANGE,
                    COLOR_RED,
                ]
            ) {
                if view {
                    if let Some(blocks) = option_blocks {
                        for coord in blocks.get_all_block_coords() {
                            plot_ui.polygon(
                                plotting::square_at_coords(coord)
                                    .stroke(Stroke {
                                        width: 1.0,
                                        color: COLOR_WIRE,
                                    })
                                    .fill_color(color),
                            );
                        }
                    }
                }
            }

            // Plot onion skinned samples
            if sampling_enabled {
                for i in 0..sampled_parameters.nr_samples {
                    plot_ui.line(
                        plotting::superellipse_at_coords(
                            sampled_parameters.parameters[i][0],
                            sampled_parameters.parameters[i][1],
                            sampled_parameters.parameters[i][2],
                            sampled_parameters.parameters[i][3],
                            sampled_parameters.parameters[i][4],
                            sampled_parameters.parameters[i][5],
                        )
                        .color(linear_gradient(
                            COLOR_DARK_GREEN,
                            COLOR_DARK_BLUE,
                            i as f64 / (sampled_parameters.nr_samples as f64 - 1.0),
                        )),
                    );
                }
            }

            // Plot center
            plot_ui.points(
                Points::new(vec![[
                    layer_config.center_offset_x,
                    layer_config.center_offset_y,
                ]])
                .radius(5.0)
                .color(COLOR_DARK_GREEN),
            );

            // Plot target shape
            plot_ui.line(
                plotting::superellipse_at_coords(
                    layer_config.radius_a,
                    layer_config.radius_b,
                    layer_config.tilt,
                    layer_config.center_offset_x,
                    layer_config.center_offset_y,
                    layer_config.squircle_parameter,
                )
                .color(COLOR_LIME),
            );

            // Plot x and y axes through the center of the shape
            plot_ui.hline(
                HLine::new(layer_config.center_offset_y)
                    .color(COLOR_X_AXIS)
                    .width(2.0),
            );
            plot_ui.vline(
                VLine::new(layer_config.center_offset_x)
                    .color(COLOR_Y_AXIS)
                    .width(2.0),
            );

            // Plot rotated x and y axes for nonzero tilt (dark orange and purple)
            if layer_config.tilt != 0.0 {
                plot_ui.line(
                    plotting::tilted_line_in_bounds(
                        plot_ui.plot_bounds(),
                        layer_config.tilt,
                        layer_config.center_offset_x,
                        layer_config.center_offset_y,
                    )
                    .color(COLOR_DARK_ORANGE),
                );
                plot_ui.line(
                    plotting::tilted_line_in_bounds(
                        plot_ui.plot_bounds(),
                        layer_config.tilt + PI / 2.0,
                        layer_config.center_offset_x,
                        layer_config.center_offset_y,
                    )
                    .color(COLOR_PURPLE),
                );
            }

            if view_intersect_area {
                let grid_size =
                    (2.0 * 1.42 * f64::max(layer_config.radius_a, layer_config.radius_b)).ceil()
                        as usize
                        + 4;

                let square = Blocks::new((0..grid_size.pow(2)).map(|_| true).collect(), grid_size);

                for coord in square.get_all_block_coords() {
                    let cell_center = [coord[0] + 0.5, coord[1] + 0.5];
                    let mut x_center = cell_center[0] - layer_config.center_offset_x;
                    let mut y_center = cell_center[1] - layer_config.center_offset_y;

                    // Dihedral symmetry swaps (see percentage.rs for explanation)
                    if x_center < 0.0 {
                        x_center = -x_center;
                    }
                    if y_center < 0.0 {
                        y_center = -y_center;
                    }
                    if x_center > y_center {
                        (y_center, x_center) = (x_center, y_center);
                    }

                    plot_ui.text(Text::new(PlotPoint::from(cell_center), {
                        let value = generation::percentage::cell_disk_intersection_area(
                            layer_config.radius_a.max(layer_config.radius_b),
                            x_center,
                            y_center,
                        );

                        if value == 0.0 {
                            // Don't show zero intersect area
                            "".to_string()
                        } else {
                            format!("{:.2}", value)
                        }
                    }));
                }
            }

            // Perhaps better to use the plot_ui.shape
            if view_convex_hull {
                for i in line_segments_from_conv_hull(convex_hull.clone()) {
                    let pts: PlotPoints = (0..=1).map(|t| i[t]).collect();
                    plot_ui.line(Line::new(pts).color(COLOR_ORANGE));
                }
            }

            // Plot outer corners of block
            if view_outer_corners {
                for [i, j] in outer_corners {
                    plot_ui.points(
                        Points::new(vec![[*i, *j]])
                            .radius(3.0)
                            .color(COLOR_DARK_ORANGE),
                    );
                }
            }

            // Plot bounds of the blocks
            if view_bounds {
                let line = bounds_from_square(blocks.get_bounds_floats());
                plot_ui.line(line.color(COLOR_ORANGE)); // todo: think about colors
            }

            // todo: think about colors
            if view_mirrors {
                match symmetry_type {
                    SymmetryType::ReflectionHorizontal => {
                        plot_ui.hline(HLine::new(center_coord[1]).color(COLOR_PURPLE).width(2.0));
                    }
                    SymmetryType::ReflectionVertical => {
                        plot_ui.vline(VLine::new(center_coord[0]).color(COLOR_PURPLE).width(2.0));
                    }
                    SymmetryType::ReflectionDiagonalUp => {
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                plot_ui.plot_bounds(),
                                std::f64::consts::FRAC_PI_4,
                                layer_config.center_offset_x,
                                layer_config.center_offset_y,
                            )
                            .color(COLOR_PURPLE),
                        );
                    }
                    SymmetryType::ReflectionDiagonalDown => {
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                plot_ui.plot_bounds(),
                                -std::f64::consts::FRAC_PI_4,
                                layer_config.center_offset_x,
                                layer_config.center_offset_y,
                            )
                            .color(COLOR_PURPLE),
                        );
                    }
                    SymmetryType::ReflectionsCardinals => {
                        plot_ui.vline(VLine::new(center_coord[0]).color(COLOR_PURPLE).width(2.0));
                        plot_ui.hline(HLine::new(center_coord[1]).color(COLOR_PURPLE).width(2.0));
                    }
                    SymmetryType::ReflectionsDiagonals => {
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                plot_ui.plot_bounds(),
                                std::f64::consts::FRAC_PI_4,
                                layer_config.center_offset_x,
                                layer_config.center_offset_y,
                            )
                            .color(COLOR_PURPLE),
                        );
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                plot_ui.plot_bounds(),
                                -std::f64::consts::FRAC_PI_4,
                                layer_config.center_offset_x,
                                layer_config.center_offset_y,
                            )
                            .color(COLOR_PURPLE),
                        );
                    }
                    SymmetryType::ReflectionsAll => {
                        plot_ui.vline(VLine::new(center_coord[0]).color(COLOR_PURPLE).width(2.0));
                        plot_ui.hline(HLine::new(center_coord[1]).color(COLOR_PURPLE).width(2.0));
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                plot_ui.plot_bounds(),
                                std::f64::consts::FRAC_PI_4,
                                layer_config.center_offset_x,
                                layer_config.center_offset_y,
                            )
                            .color(COLOR_PURPLE),
                        );
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                plot_ui.plot_bounds(),
                                -std::f64::consts::FRAC_PI_4,
                                layer_config.center_offset_x,
                                layer_config.center_offset_y,
                            )
                            .color(COLOR_PURPLE),
                        );
                    }
                    SymmetryType::RotationHalf => {} //todo: how to visualize rotational symmetry?
                    SymmetryType::RotationQuarter => {}
                    SymmetryType::NoSymmetry => {}
                }
            }
        });
}
