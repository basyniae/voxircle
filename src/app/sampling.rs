use crate::app::data_structures::zvec::ZVec;

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
    only_sample_half_of_bottom_layer: bool, //todo: implement
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
            SampleDistributeMethod::IncludeEndpoints => {
                let sample_size = 1.0 / (nr_samples_per_layer - 1) as f64;

                ZVec::new(
                    (layer_lowest..=layer_highest)
                        .map(|layer| {
                            (0..nr_samples_per_layer)
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
                            (1..=nr_samples_per_layer)
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
