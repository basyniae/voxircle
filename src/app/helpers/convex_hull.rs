use std::collections::HashSet;

use eframe::glow::TIME_ELAPSED;

// TODO: Maybe abstract convex hull to data type... we have a lot of functions specific to it

// Input: point cloud (unordered collection of points)
// Output: sequence of extreme points of the convex hull that goes around counterclockwise
// note that the starting point of the output is not well-defined, cyclic permutations are also
//  valid outputs.
pub fn get_convex_hull(points: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    if points.len() == 0 {
        return vec![];
    } else if points.len() == 1 {
        return points;
    }

    let mut conv_hull = vec![points[0], points[1]];

    // Loop over every point
    for point in points {
        // Check that i is contained in the current convex hull,
        // gather lines to which point is on the right
        let mut i_to_right_of_pts = vec![];

        for l in 0..conv_hull.len() {
            // By checking that i is to the right of every ordered line
            let line_start = conv_hull[l];
            let line_end = conv_hull[(l + 1) % conv_hull.len()];

            // If adding i to the convex hull would make it larger
            if
                is_right_of_oriented_line(point, line_start, line_end) || // to the right of a segment
                is_on_line(point, line_start, line_end) & // colinear to a segment but not inside of it
                    !is_on_line_segment(point, line_start, line_end)
            {
                // So it should be included in the new convex hull.
                // Where to insert i so that we retain ordered properties?
                // Remember which point i overrules
                let index;
                if l < conv_hull.len() {
                    index = l + conv_hull.len();
                } else {
                    index = l;
                }

                i_to_right_of_pts.push(index);
                i_to_right_of_pts.push(index + 1);
            }
        }

        if !i_to_right_of_pts.is_empty() {
            println!("interval: {:?}, modulus: {}", i_to_right_of_pts, conv_hull.len());

            let (min_index, max_index) = min_max_modular_interval(
                i_to_right_of_pts,
                conv_hull.len()
            );

            println!("min: {}, max: {}", min_index, max_index);

            let conv_hull_len = conv_hull.len();

            // Rotate so that the to-be-deleted section is at the front
            conv_hull.rotate_left((min_index + 1) % conv_hull_len);

            // Note that max_index - min_index - 1 elements have to be deleted
            conv_hull = conv_hull[(max_index - min_index - 1) % conv_hull_len..].to_vec();

            // Then push the new extreme point to the end of the new vector
            conv_hull.push(point);
        }
    }

    conv_hull
}

// Check if the input is to the right of the (infinitely long) with given start and endpoints.
// Outputs false if the input is on the line.
fn is_right_of_oriented_line(input: [f64; 2], line_start: [f64; 2], line_end: [f64; 2]) -> bool {
    let separation_vector = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
    let rotated_sep_vector = [separation_vector[1], -separation_vector[0]];

    // println!(
    //     "phi(input): {}, phi(line_start): {}",
    //     innerproduct(input, rotated_sep_vector),
    //     innerproduct(line_start, rotated_sep_vector)
    // );

    innerproduct(input, rotated_sep_vector) > innerproduct(line_start, rotated_sep_vector)
}

fn is_on_line(input: [f64; 2], line_start: [f64; 2], line_end: [f64; 2]) -> bool {
    let separation_vector = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
    let rotated_sep_vector = [separation_vector[1], -separation_vector[0]];

    innerproduct(input, rotated_sep_vector) == innerproduct(line_start, rotated_sep_vector)
}

// see notes for explanation
fn is_on_line_segment(input: [f64; 2], line_start: [f64; 2], line_end: [f64; 2]) -> bool {
    if is_on_line(input, line_start, line_end) {
        let end_vector = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
        let input_vector = [input[0] - line_start[0], input[1] - line_start[1]];

        let projection = innerproduct(end_vector, input_vector);
        let end_vector_length = innerproduct(end_vector, end_vector);

        0.0 <= projection && projection <= end_vector_length
    } else {
        false
    }
}

// Given an input and convex hull (ordered list of points)
fn is_in_conv_hull(input: [f64; 2], conv_hull: Vec<[f64; 2]>) -> bool {
    let mut in_hull_so_far = true;

    // Check that i is contained in the current convex hull
    for l in 0..conv_hull.len() {
        // By checking that i is to the right of every ordered line
        if is_right_of_oriented_line(input, conv_hull[l], conv_hull[(l + 1) % conv_hull.len()]) {
            in_hull_so_far = false;
        }
    }
    in_hull_so_far
}

// Convert the sequence of points forming a convex hull to easier to draw pairs of line segments
pub fn line_segments_from_conv_hull(conv_hull: Vec<[f64; 2]>) -> Vec<[[f64; 2]; 2]> {
    let mut line_segments = vec![];
    for i in 0..conv_hull.len() - 1 {
        line_segments.push([conv_hull[i], conv_hull[i + 1]]);
    }
    line_segments.push([conv_hull[conv_hull.len() - 1], conv_hull[0]]);

    line_segments
}

pub fn innerproduct(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

// Input: set of integers that are consequitive modulo the modulus.
// Output: start and end of consequitive sequence with 0 <= min <= max <= 2*modulus
// (strict inequality for more than two points)
fn min_max_modular_interval(set: Vec<usize>, modulus: usize) -> (usize, usize) {
    // Case out degenerate lengths
    if set.len() == 0 {
        panic!("input set should be nonempty!");
    } else if set.len() == 1 {
        return (set[0], set[0]);
    }

    let modded_set: HashSet<usize> = set
        .iter()
        .map(|u| u % modulus)
        .collect();

    // minimum: n s.t. n mod modulus is in the set but n-1 mod modulus is not
    // or None if DNE
    let mut minimum_opt = None;
    for n in &set {
        if !modded_set.contains(&((n - 1) % modulus)) {
            minimum_opt = Some(*n);
            break; // There is at most one such point by assumption on the form of the input
        }
    }

    match minimum_opt {
        Some(minimum) => {
            let minimum = minimum % modulus;
            let maximum = minimum + modded_set.len();

            return (minimum, maximum);
        }
        None => {
            return (0, modulus - 1);
        }
    }

    // // maximum: n s.t. n mod modulus is in the set but n+1 mod modulus is not
    // let mut maximum = 0;
    // for n in &set {
    //     if modded_set.contains(&((n + 1) % modulus)) {
    //         maximum = *n;
    //     }
    // }

    // // now order them appropiately
    // minimum %= modulus;
    // if maximum % modulus <= minimum {
    //     maximum = (maximum % modulus) + modulus;
    // } else {
    //     maximum %= modulus;
    // }

    // (minimum, maximum)
}
