use crate::app::helpers::blocks::Blocks;
use crate::app::helpers::circle_geometry::get_squircle_tangent_point;
use crate::app::helpers::lin_alg::{Mat2, Vec2};
use crate::app::helpers::linear_geometry::ray_line_segment_intersect;
use crate::app::helpers::square::Square;

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

    let blocks = (0..edge_length.pow(2))
        .map(|i| {
            // Loop over all coords
            // Bottom right coordinate of the box in bitmatrix coordinates is [i % edge_length, i / edge_length]
            let square = Square::new(i, edge_length, origin, center_offset, sqrt_quad_form);

            // We have that the box is contained in the disk <=> all corners of the box are in the ellipse
            // Rely on sqrt_quad_form matrix characterization of ellipse
            if squircle_parameter >= 1.0 {
                // Convexity of the squircle with parameter p>=0 gives an easy characterization, just have to check the extreme points
                square.for_all_m_corners(|corner| corner.pnorm(squircle_parameter) <= 1.0)
            } else {
                // Case 0 <= p < 1.0
                // The curve of the squircle can poke through the side of the parallelogram
                // So we need that all corners of the parallelogram and the squircle pokes through
                //  none of the sides.
                // Have 4 rays which are never in the squircle and are "optimal with respect to this condition"
                //  in the sense that a box with all corners in the squircle is not fully contained in
                //  the squircle if and only if it intersects *any* of these rays.
                // (The only if direction is the hard one, argue via Rolle's theorem(ish) and direction)

                square.for_all_m_corners(|corner| corner.pnorm(squircle_parameter) <= 1.0)
                    && !square.for_any_m_edge(|edge| {
                        ray_line_segment_intersect(
                            [squircle_tangent_x, 2.0 * squircle_tangent_x],
                            edge,
                        )
                    })
                    && !square.for_any_m_edge(|edge| {
                        ray_line_segment_intersect(
                            [-squircle_tangent_x, -2.0 * squircle_tangent_x],
                            edge,
                        )
                    })
                    && !square.for_any_m_edge(|edge| {
                        ray_line_segment_intersect(
                            [squircle_tangent_y, 2.0 * squircle_tangent_y],
                            edge,
                        )
                    })
                    && !square.for_any_m_edge(|edge| {
                        ray_line_segment_intersect(
                            [-squircle_tangent_y, -2.0 * squircle_tangent_y],
                            edge,
                        )
                    })
            }
        })
        .collect();

    Blocks {
        blocks,
        edge_length,
        origin,
    }
}
