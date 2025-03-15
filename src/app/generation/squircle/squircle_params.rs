use crate::app::math::linear_algebra::{Mat2, Vec2};

/// All parameters necessary to completely describe a squircle
#[derive(Debug, Clone)]
pub struct SquircleParams {
    pub radius_a: f64, // These two are specified (a is the x-axis if tilt = 0)
    pub radius_b: f64,

    pub tilt: f64,

    pub center_offset_x: f64,
    pub center_offset_y: f64,

    pub squircle_parameter: f64,
}

impl Default for SquircleParams {
    fn default() -> Self {
        Self {
            // Have two sets of parameters, one for "control", i.e., setting the parameter by hand,
            // the other for setting the parameter by code (called sampled_-)
            radius_a: 5.0, // default: 5.0
            radius_b: 5.0, // default: 5.0

            tilt: 0.0, // default: 0.0

            // sqrt_quad_form: Mat2::from([1.0, 0.0, 0.0, 1.0]),
            center_offset_x: 0.0, // default: 0.0, 0.0 (even circle)
            center_offset_y: 0.0,

            squircle_parameter: 2.0, // default: 2.0 (circle / ellipse)
        }
    }
}

impl SquircleParams {
    /// Compute the sqrt_quad_form for the configuration `self`
    pub fn get_sqrt_quad_form(&self) -> Mat2 {
        // Compute a square root of the PSD symmetric quadratic form X defining the ellipse:
        //  (x,y)^TX(x,y)=1.
        let c = self.tilt.cos();
        let s = self.tilt.sin();
        Mat2::from_rows(
            1.0 / self.radius_a * Vec2::from([c, s]),
            1.0 / self.radius_b * Vec2::from([-s, c]),
        )
    }
}
