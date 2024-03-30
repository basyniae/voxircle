use crate::data_structures::Blocks;

pub fn generate_alg_diamond(radius: f64, center_offset_x: f64, center_offset_y: f64) -> Blocks {
    let edge_length = ((2.0 * radius).ceil() as usize) + 4; // the 4 is needed as a buffer..
    // i think we're able to get away with less but it doesn't matter. Buffer is required to make the interior work as expected
    let origin = [(edge_length / 2) as f64, (edge_length / 2) as f64];
    // in bitmatrix coordinates, where is the center of the grid?
    let mut output_vec = Vec::new();

    // The above part is the same for all algorithms (I think at this stage)

    for i in 0..edge_length.pow(2) {
        // loop over all coords
        let d_x = ((i % edge_length) as f64) + 0.5 - (origin[0] + center_offset_x);
        let d_y = ((i / edge_length) as f64) + 0.5 - (origin[1] + center_offset_y);
        // standard formula for diamond
        output_vec.push(d_x.abs() + d_y.abs() <= radius);
    }

    Blocks {
        blocks: output_vec,
        edge_length,
        origin,
    }
}
