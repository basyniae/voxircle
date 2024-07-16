use crate::app::helpers::lin_alg::Vec2;
use std::f64::consts::PI;

// With -radius <= x0 <= x1 <= radius, what is the area of the semicircle y=sqrt(R^2-x^2) from x0 to x1?
// Have an exact primitive for non-edge cases
pub fn area_of_semicircle_section(x0: f64, x1: f64, radius: f64) -> f64 {
    if !(-radius <= x0 && x0 <= x1 && x1 <= radius) {
        // -radius <= x0 <= x1 <= radius
        panic!("ooo...");
    }

    // Primitive in case |x| != R
    fn primitive(x: f64, radius: f64) -> f64 {
        let sq = (radius.powi(2) - x.powi(2)).sqrt();
        0.5 * (x * sq + radius.powi(2) * (x / sq).atan())
    }

    let area_x0_to_0: f64;
    if x0 <= -radius {
        area_x0_to_0 = (-radius * PI) / 4.0;
    } else if x0 >= radius {
        area_x0_to_0 = (radius * PI) / 4.0;
    } else {
        area_x0_to_0 = primitive(x0, radius);
    }

    let area_x1_to_0: f64;
    if x1 <= -radius {
        area_x1_to_0 = (-radius * PI) / 4.0;
    } else if x0 >= radius {
        area_x1_to_0 = (radius * PI) / 4.0;
    } else {
        area_x1_to_0 = primitive(x1, radius);
    }

    area_x1_to_0 - area_x0_to_0 // fundamental theorem of calculus (finite additivity of integral)
}

/// Given a circle of given radius centered at (0,0), and a y level, return the pair of intersections that result from intersecting the circle and line
/// First the negative, then the positive
pub fn intersection_hline_circle(y: f64, radius: f64) -> [f64; 2] {
    let positive_intersection = (radius.powi(2) - y.powi(2)).sqrt();
    [-positive_intersection, positive_intersection]
}

// Get the point on the unit squircle where the curve is tangent to the input. There are in general
//  two such points. View the squircle as a parameterized curve counterclockwise. Then output the
//  point where the direction of the derivative matches the direction vector.
// Equivalently, the direction rotated 90° clockwise is the steepest ascent vector at the output
//  (of the p-norm ||(x,y)||_p).
pub fn get_squircle_tangent_point(squircle_parameter: f64, direction: Vec2) -> Vec2 {
    let p = squircle_parameter;
    let abs_y = {
        if direction.x == 0.0 {
            0.0
        } else {
            (1.0 + (direction.y / direction.x).abs().powf(p / (p - 1.0))).powf(-1.0 / p)
            // algebraic expression, see notebook for derivation.
            // Follows from inner product of Del ||(x,y)||_p^p with direction being 0
            // and the normalization ||(x,y)||_p=1
        }
    };

    let abs_x = {
        if direction.x == 0.0 {
            1.0
        } else {
            (1.0 - abs_y.powf(p)).powf(1.0 / p)
        }
    };

    let x = direction.x.signum() * abs_x;
    let y = -direction.y.signum() * abs_y;

    Vec2::from([x, y])
}
