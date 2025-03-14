use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::squircle_params::SquircleParams;
use crate::app::generation::shape::Shape;
use crate::app::generation::squircle::{Squircle, SquircleAlgorithm};
use crate::app::sampling::SampleCombineMethod;

// todo: generalize to taking <Alg, Params> instead of hardcoded squircle
/// Sampled parameters belonging to a single layer
#[derive(Debug, Clone)]
pub struct LayerParameters {
    // for bookkeeping
    pub nr_samples: usize,

    pub algorithm: SquircleAlgorithm,

    pub parameters: Vec<SquircleParams>,
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
    /// Run the generation algorithm for the configuration `self`, the output is a `Blocks` object. document.
    pub fn generate(&self, sample_combine_method: &SampleCombineMethod) -> Blocks {
        // Determine grid size
        // The major radius should be included, for some metrics we need at least one layer of padding
        //  around the generated figure. Assuming a square figure (squircle parameter infinity), we
        //  need an x side length of 2.0 * sqrt(2) * radius_major. Add 4 for a padding of at least 2
        //  on each side.

        // Compute the largest radii for all shapes on this layer
        let largest_radius_a = self
            .parameters
            .iter()
            .fold(f64::NEG_INFINITY, |a, b| a.max(b.radius_a));
        let largest_radius_b = self
            .parameters
            .iter()
            .fold(f64::NEG_INFINITY, |a, b| a.max(b.radius_b));

        // Compute the largest offset for all shapes on this layer
        let largest_offset_x = self
            .parameters
            .iter()
            .fold(f64::NEG_INFINITY, |a, b| a.max(b.center_offset_x));
        let largest_offset_y = self
            .parameters
            .iter()
            .fold(f64::NEG_INFINITY, |a, b| a.max(b.center_offset_y));

        // Note that this method works but is kind of stupid. Ideally we'd want to have a grid that's
        //  a lot smaller but still contains all the shapes (it can't be centered at the origin then anymore)
        let grid_size = ((2.0 * 1.42 * f64::max(largest_radius_a, largest_radius_b)).ceil()
            + 2.0 * largest_offset_x.abs().max(largest_offset_y.abs()).ceil())
            as usize
            + 4;

        // Generate from circle with selected algorithm
        Blocks::combine(
            sample_combine_method,
            self.parameters
                .iter()
                .map(|squircle_parameters| {
                    Squircle::generate(&self.algorithm, squircle_parameters, grid_size)
                })
                .collect(),
        )
    }
}
