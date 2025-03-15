use crate::app::data_structures::blocks::Blocks;
use crate::app::math::linear_algebra::Vec2;
use crate::app::math::linear_geometry::dist_to_line;

pub fn generate_line_centerpoint(
    rise: f64,
    run: f64,
    offset_y: f64,
    thickness: f64,
    grid_size: usize,
) -> Blocks {
    let origin = Blocks::get_origin_float_from_grid_size(grid_size);

    let blocks = (0..grid_size.pow(2))
        .map(|i| {
            let c =
                Vec2::from([(i % grid_size) as f64 + 0.5, (i / grid_size) as f64 + 0.5]) - origin;

            dist_to_line(c, rise, run, offset_y) <= thickness
        })
        .collect();

    Blocks::new(blocks, grid_size)
}
