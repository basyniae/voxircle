use crate::app::data_structures::blocks::Blocks;
use crate::app::generation::{generate_all_blocks, Algorithm};
use crate::app::math::linear_algebra::{Mat2, Vec2};
use crate::app::sampling::SampleCombineMethod;

/// Sampled parameters belonging to a single layer
#[derive(Debug, Clone)]
pub struct SampledParameters {
    // for bookkeeping
    pub nr_samples: usize,

    pub algorithm: Algorithm,

    // in the order
    // [0] radius_a
    // [1] radius_b
    // [2] tilt
    // [3] center_offset_x
    // [4] center_offset_y
    // [5] squircle parameter
    pub parameters: Vec<[f64; 6]>,
}

impl Default for SampledParameters {
    fn default() -> Self {
        SampledParameters {
            nr_samples: 1,
            algorithm: Default::default(),

            // Parameter defaults are the same as for the default configuration of layer_config
            //  (circle with radius 5 centered at the origin)
            parameters: vec![[5.0, 5.0, 0.0, 0.0, 0.0, 2.0]],
        }
    }
}

impl SampledParameters {
    // fixme: make grid size determination better: needs to also take care of the offset (if
    //  the input is very offset this method fails)
    /// Run the generation algorithm for the configuration `self`, the output is a `Blocks` object. document.
    pub fn generate(&self, sample_combine_method: SampleCombineMethod) -> Blocks {
        // Determine grid size
        // The major radius should be included, for some metrics we need at least one layer of padding
        //  around the generated figure. Assuming a square figure (squircle parameter infinity), we
        //  need an x side length of 2.0 * sqrt(2) * radius_major. Add 4 for a padding of at least 2
        //  on each side.
        let largest_radius_a = self
            .parameters
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b[0]));
        let largest_radius_b = self
            .parameters
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b[1]));

        let grid_size =
            (2.0 * 1.42 * f64::max(largest_radius_a, largest_radius_b)).ceil() as usize + 4;

        // Generate from circle with selected algorithm
        Blocks::combine(
            sample_combine_method,
            self.parameters.iter()
            .map(
                |[
                    radius_a,
                    radius_b,
                    tilt,
                    center_offset_x,
                    center_offset_y,
                    squircle_parameter,
                ]| {
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
        )
    }

    // TODO: think about where sqrt_quad_form is stored, maybe better to precompute it if we're
    //  using it multiple times (for this function whose purpose is display, and for the actual
    //  generation of the shape
    /// Compute the sqrt_quad_form for the sampled parameters `self`
    pub fn get_sampled_sqrt_quad_form(&self) -> Vec<Mat2> {
        // Compute a square root of the PSD symmetric quadratic form X defining the ellipse:
        //  (x,y)^TX(x,y)=1.
        self.parameters
            .iter()
            .map(
                |[radius_a, radius_b, tilt, _squircle_parameter, _center_offset_x, _center_offset_y]| {
                    let c = tilt.cos();
                    let s = tilt.sin();
                    Mat2::from_rows(
                        1.0 / radius_a * Vec2::from([c, s]),
                        1.0 / radius_b * Vec2::from([-s, c]),
                    )
                },
            )
            .collect()
    }
}
