use crate::app::data_structures::zvec::ZVec;
use std::fmt::{Display, Formatter};
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum SampleCombineMethod {
    #[default]
    AllSamples,
    AnySamples,
    Percentage(f64),
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum SampleDistributeMethod {
    #[default]
    IncludeEndpoints,
    ExcludeEndpoints,
}

/// The ZVec corresponds to the layers. Each float in the Vec for a particular layer corresponds to
///  a sample that that layer has
/// Note! The layer number is the middle!
pub fn determine_sampling_points(
    sample_distribute_method: SampleDistributeMethod,
    layer_lowest: isize,
    layer_highest: isize,
    nr_samples_per_layer: usize,
    only_sample_half_of_bottom_layer: bool,
    only_sample_half_of_top_layer: bool,
) -> ZVec<Vec<f64>> {
    // Need to handle this case separately (IncludeEndpoints is unable to cover both endpoints in
    //  one point, ExcludeEndpoints would give the right answer)
    if nr_samples_per_layer == 1 {
        ZVec::new(
            (layer_lowest..=layer_highest)
                .map(|layer| vec![layer as f64])
                .collect(),
            layer_lowest,
        )
    } else {
        match sample_distribute_method {
            // Correctness is routine to check, but do have to get pen and paper
            SampleDistributeMethod::IncludeEndpoints => {
                let sample_size = 1.0 / (nr_samples_per_layer - 1) as f64;

                ZVec::new(
                    (layer_lowest..=layer_highest)
                        .map(|layer| {
                            let start = if only_sample_half_of_bottom_layer && layer == layer_lowest
                            {
                                nr_samples_per_layer / 2 // Floor of nr_samples_per_layer over 2 (so never sample below layer_lowest)
                            } else {
                                0
                            };

                            let end = if only_sample_half_of_top_layer && layer == layer_highest {
                                (nr_samples_per_layer + 1) / 2 // Floor of nr_samples_per_layer + 1 over 2 (so never sample above layer_highest
                            } else {
                                nr_samples_per_layer
                            };

                            (start..end)
                                .map(|sample| layer as f64 + sample_size * (sample as f64) - 0.5)
                                .collect()
                        })
                        .collect(),
                    layer_lowest,
                )
            }

            SampleDistributeMethod::ExcludeEndpoints => {
                let sample_size = 1.0 / (nr_samples_per_layer) as f64;

                ZVec::new(
                    (layer_lowest..=layer_highest)
                        .map(|layer| {
                            let start = if only_sample_half_of_bottom_layer && layer == layer_lowest
                            {
                                nr_samples_per_layer / 2 + 1
                            } else {
                                1
                            };

                            let end = if only_sample_half_of_top_layer && layer == layer_highest {
                                (nr_samples_per_layer + 1) / 2
                            } else {
                                nr_samples_per_layer
                            };

                            (start..=end)
                                .map(|sample| {
                                    layer as f64 + sample_size * (sample as f64)
                                        - 0.5
                                        - 0.5 * sample_size
                                })
                                .collect()
                        })
                        .collect(),
                    layer_lowest,
                )
            }
        }
    }
}

impl Display for SampleCombineMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SampleCombineMethod::AllSamples => {
                write!(f, "All samples")
            }
            SampleCombineMethod::AnySamples => {
                write!(f, "Any samples")
            }
            SampleCombineMethod::Percentage(percentage) => {
                write!(f, "≥{:.0}% percent of samples", percentage * 100.0)
            }
        }
    }
}

impl Display for SampleDistributeMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SampleDistributeMethod::IncludeEndpoints => {
                write!(f, "Include endpoints")
            }
            SampleDistributeMethod::ExcludeEndpoints => {
                write!(f, "Exclude endpoints")
            }
        }
    }
}
