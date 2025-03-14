use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::squircle_params::SquircleParams;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::squircle::{Squircle, SquircleAlgorithm};
use crate::app::param_field::ParamField;
use crate::app::sampling::layer_parameters::LayerParameters;
use crate::app::sampling::{SampleCombineMethod, SampleDistributeMethod};

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

pub fn parameters_update(
    stack_layer_shape: &mut ZVec<SquircleParams>,
    stack_layer_parameters: &mut ZVec<LayerParameters>, // Store the configuration for each layer, handily indexed by integers
    stack_sampling_points: &ZVec<Vec<f64>>,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
    blocks_current_layer_control: &mut Control,
    blocks_all_layers_control: &mut Control, // fixme: we really should use this at least somewhere

    current_layer: isize,
    layer_lowest: isize,
    layer_highest: isize,
    param_fields: &mut Vec<ParamField>,
    squircle: &Squircle,
) {
    let single_radius = squircle.single_radius;
    // Generate parameters to be sampled
    if parameters_current_layer_control.update() {
        blocks_current_layer_control.set_outdated();
        // todo: make systematic
        let layer_alg = stack_layer_parameters.get(current_layer).unwrap().algorithm;

        // Update parameters for the sampling
        set_parameters(
            stack_layer_parameters.get_mut(current_layer).unwrap(),
            stack_sampling_points.get(current_layer).unwrap(),
            stack_layer_shape.get(current_layer).unwrap(),
            layer_alg,
            param_fields,
            single_radius,
        );

        // Update parameters for the sliders
        update_control_parameters(
            stack_layer_shape.get_mut(current_layer).unwrap(),
            current_layer,
            param_fields,
            single_radius,
        );

        for param_field in param_fields.into_iter() {
            param_field.register_success()
        }
    }

    // Generate parameters to be sampled
    if parameters_all_layers_control.update() {
        blocks_current_layer_control.set_outdated();

        // Update parameters for the sampling
        for layer in layer_lowest..=layer_highest {
            // todo: make systematic
            let layer_alg = stack_layer_parameters.get(layer).unwrap().algorithm;

            set_parameters(
                stack_layer_parameters.get_mut(layer).unwrap(),
                stack_sampling_points.get(layer).unwrap(),
                stack_layer_shape.get(layer).unwrap(),
                layer_alg,
                param_fields,
                single_radius,
            );

            // Update parameters for the sliders
            update_control_parameters(
                stack_layer_shape.get_mut(layer).unwrap(),
                layer,
                param_fields,
                single_radius,
            )
        }

        for param_field in param_fields.into_iter() {
            param_field.register_success()
        }
    }
}

pub fn blocks_update(
    stack_layer_parameters: &ZVec<LayerParameters>, // Store the configuration for each layer, handily indexed by integers
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

fn update_control_parameters(
    current_layer_shape: &mut SquircleParams,
    layer: isize,
    param_fields: &mut Vec<ParamField>,
    single_radius: bool,
) {
    // evaluate the rhai field at the layer
    if let Some(radius_a) = param_fields[0].eval(&(layer as f64)) {
        current_layer_shape.radius_a = radius_a
    }

    if single_radius {
        if let Some(radius_a) = param_fields[0].eval(&(layer as f64)) {
            current_layer_shape.radius_b = radius_a
        }
    } else {
        if let Some(radius_b) = param_fields[1].eval(&(layer as f64)) {
            current_layer_shape.radius_b = radius_b
        }
    }

    if let Some(tilt) = param_fields[2].eval(&(layer as f64)) {
        current_layer_shape.tilt = tilt
    }
    if let Some(center_offset_x) = param_fields[3].eval(&(layer as f64)) {
        current_layer_shape.center_offset_x = center_offset_x
    }
    if let Some(center_offset_y) = param_fields[4].eval(&(layer as f64)) {
        current_layer_shape.center_offset_y = center_offset_y
    }

    if let Some(squircle_parameter) = param_fields[5].eval(&(layer as f64)) {
        current_layer_shape.squircle_parameter = squircle_parameter
    }
}

/// Update (old) input LayerParameters object with new values evaluated from the code
fn set_parameters(
    layer_parameters: &mut LayerParameters,
    sampling_points: &Vec<f64>,
    default_shape: &SquircleParams,
    algorithm: SquircleAlgorithm,
    param_fields: &mut Vec<ParamField>,
    single_radius: bool,
) {
    // Set the algorithm & nr. of samples
    layer_parameters.algorithm = algorithm;
    layer_parameters.nr_samples = sampling_points.len();

    // If the code evaluation failed (returned None) resort to using the default_parameters (supplied by sliders)
    layer_parameters.parameters = sampling_points
        .iter()
        .map(|layer| SquircleParams {
            radius_a: param_fields[0]
                .eval(layer)
                .unwrap_or(default_shape.radius_a),
            radius_b: if single_radius {
                param_fields[0]
                    .eval(layer)
                    .unwrap_or(default_shape.radius_a)
            } else {
                param_fields[1]
                    .eval(layer)
                    .unwrap_or(default_shape.radius_b)
            },
            tilt: param_fields[2].eval(layer).unwrap_or(default_shape.tilt),
            center_offset_x: param_fields[3]
                .eval(layer)
                .unwrap_or(default_shape.center_offset_x),
            center_offset_y: param_fields[4]
                .eval(layer)
                .unwrap_or(default_shape.center_offset_y),
            squircle_parameter: param_fields[5]
                .eval(layer)
                .unwrap_or(default_shape.squircle_parameter),
        })
        .collect()
}
