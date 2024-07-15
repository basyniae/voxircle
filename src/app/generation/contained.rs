use crate::app::helpers::circle_geometry::get_squircle_tangent_point;
use crate::app::helpers::lin_alg::{Mat2, Vec2};
use crate::app::helpers::linear_geometry::ray_line_segment_intersect;
use crate::data_structures::Blocks;

pub fn generate_alg_contained(
    center_offset: Vec2,
    sqrt_quad_form: Mat2,
    radius_major: f64,
    squircle_parameter: f64,
) -> Blocks {
    let edge_length = ((2.0 * radius_major).ceil() as usize) + 4; // the 4 is needed as a buffer..
                                                                  // i think we're able to get away with less but it doesn't matter. Buffer is required to make the interior work as expected
    let origin = Vec2::from([(edge_length / 2) as f64, (edge_length / 2) as f64]);
    // in bitmatrix coordinates, where is the center of the grid?

    let x_grid_step = sqrt_quad_form * Vec2::from([1.0, 0.0]);
    let y_grid_step = sqrt_quad_form * Vec2::from([0.0, 1.0]);

    let squircle_tangent_x = get_squircle_tangent_point(squircle_parameter, x_grid_step);
    let squircle_tangent_y = get_squircle_tangent_point(squircle_parameter, y_grid_step);

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
        if squircle_parameter >= 1.0 {
            // Convexity of the squircle with parameter p>=0 gives an easy characterization, just have to check the extreme points
            m_lb.pnorm(squircle_parameter) <= 1.0
                && m_rb.pnorm(squircle_parameter) <= 1.0
                && m_lt.pnorm(squircle_parameter) <= 1.0
                && m_rt.pnorm(squircle_parameter) <= 1.0
        } else { // Case 0 <= p < 1.0
            // The curve of the squircle can poke through the side of the parallelogram
            // So we need that all corners of the parallelogram and the squircle pokes through
            //  none of the sides.
            // Have 4 rays which are never in the squircle and are "optimal with respect to this condition"
            //  in the sense that a box with all corners in the squircle is not fully contained in
            //  the squircle if and only if it intersects *any* of these rays.
            // (The only if direction is the hard one, argue via Rolle's theorem(ish) and direction)

            // TODO: Refactor so that we can easily loop over all edges or vertices in a box. We do this a ton of times so worth it
            m_lb.pnorm(squircle_parameter) <= 1.0
                && m_rb.pnorm(squircle_parameter) <= 1.0
                && m_lt.pnorm(squircle_parameter) <= 1.0
                && m_rt.pnorm(squircle_parameter) <= 1.0
                && !ray_line_segment_intersect([squircle_tangent_x, 2.0*squircle_tangent_x], [m_lb, m_rb])
                && !ray_line_segment_intersect([squircle_tangent_x, 2.0*squircle_tangent_x], [m_rb, m_rt])
                && !ray_line_segment_intersect([squircle_tangent_x, 2.0*squircle_tangent_x], [m_rt, m_lt])
                && !ray_line_segment_intersect([squircle_tangent_x, 2.0*squircle_tangent_x], [m_lt, m_lb])
                && !ray_line_segment_intersect([-squircle_tangent_x, -2.0*squircle_tangent_x], [m_lb, m_rb])
                && !ray_line_segment_intersect([-squircle_tangent_x, -2.0*squircle_tangent_x], [m_rb, m_rt])
                && !ray_line_segment_intersect([-squircle_tangent_x, -2.0*squircle_tangent_x], [m_rt, m_lt])
                && !ray_line_segment_intersect([-squircle_tangent_x, -2.0*squircle_tangent_x], [m_lt, m_lb])
                && !ray_line_segment_intersect([squircle_tangent_y, 2.0*squircle_tangent_y], [m_lb, m_rb])
                && !ray_line_segment_intersect([squircle_tangent_y, 2.0*squircle_tangent_y], [m_rb, m_rt])
                && !ray_line_segment_intersect([squircle_tangent_y, 2.0*squircle_tangent_y], [m_rt, m_lt])
                && !ray_line_segment_intersect([squircle_tangent_y, 2.0*squircle_tangent_y], [m_lt, m_lb])
                && !ray_line_segment_intersect([-squircle_tangent_y, -2.0*squircle_tangent_y], [m_lb, m_rb])
                && !ray_line_segment_intersect([-squircle_tangent_y, -2.0*squircle_tangent_y], [m_rb, m_rt])
                && !ray_line_segment_intersect([-squircle_tangent_y, -2.0*squircle_tangent_y], [m_rt, m_lt])
                && !ray_line_segment_intersect([-squircle_tangent_y, -2.0*squircle_tangent_y], [m_lt, m_lb])
        }
    }).collect();

    Blocks {
        blocks,
        edge_length,
        origin,
    }
}

