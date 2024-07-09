use crate::app::helpers::geometry::line_segments_intersect;
use crate::app::helpers::lin_alg::{Mat2, Vec2};
use crate::data_structures::Blocks;

pub fn generate_alg_conservative(radius: f64, center_offset: Vec2, sqrt_quad_form: Mat2) -> Blocks {
    let edge_length = ((2.0 * radius).ceil() as usize) + 4; // the 4 is needed as a buffer..
                                                            // i think we're able to get away with less but it doesn't matter. Buffer is required to make the interior work as expected
    let origin = Vec2::from([(edge_length / 2) as f64, (edge_length / 2) as f64]);
    // in bitmatrix coordinates, where is the center of the grid?

    // The above part is the same for all algorithms (I think at this stage)
    
    // For tilt 0, there is no real need to do this sort of computation: the max x is radius_a,
    // the min x is -radius_a, the max y is radius_b, the min y is radius_a
    // Note point symmetry of the ellipse around 0 gives min_x = -max_x.
    let X = sqrt_quad_form.transpose() * sqrt_quad_form;
    let max_x = Vec2::from([
        (X.d/(X.a * X.d - X.b * X.b)).sqrt(), -(X.b/X.d) * (X.d/(X.a * X.d - X.b * X.b)).sqrt()
    ]);
    let max_y = Vec2::from([
        -(X.b/X.a) * (X.a/(X.a*X.d - X.b * X.b)).sqrt(), (X.a/(X.a*X.d - X.b * X.b)).sqrt()
    ]); // formulas derived algebraically

    // TODO: parallelize using .map() with a very long map
    let blocks = (0..edge_length.pow(2))
        .map(|i| {
            // loop over all coords
            // Step one: Check if any corner points are in the disk (same code as in `contained` but with || instead of &&)

            // Bottom right coordinate of the box in bitmatrix coordinates is [i % edge_length, i / edge_length]
            // We have that the box is contained in the disk <=> all corner of the box are in the disk
            // Want to get at the distance from the corners of a box to the origin + offset (we do this component-wise)
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

            if m_lb.normsq() <= 1.0 || m_rb.normsq() <= 1.0 || m_lt.normsq() <= 1.0 || m_rt.normsq() <= 1.0
            {
                // Any extreme point of the box is in the ellipse (so their intersection is nonempty)
                true
            } else if lb.x <= 0.0 && lb.y <= 0.0 && rt.x >= 0.0 && rt.y >= 0.0 {
                // check if the origin (center of the ellipse) is in the box
                true
            } else if line_segments_intersect([-max_x, max_x], [lb, rb])
                || line_segments_intersect([-max_x, max_x], [rb, rt])
                || line_segments_intersect([-max_x, max_x], [rt, lt])
                || line_segments_intersect([-max_x, max_x], [lt, lb])
                || line_segments_intersect([-max_y, max_y], [lb, rb])
                || line_segments_intersect([-max_y, max_y], [rb, rt])
                || line_segments_intersect([-max_y, max_y], [rt, lt])
                || line_segments_intersect([-max_y, max_y], [lt, lb])
            {
                // check by extreme points
                // (these are the combinations of points on the ellipse where extreme values of x and y are achieved
                //  and edges of the box)
                true
            } else {
                false
            }
    })
    .collect();

    Blocks {
        blocks,
        edge_length,
        origin,
    }
}
