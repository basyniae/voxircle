use crate::data_structures::Blocks;

pub fn generate_alg_centerpoint(radius: f64, center_offset_x: f64, center_offset_y: f64) -> Blocks {
    let edge_length = ((2.0 * radius).ceil() as usize) + 4; // the 4 is needed as a buffer.. i think we're able to get away with less but it doesn't matter
    let origin = [(edge_length / 2) as f64, (edge_length / 2) as f64];
    // in bitmatrix coordinates, where is the center of the grid?
    let mut output_vec = Vec::new();
    // println!("edge_length: {:}, origin: {:}", edge_length, origin[0]); // debugging

    for i in 0..edge_length.pow(2) {
        // loop over all coords
        // Bottom right coordinate of the cell in bitmatrix coordinates is [i % edge_length, i / edge_length]
        // To get centerpoint of the cell, need to add [0.5, 0.5]
        // Want to get at the distance from the centerpoint of a cell to the origin + offset (we do this component-wise)
        let d_x = ((i % edge_length) as f64) + 0.5 - (origin[0] + center_offset_x);
        let d_y = ((i / edge_length) as f64) + 0.5 - (origin[1] + center_offset_y);
        // Easier to look at the squared distance
        output_vec.push(d_x.powi(2) + d_y.powi(2) <= radius.powi(2));
    }

    Blocks {
        blocks: output_vec,
        edge_length,
        origin,
    }
}
