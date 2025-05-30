use crate::app;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::AllParams;
use crate::app::math::square_max::square_max;
use crate::app::metrics::sparse_blocks::SparseBlocks;
use crate::app::metrics::symmetry_type::SymmetryType;
use app::metrics::convex_hull::get_convex_hull;
use egui::Color32;

#[derive(Default)]
pub struct Metrics {
    pub nr_blocks_total: u64,
    pub nr_blocks_interior: u64,
    pub nr_blocks_boundary: u64,

    pub boundary_2d: Blocks,
    pub boundary_conn_comp: Vec<(SparseBlocks, SparseBlocks, Color32)>,
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

impl Metrics {
    pub fn update(
        &mut self,
        current_layer: isize,
        layer_lowest: isize,
        layer_highest: isize,
        current_layer_blocks: &Blocks,
        stack_blocks: &ZVec<Blocks>,
        stack_layer_config: &ZVec<AllParams>,
    ) {
        // get boundary and interior and complement
        self.interior_2d = current_layer_blocks.get_interior();
        self.boundary_2d = current_layer_blocks.get_boundary();
        self.complement_2d = current_layer_blocks.get_complement();

        // get connected components with right tags
        // two maps instead of one so we don't have to compute the normal form twice
        // the cloning is still weird
        self.boundary_conn_comp = SparseBlocks::from(self.boundary_2d.clone())
            .connected_components()
            .iter()
            .map(|conn_comp| (conn_comp.clone(), conn_comp.normal_form()))
            .map(|(conn_comp, normal_form)| {
                (
                    conn_comp,
                    normal_form.clone(),
                    normal_form.hash_color_from_normal_form(),
                )
            })
            .collect();

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
            .map(|params| params.bounds(1.1))
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
            stack_layer_config.get(current_layer).unwrap().bounds(1.1),
        );
    }
}
