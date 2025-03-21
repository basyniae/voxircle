use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::line::Line;
use crate::app::generation::squircle::Squircle;
use crate::app::generation::{AllAlgs, AllFields, AllParams};
use crate::app::param_config::ParamConfig;
use crate::app::sampling::layer_parameters::LayerParameters;
use crate::app::sampling::SampleCombineMethod;

pub mod metrics;

pub fn parameters_update(
    stack_layer_shape: &mut ZVec<AllParams>,
    stack_layer_parameters: &mut ZVec<LayerParameters>, // Store the configuration for each layer, handily indexed by integers
    stack_sampling_points: &ZVec<Vec<f64>>,
    parameters_current_layer_control: &mut Control,
    parameters_all_layers_control: &mut Control,
    blocks_current_layer_control: &mut Control,
    blocks_all_layers_control: &mut Control, // fixme: we really should use this at least somewhere

    current_layer: isize,
    layer_lowest: isize,
    layer_highest: isize,
    fields: &mut AllFields,
    param_config: &ParamConfig,
) {
    // Generate parameters to be sampled
    if parameters_current_layer_control.update() {
        blocks_current_layer_control.set_outdated();
        let layer_alg = stack_layer_parameters.get(current_layer).unwrap().algorithm;

        // Update parameters for the sampling
        set_parameters(
            stack_layer_parameters.get_mut(current_layer).unwrap(),
            stack_sampling_points.get(current_layer).unwrap(),
            stack_layer_shape.get(current_layer).unwrap(),
            &layer_alg,
            fields,
            param_config,
        );

        // Update parameters for the sliders
        update_slider_parameters(
            stack_layer_shape.get_mut(current_layer).unwrap(),
            current_layer,
            fields,
            param_config,
        );

        fields.all_register_success()
    }

    // Generate parameters to be sampled
    if parameters_all_layers_control.update() {
        blocks_current_layer_control.set_outdated();

        // Update parameters for the sampling
        for layer in layer_lowest..=layer_highest {
            let layer_alg = stack_layer_parameters.get(layer).unwrap().algorithm;

            set_parameters(
                stack_layer_parameters.get_mut(layer).unwrap(),
                stack_sampling_points.get(layer).unwrap(),
                stack_layer_shape.get(layer).unwrap(),
                &layer_alg,
                fields,
                param_config,
            );

            // Update parameters for the sliders
            update_slider_parameters(
                stack_layer_shape.get_mut(layer).unwrap(),
                layer,
                fields,
                param_config,
            )
        }

        fields.all_register_success();
    }
}

pub fn blocks_update(
    stack_layer_parameters: &ZVec<LayerParameters>, // Store the configuration for each layer, handily indexed by integers
    stack_blocks: &mut ZVec<Blocks>,
    blocks_current_layer_control: &mut Control,
    blocks_all_layers_control: &mut Control,
    metrics_control: &mut Control,
    current_layer: isize,
    layer_lowest: isize,
    sample_combine_method: &SampleCombineMethod,
) {
    if blocks_current_layer_control.update() {
        metrics_control.set_outdated();

        stack_blocks.set(
            current_layer,
            stack_layer_parameters
                .get(current_layer)
                .unwrap()
                .generate(sample_combine_method),
        );
    }

    if blocks_all_layers_control.update() {
        metrics_control.set_outdated();

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

fn set_parameters(
    layer_parameters: &mut LayerParameters,
    sampling_points: &Vec<f64>,
    default_shape: &AllParams,
    alg: &AllAlgs,
    fields: &mut AllFields,
    param_config: &ParamConfig,
) {
    match alg {
        AllAlgs::Null => panic!("impossible"),
        AllAlgs::Squircle(alg) => {
            if let (AllParams::Squircle(default_shape), AllFields::Squircle(fields)) =
                (default_shape, fields)
            {
                Squircle::set_parameters(
                    layer_parameters,
                    sampling_points,
                    default_shape,
                    alg,
                    fields,
                    param_config,
                )
            } else {
                panic!("set_parameters called with arguments that do not match")
            }
        }
        AllAlgs::Line(alg) => {
            if let (AllParams::Line(default_shape), AllFields::Line(fields)) =
                (default_shape, fields)
            {
                Line::set_parameters(
                    layer_parameters,
                    sampling_points,
                    default_shape,
                    alg,
                    fields,
                )
            } else {
                panic!("set_parameters called with arguments that do not match")
            }
        }
    }
}

fn update_slider_parameters(
    current_layer_shape: &mut AllParams,
    layer: isize,
    fields: &mut AllFields,
    param_config: &ParamConfig,
) {
    match current_layer_shape {
        AllParams::Null => panic!("impossible"),
        AllParams::Squircle(current_layer_shape) => {
            if let AllFields::Squircle(fields) = fields {
                *current_layer_shape =
                    Squircle::eval_param(&(layer as f64), fields, current_layer_shape, param_config)
            } else {
                panic!("update_slider_parameters called with arguments that do not match")
            }
        }
        AllParams::Line(current_layer_shape) => {
            if let AllFields::Line(fields) = fields {
                *current_layer_shape =
                    Line::eval_param(&(layer as f64), fields, current_layer_shape)
            } else {
                panic!("update_slider_parameters called with arguments that do not match")
            }
        }
    }
}
