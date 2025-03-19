use crate::app::data_structures::blocks::Blocks;
use crate::app::generation::shape::{generate, AllAlgs, AllParams};
use crate::app::generation::shape_type::ShapeType;
use crate::app::sampling::SampleCombineMethod;

/// Sampled parameters belonging to a single layer
#[derive(Clone, Debug)]
pub struct LayerParameters {
    // for bookkeeping
    pub nr_samples: usize,

    pub algorithm: AllAlgs,

    pub parameters: Vec<AllParams>,
}

impl LayerParameters {
    pub fn new_null() -> Self {
        LayerParameters {
            nr_samples: 1,
            algorithm: AllAlgs::Null,

            // Parameter defaults are the same as for the default configuration of layer_config
            //  (circle with radius 5 centered at the origin)
            parameters: vec![AllParams::Null],
        }
    }

    pub fn new_from(shape_type: ShapeType) -> Self {
        match shape_type {
            ShapeType::Squircle => {
                LayerParameters {
                    nr_samples: 1,
                    algorithm: AllAlgs::Squircle(Default::default()),

                    // Parameter defaults are the same as for the default configuration of layer_config
                    //  (circle with radius 5 centered at the origin)
                    parameters: vec![AllParams::Squircle(Default::default())],
                }
            }
            ShapeType::Line => {
                LayerParameters {
                    nr_samples: 1,
                    algorithm: AllAlgs::Line(Default::default()),

                    // Parameter defaults are the same as for the default configuration of layer_config
                    //  (circle with radius 5 centered at the origin)
                    parameters: vec![AllParams::Line(Default::default())],
                }
            }
        }
    }

    /// Run the generation algorithm for the configuration `self`, the output is a `Blocks` object. document.
    pub fn generate(&self, sample_combine_method: &SampleCombineMethod) -> Blocks {
        // Generate from circle with selected algorithm
        Blocks::combine(
            sample_combine_method,
            self.parameters
                .iter()
                .map(|squircle_parameters| {
                    generate(
                        &self.algorithm,
                        squircle_parameters,
                        AllParams::grid_size(&self.parameters),
                    )
                })
                .collect(),
        )
    }
}
