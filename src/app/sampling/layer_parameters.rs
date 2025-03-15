use crate::app::data_structures::blocks::Blocks;
use crate::app::generation::shape::Shape;
use crate::app::sampling::SampleCombineMethod;
use std::fmt::{Debug, Display};

/// Sampled parameters belonging to a single layer
#[derive(Debug)]
pub struct LayerParameters<
    Alg: Debug + PartialEq + Default + Clone + Copy,
    Params: Default + Clone,
    Sh: Shape<Alg, Params> + Default + Clone,
> {
    // for bookkeeping
    pub nr_samples: usize,

    pub algorithm: Alg,

    pub parameters: Vec<Params>,

    // seems to be to satisfy the trait checker
    shape: Sh,
}

impl<
        Alg: Debug + PartialEq + Default + Clone + Copy,
        Params: Default + Clone,
        Sh: Shape<Alg, Params> + Clone + Default,
    > Default for LayerParameters<Alg, Params, Sh>
{
    fn default() -> Self {
        LayerParameters::<Alg, Params, Sh> {
            nr_samples: 1,
            algorithm: Default::default(),

            // Parameter defaults are the same as for the default configuration of layer_config
            //  (circle with radius 5 centered at the origin)
            parameters: vec![Default::default()],
            shape: Default::default(),
        }
    }
}

impl<
        Alg: Debug + PartialEq + Default + Clone + Copy,
        Params: Default + Clone,
        Sh: Shape<Alg, Params> + Clone + Default,
    > Clone for LayerParameters<Alg, Params, Sh>
{
    fn clone(&self) -> Self {
        Self {
            nr_samples: self.nr_samples.clone(),
            algorithm: self.algorithm.clone(),
            parameters: self.parameters.clone(),
            shape: self.shape.clone(),
        }
    }
}

impl<
        Alg: Debug + PartialEq + Default + Clone + Copy,
        Params: Default + Clone,
        Sh: Shape<Alg, Params> + Default + Clone,
    > LayerParameters<Alg, Params, Sh>
{
    /// Run the generation algorithm for the configuration `self`, the output is a `Blocks` object. document.
    pub fn generate(&self, sample_combine_method: &SampleCombineMethod) -> Blocks {
        // Generate from circle with selected algorithm
        Blocks::combine(
            sample_combine_method,
            self.parameters
                .iter()
                .map(|squircle_parameters| {
                    Sh::generate(
                        &self.algorithm,
                        squircle_parameters,
                        Sh::grid_size(&self.parameters),
                    )
                })
                .collect(),
        )
    }
}
