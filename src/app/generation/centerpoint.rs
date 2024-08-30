use crate::app::data_structures::blocks::Blocks;
use crate::app::math::linear_algebra::{Mat2, Vec2};

pub fn generate_alg_centerpoint(
    center_offset: Vec2,
    sqrt_quad_form: Mat2,
    squircle_parameter: f64,
    grid_size: usize,
) -> Blocks {
    let origin = Blocks::get_origin_float_from_grid_size(grid_size);

    let blocks = (0..grid_size.pow(2))
        .map(|i| {
            // loop over all coords
            // Bottom right coordinate of the cell in bitmatrix coordinates is [i % edge_length, i / edge_length]
            // To get centerpoint of the cell, need to add [0.5, 0.5]
            // Want to get at the distance from the centerpoint of a cell to the origin + offset (we do this component-wise)

            let c = Vec2::from([(i % grid_size) as f64 + 0.5, (i / grid_size) as f64 + 0.5])
                - (origin + center_offset);

            // Rely on sqrt_quad_form matrix characterization of ellipse
            let m = sqrt_quad_form * c;
            m.pnorm(squircle_parameter) <= 1.0
        })
        .collect();

    Blocks::new(blocks, grid_size)
}
