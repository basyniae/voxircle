use crate::app::helpers::lin_alg::Vec2;
use crate::data_structures::Blocks;

pub fn generate_alg_square(radius: f64, center_offset: Vec2) -> Blocks {
    let edge_length = ((2.0 * radius).ceil() as usize) + 4; // the 4 is needed as a buffer..
    // i think we're able to get away with less but it doesn't matter. Buffer is required to make the interior work as expected
    let origin = Vec2::from([(edge_length / 2) as f64, (edge_length / 2) as f64]);
    // in bitmatrix coordinates, where is the center of the grid?
    
    let blocks = (0..edge_length.pow(2)).map(|i| {
        // loop over all coords
        let c = Vec2::from([
            (i % edge_length) as f64 + 0.5,
            (i / edge_length) as f64 + 0.5,
        ]) - (origin + center_offset);
        // Standard formula for square
        c.infnorm() <= radius
    }).collect();

    Blocks {
        blocks,
        edge_length,
        origin,
    }
}
