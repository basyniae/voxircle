use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::shape::{Shape, ShapeFields};
use crate::app::sampling::layer_parameters::LayerParameters;
use crate::app::sampling::{SampleCombineMethod, SampleDistributeMethod};
use std::fmt::Debug;

pub fn sampling_points_update(
    only_sample_half_of_bottom_layer: bool,
    only_sample_half_of_top_layer: bool,
    nr_samples_per_layer: usize,
    sample_distribute_method: SampleDistributeMethod,
    stack_sampling_points: &mut ZVec<Vec<f64>>,
    sampling_points_control: &mut Control,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
    layer_lowest: isize,
    layer_highest: isize,
) {
    // Activates if the sampling options have changed (this update) or if the stack grows
    //  (previous update). The points (may) also have to be recomputed if the stack shrinks,
    //  when half_of_bottom or half_of_top layer options are implemented.
    if sampling_points_control.update() {
        // if the sampling points have changed the parameters become outdated
        parameters_current_layer_control.set_outdated();
        parameters_all_layers_control.set_outdated();

        *stack_sampling_points = crate::app::sampling::determine_sampling_points(
            sample_distribute_method,
            layer_lowest,
            layer_highest,
            nr_samples_per_layer,
            only_sample_half_of_bottom_layer,
            only_sample_half_of_top_layer,
        );
    }
}

pub fn parameters_update<
    Alg: PartialEq + Default + Clone + Copy,
    Params: Default + Clone,
    Fields: Default + ShapeFields,
    Sh: Shape<Alg, Params, Fields> + Clone + Default,
>(
    stack_layer_shape: &mut ZVec<Params>,
    stack_layer_parameters: &mut ZVec<LayerParameters<Alg, Params, Fields, Sh>>, // Store the configuration for each layer, handily indexed by integers
    stack_sampling_points: &ZVec<Vec<f64>>,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
    blocks_current_layer_control: &mut Control,
    blocks_all_layers_control: &mut Control, // fixme: we really should use this at least somewhere

    current_layer: isize,
    layer_lowest: isize,
    layer_highest: isize,
    param_fields: &mut Fields,
    shape: &Sh,
) {
    // Generate parameters to be sampled
    if parameters_current_layer_control.update() {
        blocks_current_layer_control.set_outdated();
        let layer_alg = stack_layer_parameters.get(current_layer).unwrap().algorithm;

        // Update parameters for the sampling
        shape.set_parameters(
            stack_layer_parameters.get_mut(current_layer).unwrap(),
            stack_sampling_points.get(current_layer).unwrap(),
            stack_layer_shape.get(current_layer).unwrap(),
            layer_alg,
            param_fields,
        );

        // Update parameters for the sliders
        shape.update_slider_parameters(
            stack_layer_shape.get_mut(current_layer).unwrap(),
            current_layer,
            param_fields,
        );

        param_fields.all_register_success()
    }

    // Generate parameters to be sampled
    if parameters_all_layers_control.update() {
        blocks_current_layer_control.set_outdated();

        // Update parameters for the sampling
        for layer in layer_lowest..=layer_highest {
            let layer_alg = stack_layer_parameters.get(layer).unwrap().algorithm;

            shape.set_parameters(
                stack_layer_parameters.get_mut(layer).unwrap(),
                stack_sampling_points.get(layer).unwrap(),
                stack_layer_shape.get(layer).unwrap(),
                layer_alg,
                param_fields,
            );

            // Update parameters for the sliders
            shape.update_slider_parameters(
                stack_layer_shape.get_mut(layer).unwrap(),
                layer,
                param_fields,
            )
        }

        param_fields.all_register_success();
    }
}

pub fn blocks_update<
    Alg: PartialEq + Default + Clone + Copy,
    Params: Default + Clone,
    Fields: Default + ShapeFields,
    Sh: Shape<Alg, Params, Fields> + Clone + Default,
>(
    stack_layer_parameters: &ZVec<LayerParameters<Alg, Params, Fields, Sh>>, // Store the configuration for each layer, handily indexed by integers
    stack_blocks: &mut ZVec<Blocks>,
    blocks_current_layer_control: &mut Control,
    blocks_all_layers_control: &mut Control,
    recompute_metrics: &mut bool,
    current_layer: isize,
    layer_lowest: isize,
    sample_combine_method: &SampleCombineMethod,
) {
    if blocks_current_layer_control.update() {
        *recompute_metrics = true;

        stack_blocks.set(
            current_layer,
            stack_layer_parameters
                .get(current_layer)
                .unwrap()
                .generate(sample_combine_method),
        );
    }

    if blocks_all_layers_control.update() {
        *recompute_metrics = true;

        *stack_blocks = ZVec::new(
            stack_layer_parameters
                .data
                .iter()
                .map(|config| config.generate(sample_combine_method))
                .collect(),
            layer_lowest,
        );
    }
}
