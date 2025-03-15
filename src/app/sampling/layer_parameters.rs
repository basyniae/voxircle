use crate::app::data_structures::blocks::Blocks;
use crate::app::generation::shape::{Shape, ShapeFields};
use crate::app::sampling::SampleCombineMethod;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;

/// Sampled parameters belonging to a single layer
pub struct LayerParameters<
    Alg: PartialEq + Default + Clone + Copy,
    Params: Default + Clone,
    Fields: Default + ShapeFields,
    Sh: Shape<Alg, Params, Fields> + Default,
> {
    // for bookkeeping
    pub nr_samples: usize,

    pub algorithm: Alg,

    pub parameters: Vec<Params>,

    phantom_fields: PhantomData<Fields>,
    phantom_shape: PhantomData<Sh>,
}

impl<
        Alg: PartialEq + Default + Clone + Copy,
        Params: Default + Clone,
        Fields: Default + ShapeFields,
        Sh: Shape<Alg, Params, Fields> + Clone + Default,
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
        Alg: PartialEq + Default + Clone + Copy,
        Params: Default + Clone,
        Fields: Default + ShapeFields,
        Sh: Shape<Alg, Params, Fields> + Clone + Default,
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
        Alg: PartialEq + Default + Clone + Copy,
        Params: Default + Clone,
        Fields: Default + ShapeFields,
        Sh: Shape<Alg, Params, Fields> + Default + Clone,
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
