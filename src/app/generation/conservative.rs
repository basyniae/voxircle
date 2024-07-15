use crate::app::helpers::linear_geometry::line_segments_intersect;
use crate::app::helpers::lin_alg::{Mat2, Vec2};
use crate::app::helpers::blocks::Blocks;
use crate::app::helpers::square::Square;

pub fn generate_alg_conservative(radius: f64, center_offset: Vec2, sqrt_quad_form: Mat2) -> Blocks {
    let edge_length = ((2.0 * radius).ceil() as usize) + 4; // the 4 is needed as a buffer..
                                                            // i think we're able to get away with less but it doesn't matter. Buffer is required to make the interior work as expected
    let origin = Vec2::from([(edge_length / 2) as f64, (edge_length / 2) as f64]);
    // in bitmatrix coordinates, where is the center of the grid?

    // The above part is the same for all algorithms (I think at this stage)
    
    // For tilt 0, there is no real need to do this sort of computation: the max x is radius_a,
    // the min x is -radius_a, the max y is radius_b, the min y is radius_a
    // Note point symmetry of the ellipse around 0 gives min_x = -max_x.
    let X = sqrt_quad_form.transpose() * sqrt_quad_form;
    let max_x = Vec2::from([
        (X.d/(X.a * X.d - X.b * X.b)).sqrt(), -(X.b/X.d) * (X.d/(X.a * X.d - X.b * X.b)).sqrt()
    ]);
    let max_y = Vec2::from([
        -(X.b/X.a) * (X.a/(X.a*X.d - X.b * X.b)).sqrt(), (X.a/(X.a*X.d - X.b * X.b)).sqrt()
    ]); // formulas derived algebraically

    let blocks = (0..edge_length.pow(2))
        .map(|i| {
            // loop over all coords
            let square = Square::new(i, edge_length, origin, center_offset, sqrt_quad_form);

            // Any extreme point of the box is in the ellipse (so their intersection is nonempty)
            square.for_any_m_corner(|corner| corner.normsq() <= 1.0)
                // check if the origin (center of the ellipse) is in the box
                || (square.lb.x <= 0.0 && square.lb.y <= 0.0 && square.rt.x >= 0.0 && square.rt.y >= 0.0)
                // check by extreme points
                // (these are the combinations of points on the ellipse where extreme values of x and y are achieved
                //  and edges of the box)
                || square.for_any_edge(|edge| line_segments_intersect([-max_x,max_x], edge))
                || square.for_any_edge(|edge| line_segments_intersect([-max_y,max_y], edge))
            
    })
    .collect();

    Blocks {
        blocks,
        edge_length,
        origin,
    }
}
