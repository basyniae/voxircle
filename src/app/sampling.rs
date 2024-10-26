use crate::app::data_structures::layer_config::LayerConfig;
use crate::app::data_structures::sampled_parameters::SampledParameters;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::Algorithm;
use crate::app::lua_field::LuaField;
use mlua::Lua;

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
    only_sample_half_of_bottom_layer: bool, // implement
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

// todo: move function to better location
/// Update (old) input SampledParameters object with new values evaluated from the code
pub fn set_parameters(
    sampled_parameters: &mut SampledParameters,
    sampling_points: &Vec<f64>,
    default_parameters: [f64; 6],

    algorithm: Algorithm,
    lua: &mut Lua,
    lua_field_radius_a: &mut LuaField,
    lua_field_radius_b: &mut LuaField,
    lua_field_tilt: &mut LuaField,
    lua_field_center_offset_x: &mut LuaField,
    lua_field_center_offset_y: &mut LuaField,
    lua_field_squircle_parameter: &mut LuaField,
    single_radius: bool,
) {
    // evaluate the lua field at the sampling point values
    let sampled_values: Vec<[Option<f64>; 6]> = sampling_points
        .iter()
        .map(|layer| {
            lua.globals().set("layer", *layer).unwrap();
            lua.globals().set("l", *layer).unwrap();

            [
                lua_field_radius_a.eval(lua),
                {
                    if single_radius {
                        lua_field_radius_a.eval(lua)
                    } else {
                        lua_field_radius_b.eval(lua)
                    }
                },
                lua_field_tilt.eval(lua),
                lua_field_center_offset_x.eval(lua),
                lua_field_center_offset_y.eval(lua),
                lua_field_squircle_parameter.eval(lua),
            ]
        })
        .collect();

    // Set the algorithm & nr. of samples
    sampled_parameters.algorithm = algorithm;
    sampled_parameters.nr_samples = sampling_points.len();

    // If the code evaluation failed (returned None) resort to using the default_parameters (supplied by sliders)
    sampled_parameters.parameters = (0..sampling_points.len())
        .map(|i| {
            [
                sampled_values[i][0].unwrap_or(default_parameters[0]),
                sampled_values[i][1].unwrap_or(default_parameters[1]),
                sampled_values[i][2].unwrap_or(default_parameters[2]),
                sampled_values[i][3].unwrap_or(default_parameters[3]),
                sampled_values[i][4].unwrap_or(default_parameters[4]),
                sampled_values[i][5].unwrap_or(default_parameters[5]),
            ]
        })
        .collect()
}

pub fn update_control_parameters(
    current_layer: &mut LayerConfig,
    layer: isize,

    lua: &mut Lua,
    lua_field_radius_a: &mut LuaField,
    lua_field_radius_b: &mut LuaField,
    lua_field_tilt: &mut LuaField,
    lua_field_center_offset_x: &mut LuaField,
    lua_field_center_offset_y: &mut LuaField,
    lua_field_squircle_parameter: &mut LuaField,
    single_radius: bool,
) {
    lua.globals().set("layer", layer).unwrap();
    lua.globals().set("l", layer).unwrap();

    // evaluate the lua field at the layer
    if let Some(radius_a) = lua_field_radius_a.eval(lua) {
        current_layer.radius_a = radius_a
    }

    if single_radius {
        if let Some(radius_a) = lua_field_radius_a.eval(lua) {
            current_layer.radius_b = radius_a
        }
    } else {
        if let Some(radius_b) = lua_field_radius_b.eval(lua) {
            current_layer.radius_b = radius_b
        }
    }

    if let Some(tilt) = lua_field_tilt.eval(lua) {
        current_layer.tilt = tilt
    }
    if let Some(center_offset_x) = lua_field_center_offset_x.eval(lua) {
        current_layer.center_offset_x = center_offset_x
    }
    if let Some(center_offset_y) = lua_field_center_offset_y.eval(lua) {
        current_layer.center_offset_y = center_offset_y
    }

    if let Some(squircle_parameter) = lua_field_squircle_parameter.eval(lua) {
        current_layer.squircle_parameter = squircle_parameter
    }
}
