use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::slice_parameters::SliceParameters;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::Algorithm;
use crate::app::rhai_field::RhaiField;
use crate::app::sampling::sampled_parameters::LayerParameters;
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
    stack_layer_config: &mut ZVec<SliceParameters>,
    stack_sampled_parameters: &mut ZVec<LayerParameters>, // Store the configuration for each layer, handily indexed by integers
    stack_sampling_points: &ZVec<Vec<f64>>,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
    blocks_current_layer_control: &mut Control,
    blocks_all_layers_control: &mut Control, // fixme: we really should use this at least somewhere

    current_layer: isize,
    layer_lowest: isize,
    layer_highest: isize,
    single_radius: bool,
    rhai_field_radius_a: &mut RhaiField,
    rhai_field_radius_b: &mut RhaiField,
    rhai_field_tilt: &mut RhaiField,
    rhai_field_center_offset_x: &mut RhaiField,
    rhai_field_center_offset_y: &mut RhaiField,
    rhai_field_squircle_parameter: &mut RhaiField,
) {
    // Generate parameters to be sampled
    if parameters_current_layer_control.update() {
        blocks_current_layer_control.set_outdated();

        // Update parameters for the sampling
        set_parameters(
            stack_sampled_parameters.get_mut(current_layer).unwrap(),
            stack_sampling_points.get(current_layer).unwrap(),
            stack_layer_config.get(current_layer).unwrap(),
            stack_layer_config.get(current_layer).unwrap().algorithm,
            rhai_field_radius_a,
            rhai_field_radius_b,
            rhai_field_tilt,
            rhai_field_center_offset_x,
            rhai_field_center_offset_y,
            rhai_field_squircle_parameter,
            single_radius,
        );

        // Update parameters for the sliders
        update_control_parameters(
            stack_layer_config.get_mut(current_layer).unwrap(),
            current_layer,
            rhai_field_radius_a,
            rhai_field_radius_b,
            rhai_field_tilt,
            rhai_field_center_offset_x,
            rhai_field_center_offset_y,
            rhai_field_squircle_parameter,
            single_radius,
        );

        rhai_field_radius_a.register_success();
        rhai_field_radius_b.register_success();
        rhai_field_tilt.register_success();
        rhai_field_center_offset_x.register_success();
        rhai_field_center_offset_y.register_success();
        rhai_field_squircle_parameter.register_success();
    }

    // Generate parameters to be sampled
    if parameters_all_layers_control.update() {
        blocks_current_layer_control.set_outdated();

        // Update parameters for the sampling
        for layer in layer_lowest..=layer_highest {
            set_parameters(
                stack_sampled_parameters.get_mut(layer).unwrap(),
                stack_sampling_points.get(layer).unwrap(),
                stack_layer_config.get(layer).unwrap(),
                stack_layer_config.get(layer).unwrap().algorithm,
                rhai_field_radius_a,
                rhai_field_radius_b,
                rhai_field_tilt,
                rhai_field_center_offset_x,
                rhai_field_center_offset_y,
                rhai_field_squircle_parameter,
                single_radius,
            );

            // Update parameters for the sliders
            update_control_parameters(
                stack_layer_config.get_mut(layer).unwrap(),
                layer,
                rhai_field_radius_a,
                rhai_field_radius_b,
                rhai_field_tilt,
                rhai_field_center_offset_x,
                rhai_field_center_offset_y,
                rhai_field_squircle_parameter,
                single_radius,
            )
        }

        rhai_field_radius_a.register_success();
        rhai_field_radius_b.register_success();
        rhai_field_tilt.register_success();
        rhai_field_center_offset_x.register_success();
        rhai_field_center_offset_y.register_success();
        rhai_field_squircle_parameter.register_success();
    }
}

pub fn blocks_update(
    stack_sampled_parameters: &ZVec<LayerParameters>, // Store the configuration for each layer, handily indexed by integers
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
            stack_sampled_parameters
                .get(current_layer)
                .unwrap()
                .generate(sample_combine_method),
        );
    }

    if blocks_all_layers_control.update() {
        *recompute_metrics = true;

        *stack_blocks = ZVec::new(
            stack_sampled_parameters
                .data
                .iter()
                .map(|config| config.generate(sample_combine_method))
                .collect(),
            layer_lowest,
        );
    }
}

fn update_control_parameters(
    current_layer: &mut SliceParameters,
    layer: isize,
    rhai_field_radius_a: &mut RhaiField,
    rhai_field_radius_b: &mut RhaiField,
    rhai_field_tilt: &mut RhaiField,
    rhai_field_center_offset_x: &mut RhaiField,
    rhai_field_center_offset_y: &mut RhaiField,
    rhai_field_squircle_parameter: &mut RhaiField,
    single_radius: bool,
) {
    // evaluate the rhai field at the layer
    if let Some(radius_a) = rhai_field_radius_a.eval(&(layer as f64)) {
        current_layer.radius_a = radius_a
    }

    if single_radius {
        if let Some(radius_a) = rhai_field_radius_a.eval(&(layer as f64)) {
            current_layer.radius_b = radius_a
        }
    } else {
        if let Some(radius_b) = rhai_field_radius_b.eval(&(layer as f64)) {
            current_layer.radius_b = radius_b
        }
    }

    if let Some(tilt) = rhai_field_tilt.eval(&(layer as f64)) {
        current_layer.tilt = tilt
    }
    if let Some(center_offset_x) = rhai_field_center_offset_x.eval(&(layer as f64)) {
        current_layer.center_offset_x = center_offset_x
    }
    if let Some(center_offset_y) = rhai_field_center_offset_y.eval(&(layer as f64)) {
        current_layer.center_offset_y = center_offset_y
    }

    if let Some(squircle_parameter) = rhai_field_squircle_parameter.eval(&(layer as f64)) {
        current_layer.squircle_parameter = squircle_parameter
    }
}

/// Update (old) input LayerParameters object with new values evarhaited from the code
fn set_parameters(
    sampled_parameters: &mut LayerParameters,
    sampling_points: &Vec<f64>,
    default_parameters: &SliceParameters,
    algorithm: Algorithm,
    rhai_field_radius_a: &mut RhaiField,
    rhai_field_radius_b: &mut RhaiField,
    rhai_field_tilt: &mut RhaiField,
    rhai_field_center_offset_x: &mut RhaiField,
    rhai_field_center_offset_y: &mut RhaiField,
    rhai_field_squircle_parameter: &mut RhaiField,
    single_radius: bool,
) {
    // Set the algorithm & nr. of samples
    sampled_parameters.algorithm = algorithm;
    sampled_parameters.nr_samples = sampling_points.len();

    // If the code evaluation failed (returned None) resort to using the default_parameters (supplied by sliders)
    sampled_parameters.parameters = sampling_points
        .iter()
        .map(|layer| SliceParameters {
            algorithm,
            radius_a: rhai_field_radius_a
                .eval(layer)
                .unwrap_or(default_parameters.radius_a),
            radius_b: if single_radius {
                rhai_field_radius_a
                    .eval(layer)
                    .unwrap_or(default_parameters.radius_a)
            } else {
                rhai_field_radius_b
                    .eval(layer)
                    .unwrap_or(default_parameters.radius_b)
            },
            tilt: rhai_field_tilt
                .eval(layer)
                .unwrap_or(default_parameters.tilt),
            center_offset_x: rhai_field_center_offset_x
                .eval(layer)
                .unwrap_or(default_parameters.center_offset_x),
            center_offset_y: rhai_field_center_offset_y
                .eval(layer)
                .unwrap_or(default_parameters.center_offset_y),
            squircle_parameter: rhai_field_squircle_parameter
                .eval(layer)
                .unwrap_or(default_parameters.squircle_parameter),
        })
        .collect()
}
