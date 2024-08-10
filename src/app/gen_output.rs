use crate::app::helpers::blocks::Blocks;

#[derive(Default, Debug, Clone)]
pub struct GenOutput {
    pub blocks_all: Blocks,
    pub blocks_boundary: Blocks,
    pub blocks_interior: Blocks,
    pub blocks_complement: Blocks,
}
