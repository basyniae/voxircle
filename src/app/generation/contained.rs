use crate::app::helpers::lin_alg::{Mat2, Vec2};
use crate::data_structures::Blocks;

pub fn generate_alg_contained(
    center_offset: Vec2,
    sqrt_quad_form: Mat2,
    radius_major: f64,
) -> Blocks {
    let edge_length = ((2.0 * radius_major).ceil() as usize) + 4; // the 4 is needed as a buffer..
                                                                  // i think we're able to get away with less but it doesn't matter. Buffer is required to make the interior work as expected
    let origin = Vec2::from([(edge_length / 2) as f64, (edge_length / 2) as f64]);
    // in bitmatrix coordinates, where is the center of the grid?
    let mut output_vec = Vec::new();

    // The above part is the same for all algorithms (I think at this stage)

    for i in 0..edge_length.pow(2) {
        // loop over all coords
        // Bottom right coordinate of the box in bitmatrix coordinates is [i % edge_length, i / edge_length]
        // We have that the box is contained in the disk <=> all corner of the box are in the disk
        // Want to get at the distance from the corners of a box to the origin + offset (we do this component-wise)
        let d_x_left = ((i % edge_length) as f64) - (origin.x + center_offset.x);
        let d_x_right = ((i % edge_length) as f64) + 1.0 - (origin.x + center_offset.x);
        let d_y_bottom = ((i / edge_length) as f64) - (origin.y + center_offset.y);
        let d_y_top = ((i / edge_length) as f64) + 1.0 - (origin.y + center_offset.y);

        let dLB = Vec2::from([d_x_left, d_y_bottom]);
        let dRB = Vec2::from([d_x_right, d_y_bottom]);
        let dLT = Vec2::from([d_x_left, d_y_top]);
        let dRT = Vec2::from([d_x_right, d_y_top]);

        // Apply the rotate/scale sqrt_quad_form to all corner points LB, RB, LT, RT

        let mLB = sqrt_quad_form * dLB;
        let mRB = sqrt_quad_form * dRB;
        let mLT = sqrt_quad_form * dLT;
        let mRT = sqrt_quad_form * dRT;

        // Easier to look at the squared distance
        output_vec.push(
            mLB.normsq() <= 1.0
                && mRB.normsq() <= 1.0
                && mLT.normsq() <= 1.0
                && mRT.normsq() <= 1.0,
        );
    }

    Blocks {
        blocks: output_vec,
        edge_length,
        origin,
    }
}
