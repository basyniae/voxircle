// from https://www.geeksforgeeks.org/convex-hull-monotone-chain-algorithm/
/// Input: point cloud (unordered collection of points)
/// Output: sequence of extreme points of the convex hull that goes around counterclockwise
/// note that the starting point of the output is not well-defined, cyclic permutations are also
///  valid outputs.
pub fn get_convex_hull(points: &Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    if points.len() <= 2 {
        let mut conv_hull = vec![];
        for i in points {
            conv_hull.push(*i);
        }
        return conv_hull; // trivial for 0, 1, or 2 points
    }

    let mut conv_hull = vec![];

    // Get index of leftmost point (this is necessarily in the convex hull)
    let mut leftmost_index = 0;
    for i in 0..points.len() {
        if points[i][0] < points[leftmost_index][0] {
            leftmost_index = i;
        }
    }

    let mut p = leftmost_index;
    let mut q: usize;
    loop {
        conv_hull.push(points[p]);

        // increment q
        q = (p + 1) % points.len();

        // Pick the largest i such that (p, i, q) is positively oriented
        for i in 0..points.len() {
            if
            //  check orientation (is the triple product in 2D)
            (points[i][1] - points[p][1]) * (points[q][0] - points[i][0])
                - (points[i][0] - points[p][0]) * (points[q][1] - points[i][1])
                < 0.0
            {
                // If the triple (p, i, q) is oriented counterclockwise (strictly, i.e., not colinear)
                q = i;
            }
        }
        p = q;
        if p == leftmost_index {
            break;
            // we've checked all points
        }
    }

    conv_hull
}

/// Convert the sequence of points forming a convex hull to easier to draw pairs of line segments
pub fn line_segments_from_conv_hull(conv_hull: Vec<[f64; 2]>) -> Vec<[[f64; 2]; 2]> {
    let mut line_segments = vec![];
    for i in 0..conv_hull.len() - 1 {
        line_segments.push([conv_hull[i], conv_hull[i + 1]]);
    }
    line_segments.push([conv_hull[conv_hull.len() - 1], conv_hull[0]]);

    line_segments
}
