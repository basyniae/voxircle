use crate::app::colors::*;
use crate::app::data_structures::blocks::Blocks;
use crate::app::generation::shape::{TraitAlgorithm, TraitFields, TraitParameters, TraitShape};
use crate::app::metrics::convex_hull::line_segments_from_conv_hull;
use crate::app::metrics::symmetry_type::SymmetryType;
use crate::app::plotting;
use crate::app::plotting::bounds_from_square;
use crate::app::sampling::layer_parameters::LayerParameters;
use crate::app::update::metrics::Metrics;
use crate::app::view::View;
use eframe::egui::{Stroke, Ui, Vec2b};
use egui::Color32;
use egui_plot::{
    uniform_grid_spacer, HLine, Line, Plot, PlotBounds, PlotPoints, PlotUi, Points, VLine,
};

pub fn ui_viewport<
    Alg: TraitAlgorithm,
    Params: TraitParameters,
    Fields: TraitFields,
    Shape: TraitShape<Alg, Params, Fields>,
>(
    ui: &mut Ui,
    shape_parameters: &Params,
    layer_parameters: &LayerParameters<Alg, Params, Fields, Shape>,
    blocks: &Option<&Blocks>,
    sampling_enabled: bool,
    view: &View,
    current_layer: isize,

    // Zoom options (used for double click to reset zoom)
    reset_zoom_once: &mut bool,
    reset_zoom_continuous: &mut bool,

    // Metrics
    metrics: &Metrics,
) {
    ui.visuals_mut().extreme_bg_color = COLOR_VIEWPORT_BACKGROUND;

    // on scrolling in the viewport, turn off auto zoom
    if ui.rect_contains_pointer(ui.max_rect()) {
        ui.ctx().input(|i| {
            if i.raw_scroll_delta != egui::Vec2::from([0.0, 0.0]) {
                *reset_zoom_continuous = false
            }
        })
    }

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
                let plot_bounds = plot_ui.plot_bounds();
                // this is the screen aspect ratio as we force squares to display as squares
                let aspect_ratio = plot_bounds.width() / plot_bounds.height();

                let new_bounds =
                    fit_aspect_ratio_around_box(metrics.global_bounding_box, aspect_ratio);

                plot_ui.set_plot_bounds(PlotBounds::from_min_max(new_bounds[0], new_bounds[1]));
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
            draw_blocks(plot_ui, view.blocks, blocks, COLOR_BLOCKS);
            draw_blocks(
                plot_ui,
                view.complement,
                &Some(&metrics.complement_2d),
                COLOR_COMPLEMENT_2D,
            );
            draw_blocks(
                plot_ui,
                view.complement,
                &metrics.boundary_3d.get(current_layer),
                COLOR_BOUNDARY_3D,
            );

            draw_blocks(
                plot_ui,
                view.boundary_2d && !view.boundary_2d_colorful, // only draw normally if we don't do it colorfully later
                &Some(&metrics.boundary_2d),
                COLOR_BOUNDARY_2D,
            );
            draw_blocks(
                plot_ui,
                view.interior_2d,
                &Some(&metrics.interior_2d),
                COLOR_INTERIOR_2D,
            );
            draw_blocks(
                plot_ui,
                view.interior_3d,
                &metrics.interior_3d.get(current_layer),
                COLOR_INTERIOR_3D,
            );
            draw_blocks(
                plot_ui,
                view.center_blocks,
                &blocks.map(|b| b.get_center_blocks()).as_ref(), // update with other metrics?
                COLOR_CENTER_BLOCKS,
            );

            // draw build color help
            if view.boundary_2d_colorful && view.boundary_2d {
                for comp in metrics.boundary_conn_comp.iter() {
                    for [x, y] in comp.get_coords() {
                        plot_ui.polygon(
                            plotting::square_at_coords([*x as f64, *y as f64])
                                .stroke(Stroke {
                                    width: 1.0,
                                    color: COLOR_WIRE,
                                })
                                .fill_color(comp.hash_color()),
                        )
                    }
                }
            }

            // Plot onion skinned samples
            if sampling_enabled {
                for i in 0..layer_parameters.nr_samples {
                    Shape::draw(
                        plot_ui,
                        layer_parameters.parameters[i].clone(),
                        linear_gradient(
                            COLOR_SAMPLE_A,
                            COLOR_SAMPLE_B,
                            i as f64 / (layer_parameters.nr_samples as f64 - 1.0),
                        ),
                    );
                }
            }

            // Plot convex hull
            // Perhaps better to use the plot_ui.shape
            if view.convex_hull {
                for i in line_segments_from_conv_hull(&metrics.convex_hull) {
                    let pts: PlotPoints = (0..=1).map(|t| i[t]).collect();
                    plot_ui.line(Line::new(pts).color(COLOR_CONV_HULL));
                }
            }

            // Plot outer corners of block
            if view.outer_corners {
                for [i, j] in metrics.outer_corners.iter() {
                    plot_ui.points(
                        Points::new(vec![[*i, *j]])
                            .radius(3.0)
                            .color(COLOR_OUTER_CORNERS),
                    );
                }
            }

            // Plot bounds of the blocks
            if let Some(b) = blocks {
                if view.bounds {
                    let line = bounds_from_square(b.get_bounds_floats());
                    plot_ui.line(line.color(COLOR_BOUNDS))
                }
            }

            // Plot mirrors
            if view.mirrors {
                match metrics.symmetry_type {
                    SymmetryType::ReflectionHorizontal => {
                        plot_ui.hline(
                            HLine::new(metrics.center_coord[1])
                                .color(COLOR_MIRRORS)
                                .width(2.0),
                        );
                    }
                    SymmetryType::ReflectionVertical => {
                        plot_ui.vline(
                            VLine::new(metrics.center_coord[0])
                                .color(COLOR_MIRRORS)
                                .width(2.0),
                        );
                    }
                    SymmetryType::ReflectionDiagonalUp => {
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                plot_ui.plot_bounds(),
                                std::f64::consts::FRAC_PI_4,
                                metrics.center_coord[0],
                                metrics.center_coord[1],
                            )
                            .color(COLOR_MIRRORS),
                        );
                    }
                    SymmetryType::ReflectionDiagonalDown => {
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                plot_ui.plot_bounds(),
                                -std::f64::consts::FRAC_PI_4,
                                metrics.center_coord[0],
                                metrics.center_coord[1],
                            )
                            .color(COLOR_MIRRORS),
                        );
                    }
                    SymmetryType::ReflectionsCardinals => {
                        plot_ui.vline(
                            VLine::new(metrics.center_coord[0])
                                .color(COLOR_MIRRORS)
                                .width(2.0),
                        );
                        plot_ui.hline(
                            HLine::new(metrics.center_coord[1])
                                .color(COLOR_MIRRORS)
                                .width(2.0),
                        );
                    }
                    SymmetryType::ReflectionsDiagonals => {
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                plot_ui.plot_bounds(),
                                std::f64::consts::FRAC_PI_4,
                                metrics.center_coord[0],
                                metrics.center_coord[1],
                            )
                            .color(COLOR_MIRRORS),
                        );
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                plot_ui.plot_bounds(),
                                -std::f64::consts::FRAC_PI_4,
                                metrics.center_coord[0],
                                metrics.center_coord[1],
                            )
                            .color(COLOR_MIRRORS),
                        );
                    }
                    SymmetryType::ReflectionsAll => {
                        plot_ui.vline(
                            VLine::new(metrics.center_coord[0])
                                .color(COLOR_MIRRORS)
                                .width(2.0),
                        );
                        plot_ui.hline(
                            HLine::new(metrics.center_coord[1])
                                .color(COLOR_MIRRORS)
                                .width(2.0),
                        );
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                plot_ui.plot_bounds(),
                                std::f64::consts::FRAC_PI_4,
                                metrics.center_coord[0],
                                metrics.center_coord[1],
                            )
                            .color(COLOR_MIRRORS),
                        );
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                plot_ui.plot_bounds(),
                                -std::f64::consts::FRAC_PI_4,
                                metrics.center_coord[0],
                                metrics.center_coord[1],
                            )
                            .color(COLOR_MIRRORS),
                        );
                    }
                    SymmetryType::RotationHalf => {} //todo: how to visualize rotational symmetry?
                    SymmetryType::RotationQuarter => {}
                    SymmetryType::NoSymmetry => {}
                }
            }

            // Plot target shape
            Shape::draw(plot_ui, shape_parameters.clone(), COLOR_TARGET_SHAPE);

            Shape::draw_widgets(plot_ui, shape_parameters.clone())
        });
}

fn draw_blocks(plot_ui: &mut PlotUi, view: bool, option_blocks: &Option<&Blocks>, color: Color32) {
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
                )
            }
        }
    }
}

/// Fit a box of the given aspect ratio tightly around the bounding box
fn fit_aspect_ratio_around_box(bounding_box: [[f64; 2]; 2], aspect_ratio: f64) -> [[f64; 2]; 2] {
    let center = [
        0.5 * (bounding_box[0][0] + bounding_box[1][0]),
        0.5 * (bounding_box[0][1] + bounding_box[1][1]),
    ];
    let bb_width = bounding_box[1][0] - bounding_box[0][0];
    let bb_height = bounding_box[1][1] - bounding_box[0][1];

    let width = bb_width.max(bb_height * aspect_ratio);
    let height = bb_height.max(bb_width / aspect_ratio);

    [
        [center[0] - 0.5 * width, center[1] - 0.5 * height],
        [center[0] + 0.5 * width, center[1] + 0.5 * height],
    ]
}
