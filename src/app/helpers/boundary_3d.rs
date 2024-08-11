use crate::app::gen_output::GenOutput;
use crate::app::helpers::blocks::Blocks;
use std::collections::VecDeque;

pub fn boundary_3d(
    gen_output: &VecDeque<GenOutput>,
    layer_min: isize,
    layer_max: isize,
    floating_bottom: bool,
    floating_top: bool,
) -> Vec<Blocks> {
    let out = (layer_min..=layer_max)
        .map(|layer| {
            let blocks = gen_output[(layer - layer_min) as usize].clone().blocks_all;

            Blocks {
                blocks: (0..blocks.grid_size.pow(2))
                    .map(|i| {
                        blocks.blocks[i] == true
                // has to be a block in self
                && (
                i % blocks.grid_size == 0
                || i % blocks.grid_size == blocks.grid_size - 1
                || i / blocks.grid_size == 0
                || i / blocks.grid_size == blocks.grid_size - 1
                // edges of layer boundary are automatically boundary  (extreme coords)
                || !(blocks.blocks[i + 1]
                && blocks.blocks[i - 1]
                && blocks.blocks[i + blocks.grid_size]
                && blocks.blocks[i - blocks.grid_size])
                // regular 2D boundary
                || (layer == layer_min && floating_bottom)
                || (layer == layer_max && floating_top)
                // top and bottom faces of stack
                || gen_output.get((layer + 1 - layer_min) as usize).is_some_and(|blocks_above| blocks_above.blocks_all.blocks[i] == false)
                || gen_output.get((layer - 1 - layer_min) as usize).is_some_and(|blocks_below| blocks_below.blocks_all.blocks[i] == false)
            )
                        // regular vertical boundary
                    })
                    .collect(),
                ..blocks
            }
        })
        .collect();

    out
}
