use eframe::egui;
use eframe::egui::Ui;
use mlua::Lua;

use crate::app::data_structures::sampled_parameters::SampledParameters;
use crate::app::generation::Algorithm;
use crate::app::lua_field::LuaField;

pub fn ui_generation(
    ui: &mut Ui,
    generate_current_layer: &mut bool,
    generate_all_layers: &mut bool,
    generate_current_layer_parameters: &mut bool,
    generate_all_layer_parameters: &mut bool,
    layer_mode: bool,
    lua_mode: bool,
) {
    egui::Grid::new("generation_grid").show(ui, |ui| {
        if lua_mode {
            let response = ui.add_sized(
                [150.0, 50.0],
                egui::Button::new({
                    if layer_mode {
                        "Set parameters for current layer by code"
                    } else {
                        "Set parameters by code"
                    }
                })
                .wrap(true),
            );

            *generate_current_layer_parameters |= response.clicked();
        }

        if lua_mode && layer_mode {
            let response = ui.add_sized(
                [150.0, 50.0],
                egui::Button::new("Set parameters for all layers by code").wrap(true),
            );

            *generate_all_layer_parameters |= response.clicked();
        }

        if lua_mode {
            ui.end_row();
        }

        let response = ui.add_sized(
            [150.0, 50.0],
            egui::Button::new({
                if layer_mode {
                    "Generate current layer"
                } else {
                    "Generate"
                }
            })
            .wrap(true),
        );
        *generate_current_layer = response.clicked();

        if layer_mode {
            let response = ui.add_sized(
                [150.0, 50.0],
                egui::Button::new("Generate all layers").wrap(true),
            );
            *generate_all_layers = response.clicked();
        };
    });
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

    // Set the algorithm
    sampled_parameters.algorithm = algorithm;

    // If the number of samples did not change, we can use the old sampled_parameters as defaults for the new
    if sampling_points.len() == sampled_parameters.nr_samples {
        for i in 0..sampling_points.len() {
            // loop over all parameters
            for b in 0..6 {
                if sampled_values[i][b].is_some() {
                    sampled_parameters.parameters[i][b] = sampled_values[i][b].unwrap()
                }
            }
        }
    } else {
        sampled_parameters.nr_samples = sampling_points.len();

        // resort to using the default_parameters
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
}
