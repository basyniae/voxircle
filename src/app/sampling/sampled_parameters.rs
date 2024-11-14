use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::slice_parameters::SliceParameters;
use crate::app::generation::{generate_all_blocks, Algorithm};
use crate::app::math::linear_algebra::{Mat2, Vec2};
use crate::app::sampling::SampleCombineMethod;

/// Sampled parameters belonging to a single layer
#[derive(Debug, Clone)]
pub struct LayerParameters {
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
    pub parameters: Vec<SliceParameters>,
}

impl Default for LayerParameters {
    fn default() -> Self {
        LayerParameters {
            nr_samples: 1,
            algorithm: Default::default(),

            // Parameter defaults are the same as for the default configuration of layer_config
            //  (circle with radius 5 centered at the origin)
            parameters: vec![Default::default()],
        }
    }
}

impl LayerParameters {
    // fixme: make grid size determination better: needs to also take care of the offset (if
    //  the input is very offset this method fails)
    // todo: use reference to sample combine method
    /// Run the generation algorithm for the configuration `self`, the output is a `Blocks` object. document.
    pub fn generate(&self, sample_combine_method: &SampleCombineMethod) -> Blocks {
        // Determine grid size
        // The major radius should be included, for some metrics we need at least one layer of padding
        //  around the generated figure. Assuming a square figure (squircle parameter infinity), we
        //  need an x side length of 2.0 * sqrt(2) * radius_major. Add 4 for a padding of at least 2
        //  on each side.
        let largest_radius_a = self
            .parameters
            .iter()
            .fold(f64::NEG_INFINITY, |a, b| a.max(b.radius_a));
        let largest_radius_b = self
            .parameters
            .iter()
            .fold(f64::NEG_INFINITY, |a, b| a.max(b.radius_b));

        let grid_size =
            (2.0 * 1.42 * f64::max(largest_radius_a, largest_radius_b)).ceil() as usize + 4;

        // Generate from circle with selected algorithm
        Blocks::combine(
            sample_combine_method,
            self.parameters
                .iter()
                .map(|slice_parameters| {
                    generate_all_blocks(
                        &self.algorithm,
                        Vec2::from([
                            slice_parameters.center_offset_x,
                            slice_parameters.center_offset_y,
                        ]),
                        slice_parameters.get_sqrt_quad_form(),
                        slice_parameters.squircle_parameter,
                        slice_parameters.radius_a,
                        slice_parameters.radius_b,
                        grid_size,
                    )
                })
                .collect(),
        )
    }

    // TODO: think about where sqrt_quad_form is stored, maybe better to precompute it if we're
    //  using it multiple times (for this function whose purpose is display, and for the actual
    //  generation of the shape)
    /// Compute the sqrt_quad_form for the sampled parameters `self`
    pub fn get_sampled_sqrt_quad_form(&self) -> Vec<Mat2> {
        // Compute a square root of the PSD symmetric quadratic form X defining the ellipse:
        //  (x,y)^TX(x,y)=1.
        self.parameters
            .iter()
            .map(|slice_parameters| slice_parameters.get_sqrt_quad_form())
            .collect()
    }
}
