#[derive(Debug, Clone)]
pub struct LineParams {
    // rise and run should be separate to allow for vertical lines
    pub rise: f64,
    pub run: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub thickness: f64,
    pub length: f64,
}

impl Default for LineParams {
    fn default() -> Self {
        Self {
            rise: 1.0,
            run: 2.0,
            offset_x: 0.0,
            offset_y: 0.0,
            thickness: 1.0,
            length: 10.0,
        }
    }
}

impl LineParams {}
