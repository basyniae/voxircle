use eframe::egui;
use eframe::egui::Ui;
use mlua::Lua;

use crate::app::data_structures::layer_config::LayerConfig;
use crate::app::data_structures::zvec::ZVec;
use crate::app::lua_field::LuaField;

pub fn ui_generation(
    ui: &mut Ui,
    generate_current_layer: &mut bool,
    generate_all_layers: &mut bool,
    single_radius: bool,
    layer_mode: bool,
    lua_mode: bool,
    stack_gen_config: &mut ZVec<LayerConfig>,
    lua: &mut Lua,
    lua_field_radius_a: &mut LuaField,
    lua_field_radius_b: &mut LuaField,
    lua_field_tilt: &mut LuaField,
    lua_field_center_offset_x: &mut LuaField,
    lua_field_center_offset_y: &mut LuaField,
    lua_field_squircle_parameter: &mut LuaField,
    layer_lowest: isize,
    layer_highest: isize,
    current_layer: isize,
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

            if response.clicked() {
                set_parameters_for_layer(
                    &mut stack_gen_config.get_mut(current_layer).unwrap(),
                    lua,
                    lua_field_radius_a,
                    lua_field_radius_b,
                    lua_field_tilt,
                    lua_field_center_offset_x,
                    lua_field_center_offset_y,
                    lua_field_squircle_parameter,
                    single_radius,
                    current_layer,
                );
            }
        }

        if lua_mode && layer_mode {
            let response = ui.add_sized(
                [150.0, 50.0],
                egui::Button::new("Set parameters for all layers by code").wrap(true),
            );

            if response.clicked() {
                for layer in layer_lowest..=layer_highest {
                    set_parameters_for_layer(
                        &mut stack_gen_config.get_mut(layer).unwrap(),
                        lua,
                        lua_field_radius_a,
                        lua_field_radius_b,
                        lua_field_tilt,
                        lua_field_center_offset_x,
                        lua_field_center_offset_y,
                        lua_field_squircle_parameter,
                        single_radius,
                        layer,
                    )
                }
            }
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

fn set_parameters_for_layer(
    gen_config: &mut LayerConfig,
    lua: &mut Lua,
    lua_field_radius_a: &mut LuaField,
    lua_field_radius_b: &mut LuaField,
    lua_field_tilt: &mut LuaField,
    lua_field_center_offset_x: &mut LuaField,
    lua_field_center_offset_y: &mut LuaField,
    lua_field_squircle_parameter: &mut LuaField,
    single_radius: bool,
    layer: isize,
) {
    lua.globals().set("layer", layer).unwrap();
    lua.globals().set("l", layer).unwrap();

    lua_field_radius_a.eval(lua, &mut gen_config.radius_a);

    if single_radius {
        lua_field_radius_a.eval(lua, &mut gen_config.radius_b);
    } else {
        lua_field_radius_b.eval(lua, &mut gen_config.radius_b);
    }

    lua_field_tilt.eval(lua, &mut gen_config.tilt);

    lua_field_squircle_parameter.eval(lua, &mut gen_config.squircle_parameter);

    lua_field_center_offset_x.eval(lua, &mut gen_config.center_offset_x);

    lua_field_center_offset_y.eval(lua, &mut gen_config.center_offset_y);
}
