use crate::app::colors::COLOR_WIRE;
use crate::app::math::linear_algebra::Vec2;
use crate::app::metrics::sparse_blocks::SparseBlocks;
use crate::app::plotting;
use eframe::epaint::Stroke;
use egui::{Color32, RichText, Ui};
use egui_plot::{uniform_grid_spacer, Plot, PlotPoint, PlotUi, Text};

pub fn ui_build_seq(
    ui: &mut Ui,
    boundary_conn_comp: &Vec<(SparseBlocks, SparseBlocks, Color32)>,
    boundary_conn_comp_centers: &Vec<[f64; 2]>,
    build_seq: &Vec<usize>,
) {
    // The header needs exact sizing, otherwise it grows to meet the top
    // todo: disable horizontal movement
    Plot::new("build_seq_plot")
        .data_aspect(1.0)
        // grid is for debugging
        .x_grid_spacer(uniform_grid_spacer(|_gridinput| [1.0, 5.0, 10.0]))
        .y_grid_spacer(uniform_grid_spacer(|_gridinput| [1.0, 5.0, 10.0]))
        .allow_boxed_zoom(false)
        .auto_bounds([true, true]) // todo: set default false and zoom manually
        .allow_double_click_reset(false)
        .show_axes([false, false])
        .show(ui, |plot_ui| {
            // Group same type of shapes based on trans_normal_form
            // using the second usize to represent the number of those blocks in a row
            let mut grouped_seq: Vec<(usize, usize)> = Vec::new();
            let mut counter = 0;
            let mut prev_trans_normal_form = SparseBlocks::default();
            let mut prev_i = 0;
            for &i in build_seq {
                let trans_normal_form = boundary_conn_comp
                    .get(i)
                    .unwrap()
                    .clone()
                    .0
                    .trans_normal_form();
                if trans_normal_form == prev_trans_normal_form {
                    counter += 1;
                } else {
                    if counter != 0 {
                        grouped_seq.push((prev_i, counter));
                    }
                    prev_trans_normal_form = trans_normal_form;
                    counter = 1;
                }
                prev_i = i;
            }
            if counter != 0 {
                grouped_seq.push((prev_i, counter));
            }

            let n = grouped_seq.len();
            for j in 0..n {
                let (i, count) = grouped_seq[j];

                draw_sparse_blocks_normalized(
                    plot_ui,
                    boundary_conn_comp[i].0.clone(), // non-normalized version
                    1.0,
                    3.0,
                    boundary_conn_comp[i].2, // color from hash
                    [0.0, -4.0 * j as f64],
                );

                plot_ui.text(Text::new(
                    PlotPoint::from([-4.0, -4.0 * j as f64]),
                    RichText::new(format!("{}.", j + 1)).size(30.0),
                ));
                if count > 1 {
                    plot_ui.text(Text::new(
                        PlotPoint::from([-2.0, -4.0 * j as f64]),
                        RichText::new(format!("{}×", count)).size(30.0),
                    ));
                }
            }
            // todo: draw arrows between blocks
            // todo: nicer ui, frames and buttons etc
        });
}

/// blocks: which SparseBlocks to paint
/// max_scale: the largest size a single block can be painted
/// max_size: the size in which blocks should be painted
fn draw_sparse_blocks_normalized(
    plot_ui: &mut PlotUi,
    blocks: SparseBlocks,
    max_scale: f64,
    max_size: f64,
    fill_color: Color32,
    center_offset: [f64; 2],
) {
    // 1. determine scale
    let dim = *blocks.get_dimensions().iter().max().unwrap() as f64;
    let scale = max_scale.min(max_size / dim);

    let blocks_center: Vec2 = blocks.get_center().into();

    // 2. loop over SparseBlocks, calling plotting::square_at_coords (see viewport.rs)
    for block in blocks.get_coords() {
        let block = Vec2::from([block[0] as f64, block[1] as f64]);
        let transformed_coord = scale * (block - blocks_center) + center_offset.into();

        plot_ui.polygon(
            plotting::square_at_coords(transformed_coord.into(), scale)
                .stroke(Stroke {
                    width: 1.0,
                    color: COLOR_WIRE,
                })
                .fill_color(fill_color),
        )
    }
}
