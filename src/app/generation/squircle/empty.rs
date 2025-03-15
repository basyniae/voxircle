use crate::app::data_structures::blocks::Blocks;

// generate empty blocks with correct grid size
pub fn generate_alg_empty(grid_size: usize) -> Blocks {
    let blocks = (0..grid_size.pow(2)).map(|_| false).collect();

    Blocks::new(blocks, grid_size)
}
