use crate::app;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::slice_parameters::SliceParameters;
use crate::app::data_structures::sparse_blocks::SparseBlocks;
use crate::app::data_structures::symmetry_type::SymmetryType;
use crate::app::data_structures::zvec::ZVec;
use crate::app::math::exact_squircle_bounds::exact_squircle_bounds;
use crate::app::math::square_max::square_max;
use app::metrics::convex_hull::get_convex_hull;

#[derive(Default)] // todo: should probably do this manually... see previous versions of code for that
pub struct Metrics {
    // Metrics todo: make private?
    pub nr_blocks_total: u64,
    pub nr_blocks_interior: u64,
    pub nr_blocks_boundary: u64,

    pub boundary_2d: Blocks,
    pub boundary_conn_comp: Vec<SparseBlocks>,
    pub interior_2d: Blocks,
    pub complement_2d: Blocks,
    pub boundary_3d: ZVec<Blocks>,
    pub interior_3d: ZVec<Blocks>,
    pub convex_hull: Vec<[f64; 2]>,
    pub outer_corners: Vec<[f64; 2]>,
    pub symmetry_type: SymmetryType,
    pub center_coord: [f64; 2],

    pub global_bounding_box: [[f64; 2]; 2],
}

// todo: make a struct containing all these parameters for easier maintenance
impl Metrics {
    pub fn update_metrics(
        &mut self,
        current_layer: isize,
        layer_lowest: isize,
        layer_highest: isize,
        current_layer_blocks: Blocks,
        stack_blocks: &ZVec<Blocks>,
        stack_layer_config: &ZVec<SliceParameters>,
    ) {
        // update 2d spatial metrics
        self.interior_2d = current_layer_blocks.get_interior();
        self.boundary_2d = current_layer_blocks.get_boundary();
        self.complement_2d = current_layer_blocks.get_complement();
        self.boundary_conn_comp =
            (SparseBlocks::from(self.boundary_2d.clone())).connected_components();

        // update 3d spatial metrics
        self.boundary_3d = app::metrics::boundary_3d::boundary_3d(
            stack_blocks,
            layer_lowest,
            layer_highest,
            true,
            true,
        );

        self.interior_3d = app::metrics::boundary_3d::interior_3d(
            stack_blocks,
            layer_lowest,
            layer_highest,
            true,
            true,
        );

        // update numerical metrics
        self.nr_blocks_total = current_layer_blocks.get_nr_blocks();
        self.nr_blocks_interior = self.interior_2d.get_nr_blocks();
        self.nr_blocks_boundary = self.boundary_2d.get_nr_blocks();

        self.outer_corners = current_layer_blocks.get_outer_corners();
        self.convex_hull = get_convex_hull(&self.outer_corners);

        self.global_bounding_box = stack_layer_config
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

        self.symmetry_type = current_layer_blocks.get_symmetry_type();

        self.center_coord = current_layer_blocks.get_center_coord();

        self.global_bounding_box = square_max(
            self.global_bounding_box,
            exact_squircle_bounds(&stack_layer_config.get(current_layer).unwrap(), 1.1),
        );
    }
}
