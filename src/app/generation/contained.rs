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
    let blocks = (0..edge_length.pow(2)).map(|i| {
        // Loop over all coords
        // Bottom right coordinate of the box in bitmatrix coordinates is [i % edge_length, i / edge_length]
        let lb = Vec2::from([(i % edge_length) as f64,
                             (i / edge_length) as f64 ]) - (origin + center_offset);
        let rb = lb + Vec2::from([1.0, 0.0]);
        let lt = lb + Vec2::from([0.0, 1.0]);
        let rt = lb + Vec2::from([1.0, 1.0]);
        
        // Apply the rotate/scale sqrt_quad_form to all corner points LB, RB, LT, RT
        let m_lb = sqrt_quad_form * lb;
        let m_rb = sqrt_quad_form * rb;
        let m_lt = sqrt_quad_form * lt;
        let m_rt = sqrt_quad_form * rt;

        // We have that the box is contained in the disk <=> all corners of the box are in the ellipse
        // Rely on sqrt_quad_form matrix characterization of ellipse
        m_lb.normsq() <= 1.0
            && m_rb.normsq() <= 1.0
            && m_lt.normsq() <= 1.0
            && m_rt.normsq() <= 1.0
    }).collect();

    Blocks {
        blocks,
        edge_length,
        origin,
    }
}