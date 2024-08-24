use crate::app::helpers::circle_geometry::get_squircle_tangent_point;
use crate::app::helpers::gen_config::GenConfig;
use crate::app::helpers::linear_algebra::Vec2;

pub fn exact_squircle_bounds(gen_config: &GenConfig, pad_factor: f64) -> [[f64; 2]; 2] {
    let squircle_parameter = gen_config.squircle_parameter;
    let sqrt_quad_form = gen_config.get_sqrt_quad_form();
    let center_offset_x = gen_config.center_offset_x;
    let center_offset_y = gen_config.center_offset_y;

    if gen_config.radius_a == 0.0 || gen_config.radius_b == 0.0 {
        [
            [-1.0 + center_offset_x, -1.0 + center_offset_y],
            [1.0 + center_offset_x, 1.0 + center_offset_y],
        ]
    } else {
        let m_a = {
            if squircle_parameter > 1.0 {
                get_squircle_tangent_point(
                    squircle_parameter,
                    sqrt_quad_form * Vec2::from([1.0, 0.0]),
                )
            } else {
                // don't care about which values are minimized / maximized since it's easy to compute
                Vec2::from([1.0, 0.0])
            }
        };
        let m_b = {
            if squircle_parameter > 1.0 {
                get_squircle_tangent_point(
                    squircle_parameter,
                    sqrt_quad_form * Vec2::from([0.0, 1.0]),
                )
            } else {
                Vec2::from([0.0, 1.0])
            }
        };

        let a = pad_factor * sqrt_quad_form.inverse().unwrap() * m_a;
        let b = pad_factor * sqrt_quad_form.inverse().unwrap() * m_b;

        let max = [a.x.abs().max(b.x.abs()), a.y.abs().max(b.y.abs())];
        let min = [-max[0], -max[1]];
        [
            [min[0] + center_offset_x, min[1] + center_offset_y],
            [max[0] + center_offset_x, max[1] + center_offset_y],
        ]
    }
}
