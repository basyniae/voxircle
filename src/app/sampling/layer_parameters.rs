use crate::app::data_structures::blocks::Blocks;
use crate::app::generation::shape::{TraitAlgorithm, TraitFields, TraitParameters, TraitShape};
use crate::app::sampling::SampleCombineMethod;
use std::marker::PhantomData;

/// Sampled parameters belonging to a single layer
pub struct LayerParameters<
    Alg: TraitAlgorithm,
    Params: TraitParameters,
    Fields: TraitFields,
    Shape: TraitShape<Alg, Params, Fields>,
> {
    // for bookkeeping
    pub nr_samples: usize,

    pub algorithm: Alg,

    pub parameters: Vec<Params>,

    phantom_fields: PhantomData<Fields>,
    phantom_shape: PhantomData<Shape>,
}

impl<
        Alg: TraitAlgorithm,
        Params: TraitParameters,
        Fields: TraitFields,
        Sh: TraitShape<Alg, Params, Fields>,
    > Default for LayerParameters<Alg, Params, Fields, Sh>
{
    fn default() -> Self {
        LayerParameters::<Alg, Params, Fields, Sh> {
            nr_samples: 1,
            algorithm: Default::default(),

            // Parameter defaults are the same as for the default configuration of layer_config
            //  (circle with radius 5 centered at the origin)
            parameters: vec![Default::default()],
            phantom_fields: Default::default(),
            phantom_shape: Default::default(),
        }
    }
}

impl<
        Alg: TraitAlgorithm,
        Params: TraitParameters,
        Fields: TraitFields,
        Sh: TraitShape<Alg, Params, Fields>,
    > Clone for LayerParameters<Alg, Params, Fields, Sh>
{
    fn clone(&self) -> Self {
        Self {
            nr_samples: self.nr_samples.clone(),
            algorithm: self.algorithm.clone(),
            parameters: self.parameters.clone(),
            phantom_fields: Default::default(),
            phantom_shape: Default::default(),
        }
    }
}

impl<
        Alg: TraitAlgorithm,
        Params: TraitParameters,
        Fields: TraitFields,
        Sh: TraitShape<Alg, Params, Fields>,
    > LayerParameters<Alg, Params, Fields, Sh>
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
