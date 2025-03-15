use crate::app::data_structures::blocks::Blocks;
use crate::app::math::circle_geometry::get_squircle_tangent_point;
use crate::app::math::linear_algebra::{Mat2, Vec2};
use crate::app::math::linear_geometry::intersect_complemented_ray_segment;
use crate::app::math::square::Square;

/// Return blocks object with block contained in the squircle defined by the parameters
pub fn generate_alg_contained(
    center_offset: Vec2,
    sqrt_quad_form: Mat2,
    squircle_parameter: f64,
    grid_size: usize,
) -> Blocks {
    let origin = Blocks::get_origin_float_from_grid_size(grid_size);

    let x_grid_step = sqrt_quad_form * Vec2::UNIT_X;
    let y_grid_step = sqrt_quad_form * Vec2::UNIT_Y;

    let squircle_tangent_x = get_squircle_tangent_point(squircle_parameter, x_grid_step);
    let squircle_tangent_y = get_squircle_tangent_point(squircle_parameter, y_grid_step);

    let blocks = (0..grid_size.pow(2))
        .map(|i| {
            // Loop over all coords
            // Bottom right coordinate of the box in bitmatrix coordinates is [i % edge_length, i / edge_length]
            let square = Square::new(i, grid_size, origin, center_offset, sqrt_quad_form);

            // We have that the box is contained in the disk <=> all corners of the box are in the ellipse
            // Rely on sqrt_quad_form matrix characterization of ellipse
            if squircle_parameter >= 1.0 {
                // Convexity of the squircle with parameter p>=0 gives an easy characterization, just have to check the extreme points
                square.for_all_m_corners(|corner| corner.pnorm(squircle_parameter) <= 1.0)
            } else {
                // See tex pdf

                square.for_all_m_corners(|corner| corner.pnorm(squircle_parameter) <= 1.0)
                    && square.for_all_m_edges(|edge| {
                        !intersect_complemented_ray_segment(
                            [-squircle_tangent_x, squircle_tangent_x],
                            edge,
                        ) && !intersect_complemented_ray_segment(
                            [-squircle_tangent_y, squircle_tangent_y],
                            edge,
                        )
                    })
            }
        })
        .collect();

    Blocks::new(blocks, grid_size)
}
