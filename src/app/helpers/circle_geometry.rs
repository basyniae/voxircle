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