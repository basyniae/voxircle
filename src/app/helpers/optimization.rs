/// For a set of lines (affine y = ax + b), minimize_{x\in RR} max{ax+b} (i.e., the minimize the maximum of the lines)
/// The objective x |-> max{ax+b} is convex, so the subgradient test works: x is optimal iff 0 is a subgradient
///  of the objective, i.e., the slope of the left or right function is zero, or changes sign.
/// Output is [x where optimum is achieved, optimal value y]. For zero slope, prefer the left point
///  (i.e., treat zero slope as "positive" in terms of sign change)
/// Input is vector of pairs [a,b], for the line y=ax+b
/// Make sure all lines have different slope!
pub fn minimize_maximum_straight_lines(lines: Vec<[f64; 2]>) -> [f64; 2] {
    let lines_nonnegative_slope: Vec<_> = lines.iter().filter(|line| line[0] >= 0.0).collect();
    let lines_negative_slope: Vec<_> = lines.iter().filter(|line| line[0] < 0.0).collect();

    let objective = |x: f64| -> f64 {
        lines
            .iter()
            .map(|&line| line[0] * x + line[1])
            .fold(f64::INFINITY, |a, b| a.min(b))
    };

    let mut result = [0.0, 0.0];
    for (n_line, nn_line) in itertools::iproduct!(&lines_negative_slope, &lines_nonnegative_slope) {
        // Compute intersection of these two lines
        let x = (nn_line[1] - n_line[1]) / (n_line[0] - nn_line[0]);
        // check if the intersection of these lines is on the graph of the objective
        // then the slope changes signs by how the lines are structured, so we've found an optimum
        if objective(x) == n_line[0] * x + n_line[1] {
            result = [x, objective(x)];
            break;
        }
    }

    result
}
