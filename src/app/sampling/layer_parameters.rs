use crate::app::data_structures::blocks::Blocks;
use crate::app::generation::shape_type::ShapeType;
use crate::app::generation::{generate, AllAlgs, AllParams};
use crate::app::sampling::SampleCombineMethod;

/// Sampled parameters belonging to a single layer
#[derive(Clone, Debug)]
pub struct LayerParameters {
    pub algorithm: AllAlgs,
    pub parameters: Vec<AllParams>,
}

impl LayerParameters {
    /// new from defaults provided by the shape type
    pub fn new_from(shape_type: ShapeType) -> Self {
        match shape_type {
            ShapeType::Squircle => LayerParameters {
                algorithm: AllAlgs::Squircle(Default::default()),
                parameters: vec![AllParams::Squircle(Default::default())],
            },
            ShapeType::Line => LayerParameters {
                algorithm: AllAlgs::Line(Default::default()),
                parameters: vec![AllParams::Line(Default::default())],
            },
        }
    }

    /// Run the generation algorithm for the configuration `self`, the output is a `Blocks` object.
    ///  Note that self contains almost all data to do this, only the sample combine method needs to
    ///  be provided.
    pub fn generate(&self, sample_combine_method: &SampleCombineMethod) -> Blocks {
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

    pub fn nr_samples(&self) -> usize {
        self.parameters.len()
    }
}
