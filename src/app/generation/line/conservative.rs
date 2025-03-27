use crate::app::data_structures::blocks::Blocks;
use crate::app::math::linear_algebra::Vec2;
use crate::app::math::linear_geometry::{dist_to_line, intersect_segment_segment};
use crate::app::math::square::Square;

pub fn generate_alg_conservative(
    rise_run: Vec2,
    offset: Vec2,
    thickness: f64,
    length: f64,
    grid_size: usize,
) -> Blocks {
    let origin = Blocks::get_origin_float_from_grid_size(grid_size);

    let max_x = (length / 2.0) * rise_run.normalize()
        + (thickness / 2.0) * rise_run.normalize().rot_90_CCW();
    let max_y = (length / 2.0) * rise_run.normalize()
        - (thickness / 2.0) * rise_run.normalize().rot_90_CCW();

    let blocks = (0..grid_size.pow(2))
        .map(|i| {
            let square = Square::new_straight(i, grid_size, origin, offset);

            square.for_any_m_corner(|corner| {
                dist_to_line(corner, rise_run, offset) <= thickness / 2.0
                    && dist_to_line(corner, rise_run.rot_90_CCW(), offset) <= length / 2.0
            }) || (square.lb.x <= 0.0
                && square.lb.y <= 0.0
                && square.rt.x >= 0.0
                && square.rt.y >= 0.0)
                || {
                    // check by extreme points
                    // (these are the combinations of points on the ellipse where extreme values of x and y are achieved
                    //  and edges of the box)
                    square.for_any_m_edge(|edge| intersect_segment_segment([-max_x, max_x], edge))
                        || square
                            .for_any_m_edge(|edge| intersect_segment_segment([-max_y, max_y], edge))
                }
        })
        .collect();

    Blocks::new(blocks, grid_size)
}
