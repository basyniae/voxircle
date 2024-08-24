use crate::app::helpers::blocks::Blocks;
use crate::app::helpers::zvec::ZVec;

/// document
pub fn boundary_3d(
    gen_output: &ZVec<Blocks>,
    layer_min: isize,
    layer_max: isize,
    floating_bottom: bool,
    floating_top: bool,
) -> ZVec<Blocks> {
    let out = (layer_min..=layer_max)
        .map(|layer| {
            let blocks = gen_output.get(layer).clone().unwrap();

            Blocks::new(
                (0..blocks.grid_size.pow(2))
                    .map(|i| {
                        let global_coords = blocks.get_global_coord_usize_from_index(i);

                        blocks.blocks[i] == true
                            // has to be a block in self
                            && (

                            i % blocks.grid_size == 0
                                // edges of layer boundary are automatically boundary  (extreme coords):
                                || i % blocks.grid_size == blocks.grid_size - 1
                                || i / blocks.grid_size == 0
                                || i / blocks.grid_size == blocks.grid_size - 1
                                // regular 2D boundary: (look in each horizontal direction, any must be empty for i not to be a boundary
                                || !blocks.blocks[i + 1]
                                || !blocks.blocks[i - 1]
                                || !blocks.blocks[i + blocks.grid_size]
                                || !blocks.blocks[i - blocks.grid_size]
                                // top and bottom faces of stack: (
                                || (layer == layer_min && floating_bottom)
                                || (layer == layer_max && floating_top)
                                // vertical boundary: see if 1. the layer above exists, 2. there is no block at the global coords on the layer above
                                || gen_output.get(layer + 1).is_some_and(|layer_above| !layer_above.is_block_on_global_coord(global_coords))
                                || gen_output.get(layer - 1).is_some_and(|layer_above| !layer_above.is_block_on_global_coord(global_coords))
                            )
                    })
                    .collect(), blocks.grid_size)
            }
        )
        .collect();

    ZVec::new(out, layer_min)
}
