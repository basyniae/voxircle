use std::cmp::Ordering;

use crate::app::math::linear_algebra::Vec2;

/// Return true if the closed line segments intersect, false otherwise.
/// From http://www.dcs.gla.ac.uk/~pat/52233/slides/Geometry1x1.pdf p.6
pub fn intersect_segment_segment(line_one: [Vec2; 2], line_two: [Vec2; 2]) -> bool {
    let p_1 = line_one[0];
    let q_1 = line_one[1];
    let p_2 = line_two[0];
    let q_2 = line_two[1];

    ((orient_triple(p_1, q_1, p_2) != orient_triple(p_1, q_1, q_2)) // General case
        && (orient_triple(p_2, q_2, p_1) != orient_triple(p_2, q_2, q_1)))
        || ((orient_triple(p_1, q_1, p_2) == Orientation::Collinear) // Special case
        && (orient_triple(p_1, q_1, q_2) == Orientation::Collinear)
        && (orient_triple(p_2, q_2, p_1) == Orientation::Collinear)
        && (orient_triple(p_2, q_2, q_1) == Orientation::Collinear)
        && intervals_intersect([p_1.x, q_1.x], [p_2.x, q_2.x])
        && intervals_intersect([p_1.y, q_1.y], [p_2.y, q_2.y]))
}

/// Return true if the closed line segment has nonempty intersection with the part of the line defined by the line segment that is not in the line segment itself
pub fn intersect_complemented_ray_segment(
    complemented_segment: [Vec2; 2],
    segment: [Vec2; 2],
) -> bool {
    let p_1 = complemented_segment[0];
    let q_1 = complemented_segment[1];
    let p_2 = segment[0];
    let q_2 = segment[1];

    ((orient_triple(p_1, q_1, p_2) != orient_triple(p_1, q_1, q_2)) // General case
        && (orient_triple(p_2, q_2, p_1) == orient_triple(p_2, q_2, q_1)))
        || ((orient_triple(p_1, q_1, p_2) == Orientation::Collinear) // Special case (all points collinear)
        && (orient_triple(p_1, q_1, q_2) == Orientation::Collinear)
        && (orient_triple(p_2, q_2, p_1) == Orientation::Collinear)
        && (orient_triple(p_2, q_2, q_1) == Orientation::Collinear)
        && !(intervals_contains([p_1.x, q_1.x], [p_2.x, q_2.x])
        || intervals_contains([p_1.y, q_1.y], [p_2.y, q_2.y])))
}

/// Return true if the line intersects the segment
pub fn intersect_line_segment(line: [Vec2; 2], segment: [Vec2; 2]) -> bool {
    let p_1 = line[0];
    let q_1 = line[1];
    let p_2 = segment[0];
    let q_2 = segment[1];

    ((orient_triple(p_1, q_1, p_2) != orient_triple(p_1, q_1, q_2)) // General case
        && (orient_triple(p_2, q_2, p_1) != orient_triple(p_2, q_2, q_1)))
        || ((orient_triple(p_1, q_1, p_2) == Orientation::Collinear) // Special case
        && (orient_triple(p_1, q_1, q_2) == Orientation::Collinear)
        && (orient_triple(p_2, q_2, q_1) == Orientation::Collinear)
        && (orient_triple(p_2, q_2, p_1) == Orientation::Collinear))
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Orientation {
    Counterclockwise,
    Clockwise,
    Collinear,
}

// http://www.dcs.gla.ac.uk/~pat/52233/slides/Geometry1x1.pdf p.10
pub fn orient_triple(a: Vec2, b: Vec2, c: Vec2) -> Orientation {
    // abs needed to preserve correct signs
    match ((b.y - a.y) * (c.x - b.x) - (c.y - b.y) * (b.x - a.x)).partial_cmp(&0.0) {
        None => {
            panic!("uh oh! something is NaN")
        }
        Some(ordering) => match ordering {
            Ordering::Less => Orientation::Counterclockwise,
            Ordering::Equal => Orientation::Collinear,
            Ordering::Greater => Orientation::Clockwise,
        },
    }
}

/// Do the closed intervals have nonempty intersection?
pub fn intervals_intersect(interval_one: [f64; 2], interval_two: [f64; 2]) -> bool {
    let a_1 = interval_one[0].min(interval_one[1]);
    let a_2 = interval_one[0].max(interval_one[1]);
    let b_1 = interval_two[0].min(interval_two[1]);
    let b_2 = interval_two[0].max(interval_two[1]);

    // Case on if b_1 is in the interval or b_2 is in the interval
    (a_1 <= b_1 && b_1 <= a_2) || (a_1 <= b_2 && b_2 <= a_2)
}

/// Does interval one contain interval two?
pub fn intervals_contains(interval_one: [f64; 2], interval_two: [f64; 2]) -> bool {
    let a_1 = interval_one[0].min(interval_one[1]);
    let a_2 = interval_one[0].max(interval_one[1]);
    let b_1 = interval_two[0].min(interval_two[1]);
    let b_2 = interval_two[0].max(interval_two[1]);

    // Case on if b_1 is in the interval or b_2 is in the interval
    (a_1 <= b_1) && (b_2 <= a_2)
}

pub fn dist_to_line(pt: Vec2, rise_run: Vec2, offset: Vec2) -> f64 {
    let p = pt - offset;

    let projection_factor = p.ip(rise_run) / rise_run.normsq();

    (p - projection_factor * rise_run).norm()
}
