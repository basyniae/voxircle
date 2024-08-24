use crate::app::helpers::blocks::Blocks;
use crate::app::helpers::circle_geometry::get_squircle_tangent_point;
use crate::app::helpers::linear_algebra::{Mat2, Vec2};
use crate::app::helpers::linear_geometry::line_segments_intersect;
use crate::app::helpers::square::Square;

pub fn generate_alg_conservative(
    center_offset: Vec2,
    sqrt_quad_form: Mat2,
    squircle_parameter: f64,
    grid_size: usize,
) -> Blocks {
    let origin = Blocks::get_origin_from_grid_size(grid_size);

    // For tilt 0, there is no real need to do this sort of computation: the max x is radius_a,
    // the min x is -radius_a, the max y is radius_b, the min y is radius_a
    // Note point symmetry of the ellipse around 0 gives min_x = -max_x.

    // FIXME: treat p = infty case separately.... though we've had no issues so far
    let extremize = |v| {
        if squircle_parameter > 1.0 {
            get_squircle_tangent_point(squircle_parameter, sqrt_quad_form * v)
        } else {
            v
        }
    };

    let max_x = extremize(Vec2::UNIT_X);
    let max_y = extremize(Vec2::UNIT_Y);

    let blocks = (0..grid_size.pow(2))
        .map(|i| {
            // loop over all coords
            let square = Square::new(i, grid_size, origin, center_offset, sqrt_quad_form);

            // Any extreme point of the box is in the ellipse (so their intersection is nonempty)
            square.for_any_m_corner(|corner| corner.pnorm(squircle_parameter) <= 1.0)
                // check if the origin (center of the ellipse) is in the box
                ||
                (square.lb.x <= 0.0 && square.lb.y <= 0.0 && square.rt.x >= 0.0 && square.rt.y >= 0.0)
                ||
                {
                    // check by extreme points
                    // (these are the combinations of points on the ellipse where extreme values of x and y are achieved
                    //  and edges of the box)
                    square.for_any_m_edge(|edge| line_segments_intersect([-max_x, max_x], edge))
                        || square.for_any_m_edge(|edge| line_segments_intersect([-max_y, max_y], edge))
                }
    }).collect();

    Blocks::new(blocks, grid_size)
}
