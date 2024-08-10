use crate::app::helpers::blocks::Blocks;
use crate::app::helpers::linear_algebra::Vec2;

// generate empty blocks with correct grid size
pub fn generate_alg_empty(grid_size: usize, origin: Vec2) -> Blocks {
    let blocks = (0..grid_size.pow(2)).map(|_| false).collect();

    Blocks {
        blocks,
        grid_size,
        origin,
    }
}
