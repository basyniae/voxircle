use crate::app::generation::{generate_all_blocks, Algorithm};
use crate::app::helpers::blocks::Blocks;
use crate::app::helpers::gen_output::GenOutput;
use crate::app::helpers::linear_algebra::{Mat2, Vec2};
use std::ops::Not;

#[derive(Debug, Clone)]
pub struct GenConfig {
    pub algorithm: Algorithm,

    pub radius_a: f64, // These two are specified (a is the x-axis if tilt = 0)
    pub radius_b: f64,
    pub radius_major: f64, // The algorithms which do not (yet) support ellipses use larger radius
    pub radius_minor: f64,
    pub tilt: f64,

    pub sqrt_quad_form: Mat2,
    // The square root of the PSD symmetric quadratic form X defining the ellipse:
    //  (x,y)^TX(x,y)=1
    // store [a,b,c,d] for [[a,b],[c,d]] (obviously)
    pub circle_mode: bool,

    pub squircle_parameter: f64,
    pub squircle_ui_parameter: f64,

    pub center_offset_x: f64,
    pub center_offset_y: f64,
}

impl Default for GenConfig {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::CenterPoint, // default: Centerpoint

            radius_a: 5.0, // default: 5.0
            radius_b: 5.0, // default: 5.0
            radius_major: Default::default(),
            radius_minor: Default::default(),

            tilt: 0.0, // default: 0.0

            sqrt_quad_form: Mat2::from([1.0, 0.0, 0.0, 1.0]),

            center_offset_x: Default::default(),
            center_offset_y: Default::default(),

            circle_mode: true, // default: true

            squircle_parameter: 2.0, // default: 2.0 (circle / ellipse)
            squircle_ui_parameter: 0.666666666666666, // default: 0.666666666666666
        }
    }
}

impl GenConfig {
    pub fn generate(&self) -> GenOutput {
        // Determine grid size
        // The major radius should be included, for some metrics we need at least one layer of padding
        //  around the generated figure. Assuming a square figure (squircle parameter infinity), we
        //  need an x side length of 2.0 * sqrt(2) * radius_major. Add 4 for a padding of at least 2
        //  on each side.
        let grid_size = (2.0 * 1.42 * f64::max(self.radius_a, self.radius_b)).ceil() as usize + 4;

        // In bitmatrix coordinates, where is the point (0,0)? (Note that it has integer coordinates)
        let origin = Vec2::from([(grid_size / 2) as f64, (grid_size / 2) as f64]);

        // Generate from circle with selected algorithm
        let blocks_all = generate_all_blocks(
            &self.algorithm,
            Vec2::from([self.center_offset_x, self.center_offset_y]),
            self.sqrt_quad_form,
            self.squircle_parameter,
            self.tilt,
            self.radius_a,
            self.radius_b,
            grid_size,
            origin,
        );

        // run preprocessing
        let blocks_interior = blocks_all.get_interior();
        let blocks_boundary = Blocks {
            // boundary is in all but not in interior (so all && interior.not())
            blocks: blocks_all
                .blocks
                .iter()
                .zip(blocks_interior.blocks.iter())
                .map(|(all, interior)| *all && interior.not())
                .collect(),
            grid_size: blocks_all.grid_size,
            origin: blocks_all.origin,
        };
        let blocks_complement = blocks_all.get_complement();

        GenOutput {
            blocks_all,
            blocks_interior,
            blocks_boundary,
            blocks_complement,
        }
        // update metrics
        // let nr_blocks_total = self.blocks_all.get_nr_blocks();
        // let nr_blocks_interior = self.blocks_interior.get_nr_blocks();
        // let nr_blocks_boundary = self.blocks_boundary.get_nr_blocks();
        //
        // self.outer_corners = self.blocks_all.get_outer_corners();
        // self.convex_hull = get_convex_hull(&self.outer_corners);
    }
}
