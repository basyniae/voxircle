/// Return largest square (specified by min and max vector)
pub fn square_max(a: [[f64; 2]; 2], b: [[f64; 2]; 2]) -> [[f64; 2]; 2] {
    let low_x = a[0][0].min(b[0][0]);
    let high_x = a[1][0].max(b[1][0]);
    let low_y = a[0][1].min(b[0][1]);
    let high_y = a[1][1].max(b[1][1]);

    [[low_x, low_y], [high_x, high_y]]
}
