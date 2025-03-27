use crate::app::control::Control;
use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::line::{Line, LineFields};
use crate::app::generation::squircle::{Squircle, SquircleFields};
use crate::app::generation::{AllAlgs, AllParams};
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
    squircle_fields: &mut SquircleFields,
    line_fields: &mut LineFields,
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
            squircle_fields,
            line_fields,
            param_config,
        );

        // Update parameters for the sliders
        update_slider_parameters(
            stack_layer_shape.get_mut(current_layer).unwrap(),
            current_layer,
            squircle_fields,
            line_fields,
            param_config,
        );

        squircle_fields.all_register_success();
        line_fields.all_register_success();
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
                squircle_fields,
                line_fields,
                param_config,
            );

            // Update parameters for the sliders
            update_slider_parameters(
                stack_layer_shape.get_mut(layer).unwrap(),
                layer,
                squircle_fields,
                line_fields,
                param_config,
            )
        }

        squircle_fields.all_register_success();
        line_fields.all_register_success();
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
    squircle_fields: &mut SquircleFields,
    line_fields: &mut LineFields,
    param_config: &ParamConfig,
) {
    match alg {
        AllAlgs::Squircle(alg) => {
            //todo: does this make sense
            let default_shape = match default_shape {
                AllParams::Squircle(x) => x.clone(),
                _ => Default::default(),
            };

            Squircle::set_parameters(
                layer_parameters,
                sampling_points,
                &default_shape,
                alg,
                squircle_fields,
                param_config,
            )
        }
        AllAlgs::Line(alg) => {
            //todo: does this make sense
            let default_shape = match default_shape {
                AllParams::Line(x) => x.clone(),
                _ => Default::default(),
            };

            Line::set_parameters(
                layer_parameters,
                sampling_points,
                &default_shape,
                alg,
                line_fields,
            )
        }
    }
}

fn update_slider_parameters(
    current_layer_shape: &mut AllParams,
    layer: isize,
    squircle_fields: &mut SquircleFields,
    line_fields: &mut LineFields,
    param_config: &ParamConfig,
) {
    match current_layer_shape {
        AllParams::Squircle(current_layer_shape) => {
            *current_layer_shape = Squircle::eval_param(
                &(layer as f64),
                squircle_fields,
                current_layer_shape,
                param_config,
            )
        }
        AllParams::Line(current_layer_shape) => {
            *current_layer_shape =
                Line::eval_param(&(layer as f64), line_fields, current_layer_shape)
        }
    }
}
