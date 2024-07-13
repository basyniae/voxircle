use crate::app::helpers::lin_alg::{Mat2, Vec2};
use crate::data_structures::Blocks;

pub fn generate_alg_centerpoint(
    center_offset: Vec2,
    sqrt_quad_form: Mat2,
    radius_major: f64,
    squircle_parameter: f64,
) -> Blocks {
    let edge_length = (1.5 * 2.0 * radius_major).ceil() as usize; // the 1.5 is needed as a buffer (could theoretically be sqrt(2))
    let origin = Vec2::from([(edge_length / 2) as f64, (edge_length / 2) as f64]);
    // in bitmatrix coordinates, where is the center of the grid?

    let blocks = (0..edge_length.pow(2))
        .map(|i| {
            // loop over all coords
            // Bottom right coordinate of the cell in bitmatrix coordinates is [i % edge_length, i / edge_length]
            // To get centerpoint of the cell, need to add [0.5, 0.5]
            // Want to get at the distance from the centerpoint of a cell to the origin + offset (we do this component-wise)

            let c = Vec2::from([
                (i % edge_length) as f64 + 0.5,
                (i / edge_length) as f64 + 0.5,
            ]) - (origin + center_offset);

            // Rely on sqrt_quad_form matrix characterization of ellipse
            let m = sqrt_quad_form * c;
            m.pnorm(squircle_parameter) <= 1.0
        })
        .collect();

    Blocks {
        blocks,
        edge_length,
        origin,
    }
}
