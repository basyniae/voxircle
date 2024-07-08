use crate::app::helpers::geometry::line_segments_intersect;
use crate::app::helpers::lin_alg::{Mat2, Vec2};
use crate::data_structures::Blocks;

pub fn generate_alg_conservative(radius: f64, center_offset: Vec2, sqrt_quad_form: Mat2) -> Blocks {
    let edge_length = ((2.0 * radius).ceil() as usize) + 4; // the 4 is needed as a buffer..
                                                            // i think we're able to get away with less but it doesn't matter. Buffer is required to make the interior work as expected
    let origin = Vec2::from([(edge_length / 2) as f64, (edge_length / 2) as f64]);
    // in bitmatrix coordinates, where is the center of the grid?

    // The above part is the same for all algorithms (I think at this stage)

    // FIXME: Compute points on the ellipse with extreme x or y coordinate (taking 0 as the origin)
    let maxx = Vec2::from([1.0, 0.0]);
    let minx = Vec2::from([1.0, 0.0]);
    let maxy = Vec2::from([1.0, 0.0]);
    let miny = Vec2::from([1.0, 0.0]);

    let O = Vec2::from([0.0, 0.0]);

    // TODO: parallelize using .map() with a very long map
    let mut output_vec = Vec::new();
    for i in 0..edge_length.pow(2) {
        // loop over all coords
        // Step one: Check if any corner points are in the disk (same code as in `contained` but with || instead of &&)

        // Bottom right coordinate of the box in bitmatrix coordinates is [i % edge_length, i / edge_length]
        // We have that the box is contained in the disk <=> all corner of the box are in the disk
        // Want to get at the distance from the corners of a box to the origin + offset (we do this component-wise)
        let d_x_left = ((i % edge_length) as f64) - (origin.x + center_offset.x);
        let d_x_right = ((i % edge_length) as f64) + 1.0 - (origin.x + center_offset.x);
        let d_y_bottom = ((i / edge_length) as f64) - (origin.y + center_offset.y);
        let d_y_top = ((i / edge_length) as f64) + 1.0 - (origin.y + center_offset.y);

        let dLB = Vec2::from([d_x_left, d_y_bottom]);
        let dRB = Vec2::from([d_x_right, d_y_bottom]);
        let dLT = Vec2::from([d_x_left, d_y_top]);
        let dRT = Vec2::from([d_x_right, d_y_top]);

        // Apply the rotate/scale sqrt_quad_form to all corner points LB, RB, LT, RT

        let mLB = sqrt_quad_form * dLB;
        let mRB = sqrt_quad_form * dRB;
        let mLT = sqrt_quad_form * dLT;
        let mRT = sqrt_quad_form * dRT;

        if mLB.normsq() <= 1.0 || mRB.normsq() <= 1.0 || mLT.normsq() <= 1.0 || mRT.normsq() <= 1.0
        {
            // Any extreme point of the box is in the ellipse (so their intersection is nonempty)
            output_vec.push(true);
        // } else if d_x_left <= 0.0 || d_x_right >= 0.0 || d_y_top >= 0.0 || d_y_bottom <= 0.0 {
        //     // check if the origin (center of the ellipse) is in the box
        //     output_vec.push(true);
        //     // TODO: implement some heuristic... so that not all boxes have to do the 16 line checks
        } else if line_segments_intersect([O, maxx], [dLB, dRB])
            || line_segments_intersect([O, maxx], [dRB, dRT])
            || line_segments_intersect([O, maxx], [dRT, dLT])
            || line_segments_intersect([O, maxx], [dLT, dLB])
            || line_segments_intersect([O, minx], [dLB, dRB])
            || line_segments_intersect([O, minx], [dRB, dRT])
            || line_segments_intersect([O, minx], [dRT, dLT])
            || line_segments_intersect([O, minx], [dLT, dLB])
            || line_segments_intersect([O, maxy], [dLB, dRB])
            || line_segments_intersect([O, maxy], [dRB, dRT])
            || line_segments_intersect([O, maxy], [dRT, dLT])
            || line_segments_intersect([O, maxy], [dLT, dLB])
            || line_segments_intersect([O, miny], [dLB, dRB])
            || line_segments_intersect([O, miny], [dRB, dRT])
            || line_segments_intersect([O, miny], [dRT, dLT])
            || line_segments_intersect([O, miny], [dLT, dLB])
        {
            // check by extreme points
            // (these are the combinations of the extreme points x,y points of the ellipse and edges of the box)
            output_vec.push(true);
        } else {
            output_vec.push(false);
        }
    }

    Blocks {
        blocks: output_vec,
        edge_length,
        origin,
    }
}
