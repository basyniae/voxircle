use crate::app;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::slice_parameters::SliceParameters;
use crate::app::data_structures::symmetry_type::SymmetryType;
use crate::app::data_structures::zvec::ZVec;
use crate::app::math::exact_squircle_bounds::exact_squircle_bounds;
use crate::app::math::square_max::square_max;
use app::metrics::convex_hull::get_convex_hull;

pub fn update_metrics(
    current_layer: isize,
    layer_lowest: isize,
    layer_highest: isize,
    current_layer_blocks: Blocks,
    stack_blocks: &ZVec<Blocks>,
    stack_layer_config: &ZVec<SliceParameters>,

    // Metrics
    nr_blocks_total: &mut u64,
    nr_blocks_interior: &mut u64,
    nr_blocks_boundary: &mut u64,
    boundary_2d: &mut Blocks,
    interior_2d: &mut Blocks,
    complement_2d: &mut Blocks,
    boundary_3d: &mut ZVec<Blocks>,
    interior_3d: &mut ZVec<Blocks>,
    convex_hull: &mut Vec<[f64; 2]>,
    outer_corners: &mut Vec<[f64; 2]>,
    symmetry_type: &mut SymmetryType,
    center_coord: &mut [f64; 2],

    global_bounding_box: &mut [[f64; 2]; 2],
) {
    // update 2d spatial metrics
    *interior_2d = current_layer_blocks.get_interior();
    *boundary_2d = current_layer_blocks.get_boundary();
    *complement_2d = current_layer_blocks.get_complement();

    // update 3d spatial metrics
    *boundary_3d = app::metrics::boundary_3d::boundary_3d(
        stack_blocks,
        layer_lowest,
        layer_highest,
        true,
        true,
    );

    *interior_3d = app::metrics::boundary_3d::interior_3d(
        stack_blocks,
        layer_lowest,
        layer_highest,
        true,
        true,
    );

    // update numerical metrics
    *nr_blocks_total = current_layer_blocks.get_nr_blocks();
    *nr_blocks_interior = interior_2d.get_nr_blocks();
    *nr_blocks_boundary = boundary_2d.get_nr_blocks();

    *outer_corners = current_layer_blocks.get_outer_corners();
    *convex_hull = get_convex_hull(&outer_corners);

    *global_bounding_box = stack_layer_config
        .data
        .iter()
        .map(|g_c| exact_squircle_bounds(g_c, 1.1))
        .fold(
            [
                [f64::INFINITY, f64::INFINITY],
                [f64::NEG_INFINITY, f64::NEG_INFINITY],
            ],
            |a, b| square_max(a, b),
        );

    *symmetry_type = current_layer_blocks.get_symmetry_type();

    *center_coord = current_layer_blocks.get_center_coord();

    *global_bounding_box = square_max(
        *global_bounding_box,
        exact_squircle_bounds(&stack_layer_config.get(current_layer).unwrap(), 1.1),
    );
}
