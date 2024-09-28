use crate::app::data_structures::blocks::Blocks;
use crate::app::generation::{generate_all_blocks, Algorithm};
use crate::app::math::linear_algebra::{Mat2, Vec2};
use crate::app::sampling::SampleCombineMethod;
use itertools::izip;

/// All parameters necessary to run the generation algorithm
#[derive(Debug, Clone)]
pub struct LayerConfig {
    pub algorithm: Algorithm,

    pub radius_a: f64, // These two are specified (a is the x-axis if tilt = 0)
    pub radius_b: f64,
    pub tilt: f64,
    pub squircle_parameter: f64,
    pub center_offset_x: f64,
    pub center_offset_y: f64,

    pub sample_combine_method: SampleCombineMethod,
    pub nr_samples: usize,

    pub sampled_radius_a: Vec<f64>,
    pub sampled_radius_b: Vec<f64>,
    pub sampled_tilt: Vec<f64>,
    pub sampled_squircle_parameter: Vec<f64>,
    pub sampled_center_offset_x: Vec<f64>,
    pub sampled_center_offset_y: Vec<f64>,
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::CenterPoint, // default: Centerpoint

            // Have two sets of parameters, one for "control", i.e., setting the parameter by hand,
            // the other for setting the parameter by code (called sampled_-)
            radius_a: 5.0, // default: 5.0
            radius_b: 5.0, // default: 5.0

            tilt: 0.0, // default: 0.0

            // sqrt_quad_form: Mat2::from([1.0, 0.0, 0.0, 1.0]),
            center_offset_x: 0.0, // default: 0.0, 0.0 (even circle)
            center_offset_y: 0.0,

            squircle_parameter: 2.0, // default: 2.0 (circle / ellipse)

            sampled_radius_a: vec![],
            sampled_radius_b: vec![],
            sampled_tilt: vec![],
            sampled_squircle_parameter: vec![],
            sampled_center_offset_x: vec![],
            sampled_center_offset_y: vec![],
        }
    }
}

impl LayerConfig {
    /// Run the generation algorithm for the configuration `self`, the output is a `Blocks` object.
    pub fn generate(&self, sample_combine_method: SampleCombineMethod) -> Blocks {
        // Determine grid size
        // The major radius should be included, for some metrics we need at least one layer of padding
        //  around the generated figure. Assuming a square figure (squircle parameter infinity), we
        //  need an x side length of 2.0 * sqrt(2) * radius_major. Add 4 for a padding of at least 2
        //  on each side.
        let grid_size = (2.0 * 1.42 * f64::max(self.radius_a, self.radius_b)).ceil() as usize + 4;

        // Generate from circle with selected algorithm
        Blocks::combine(
            sample_combine_method,
            izip!(
                self.sampled_radius_a.iter(),
                self.sampled_radius_b.iter(),
                self.sampled_tilt.iter(),
                self.sampled_squircle_parameter.iter(),
                self.sampled_center_offset_x.iter(),
                self.sampled_center_offset_y.iter()
            )
            .map(
                |(
                    radius_a,
                    radius_b,
                    tilt,
                    squircle_parameter,
                    center_offset_x,
                    center_offset_y,
                )| {
                    let c = tilt.cos();
                    let s = tilt.sin();
                    let sqrt_quad_form = Mat2::from_rows(
                        1.0 / radius_a * Vec2::from([c, s]),
                        1.0 / radius_b * Vec2::from([-s, c]),
                    );

                    generate_all_blocks(
                        &self.algorithm,
                        Vec2::from([*center_offset_x, *center_offset_y]),
                        sqrt_quad_form,
                        *squircle_parameter,
                        *radius_a,
                        *radius_b,
                        grid_size,
                    )
                },
            )
            .collect(),
        );

        generate_all_blocks(
            &self.algorithm,
            Vec2::from([self.center_offset_x, self.center_offset_y]),
            self.get_sqrt_quad_form(),
            self.squircle_parameter,
            self.radius_a,
            self.radius_b,
            grid_size,
        )
    }

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

    /// Get the squircle ui parameter (used for the slider) from the configuration `self`
    pub fn get_squircle_ui_parameter(&self) -> f64 {
        1.0 - 1.0 / (1.0 + self.squircle_parameter)
    }
}
