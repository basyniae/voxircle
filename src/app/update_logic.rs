use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::layer_config::LayerConfig;
use crate::app::data_structures::sampled_parameters::SampledParameters;
use crate::app::data_structures::zvec::ZVec;
use crate::app::lua_field::LuaField;
use crate::app::sampling::{SampleCombineMethod, SampleDistributeMethod};
use mlua::Lua;

pub fn sampling_points_update(
    only_sample_half_of_bottom_layer: bool,
    only_sample_half_of_top_layer: bool,
    nr_samples_per_layer: usize,
    sample_distribute_method: SampleDistributeMethod,
    stack_sampling_points: &mut ZVec<Vec<f64>>,

    sampling_points_compute_once: &mut bool,
    sampling_points_compute_auto: bool,
    sampling_points_is_outdated: &mut bool,

    layer_lowest: isize,
    layer_highest: isize,
) {
    // Activates if the sampling options have changed (this update) or if the stack grows
    //  (previous update). The points may also have to be recomputed if the stack shrinks,
    //  when half_of_bottom or half_of_top layer options are implemented. Fixme.
    if (*sampling_points_compute_once || sampling_points_compute_auto)
        && *sampling_points_is_outdated
    {
        *sampling_points_compute_once = false;
        *sampling_points_is_outdated = false;

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
    stack_layer_config: &mut ZVec<LayerConfig>,
    stack_sampled_parameters: &mut ZVec<SampledParameters>, // Store the configuration for each layer, handily indexed by integers
    stack_sampling_points: &ZVec<Vec<f64>>,

    parameters_current_layer_sample_once: &mut bool,
    parameters_current_layer_sample_auto: bool,
    parameters_current_layer_is_outdated: &mut bool,

    parameters_all_layers_sample_once: &mut bool,
    parameters_all_layers_sample_auto: bool,
    parameters_all_layers_is_outdated: &mut bool,

    blocks_current_layer_is_outdated: &mut bool,
    blocks_all_layer_is_outdated: &mut bool, // fixme: we really should use this at least somewhere

    current_layer: isize,
    layer_lowest: isize,
    layer_highest: isize,

    single_radius: bool,

    lua: &mut Lua, // Lua instance (only initialized once)
    lua_field_radius_a: &mut LuaField,
    lua_field_radius_b: &mut LuaField,
    lua_field_tilt: &mut LuaField,
    lua_field_center_offset_x: &mut LuaField,
    lua_field_center_offset_y: &mut LuaField,
    lua_field_squircle_parameter: &mut LuaField,
) {
    // Generate parameters to be sampled
    if (*parameters_current_layer_sample_once || parameters_current_layer_sample_auto)
        && (*parameters_current_layer_is_outdated
            || lua_field_radius_a.has_changed()
            || lua_field_radius_b.has_changed()
            || lua_field_tilt.has_changed()
            || lua_field_center_offset_x.has_changed()
            || lua_field_center_offset_y.has_changed()
            || lua_field_squircle_parameter.has_changed())
    {
        *parameters_current_layer_sample_once = false;
        *parameters_current_layer_is_outdated = false;

        *blocks_current_layer_is_outdated = true;

        // Update parameters for the sampling
        {
            crate::app::sampling::set_parameters(
                stack_sampled_parameters.get_mut(current_layer).unwrap(),
                &stack_sampling_points.get(current_layer).unwrap(),
                [
                    stack_layer_config.get(current_layer).unwrap().radius_a,
                    stack_layer_config.get(current_layer).unwrap().radius_b,
                    stack_layer_config.get(current_layer).unwrap().tilt,
                    stack_layer_config
                        .get(current_layer)
                        .unwrap()
                        .center_offset_x,
                    stack_layer_config
                        .get(current_layer)
                        .unwrap()
                        .center_offset_y,
                    stack_layer_config
                        .get(current_layer)
                        .unwrap()
                        .squircle_parameter,
                ],
                stack_layer_config.get(current_layer).unwrap().algorithm,
                lua,
                lua_field_radius_a,
                lua_field_radius_b,
                lua_field_tilt,
                lua_field_center_offset_x,
                lua_field_center_offset_y,
                lua_field_squircle_parameter,
                single_radius,
            );
        }

        // Update parameters for the sliders
        crate::app::sampling::update_control_parameters(
            stack_layer_config.get_mut(current_layer).unwrap(),
            current_layer,
            lua,
            lua_field_radius_a,
            lua_field_radius_b,
            lua_field_tilt,
            lua_field_center_offset_x,
            lua_field_center_offset_y,
            lua_field_squircle_parameter,
            single_radius,
        );

        lua_field_radius_a.register_success();
        lua_field_radius_b.register_success();
        lua_field_tilt.register_success();
        lua_field_center_offset_x.register_success();
        lua_field_center_offset_y.register_success();
        lua_field_squircle_parameter.register_success();
    }

    // Generate parameters to be sampled
    if (*parameters_all_layers_sample_once || parameters_all_layers_sample_auto)
        && (*parameters_all_layers_is_outdated
            || lua_field_radius_a.has_changed()
            || lua_field_radius_b.has_changed()
            || lua_field_tilt.has_changed()
            || lua_field_center_offset_x.has_changed()
            || lua_field_center_offset_y.has_changed()
            || lua_field_squircle_parameter.has_changed())
    {
        *parameters_all_layers_sample_once = false;
        *parameters_all_layers_is_outdated = false;

        *blocks_current_layer_is_outdated = true;

        // Update parameters for the sampling
        for layer in layer_lowest..=layer_highest {
            crate::app::sampling::set_parameters(
                stack_sampled_parameters.get_mut(layer).unwrap(),
                &stack_sampling_points.get(layer).unwrap(),
                [
                    stack_layer_config.get(layer).unwrap().radius_a,
                    stack_layer_config.get(layer).unwrap().radius_b,
                    stack_layer_config.get(layer).unwrap().tilt,
                    stack_layer_config.get(layer).unwrap().center_offset_x,
                    stack_layer_config.get(layer).unwrap().center_offset_y,
                    stack_layer_config.get(layer).unwrap().squircle_parameter,
                ],
                stack_layer_config.get(layer).unwrap().algorithm,
                lua,
                lua_field_radius_a,
                lua_field_radius_b,
                lua_field_tilt,
                lua_field_center_offset_x,
                lua_field_center_offset_y,
                lua_field_squircle_parameter,
                single_radius,
            );

            // Update parameters for the sliders
            crate::app::sampling::update_control_parameters(
                stack_layer_config.get_mut(layer).unwrap(),
                layer,
                lua,
                lua_field_radius_a,
                lua_field_radius_b,
                lua_field_tilt,
                lua_field_center_offset_x,
                lua_field_center_offset_y,
                lua_field_squircle_parameter,
                single_radius,
            )
        }

        lua_field_radius_a.register_success();
        lua_field_radius_b.register_success();
        lua_field_tilt.register_success();
        lua_field_center_offset_x.register_success();
        lua_field_center_offset_y.register_success();
        lua_field_squircle_parameter.register_success();
    }
}

pub fn blocks_update(
    stack_sampled_parameters: &ZVec<SampledParameters>, // Store the configuration for each layer, handily indexed by integers
    stack_blocks: &mut ZVec<Blocks>,

    blocks_current_layer_generate_once: &mut bool,
    blocks_current_layer_generate_auto: bool,
    blocks_current_layer_is_outdated: &mut bool,

    blocks_all_layers_generate_once: &mut bool,
    blocks_all_layers_generate_auto: bool,
    blocks_all_layers_is_outdated: &mut bool,

    recompute_metrics: &mut bool,

    current_layer: isize,
    layer_lowest: isize,

    sample_combine_method: SampleCombineMethod,
) {
    if (*blocks_current_layer_generate_once || blocks_current_layer_generate_auto)
        && *blocks_current_layer_is_outdated
    {
        *blocks_current_layer_generate_once = false;
        *blocks_current_layer_is_outdated = false;
        *recompute_metrics = true;

        stack_blocks.set(
            current_layer,
            stack_sampled_parameters
                .get(current_layer)
                .unwrap()
                .generate(sample_combine_method),
        );
    }

    // fixme: not sure where to set can_generate_all_layers
    if (*blocks_all_layers_generate_once || blocks_all_layers_generate_auto)
        && (*blocks_all_layers_is_outdated || *blocks_current_layer_is_outdated)
    {
        *blocks_all_layers_generate_once = false;
        *blocks_all_layers_is_outdated = false;
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
