use eframe::egui::Ui;
use mlua::Lua;

use crate::app::data_structures::gen_config::GenConfig;
use crate::app::data_structures::zvec::ZVec;
use crate::app::lua_field::LuaField;

pub fn ui_generation(
    ui: &mut Ui,
    generate_current_layer: &mut bool,
    generate_all_layers: &mut bool,
    link_radii: bool,
    layer_mode: bool,
    lua_mode: bool,
    stack_gen_config: &mut ZVec<GenConfig>,
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
    if lua_mode {
        ui.columns(2, |columns| {
            if columns[0]
                .button("Set parameters for current layer by code")
                .clicked()
            {
                set_parameters_for_layer(
                    &mut stack_gen_config.get_mut(current_layer).unwrap(),
                    lua,
                    lua_field_radius_a,
                    lua_field_radius_b,
                    lua_field_tilt,
                    lua_field_center_offset_x,
                    lua_field_center_offset_y,
                    lua_field_squircle_parameter,
                    link_radii,
                    current_layer,
                );
            }
            columns[0].centered_and_justified(|ui| {
                *generate_current_layer = ui.button("Generate current layer").clicked();
            });

            if columns[1]
                .button("Set parameters for all layers by code")
                .clicked()
            {
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
                        link_radii,
                        layer,
                    )
                }
            }
            columns[1].centered_and_justified(|ui| {
                *generate_all_layers = ui.button("Generate all layers").clicked();
            });
        });
    } else {
        if layer_mode {
            ui.columns(2, |columns| {
                columns[0].centered_and_justified(|ui| {
                    *generate_current_layer = ui.button("Generate current layer").clicked();
                });

                columns[1].centered_and_justified(|ui| {
                    *generate_all_layers = ui.button("Generate all layers").clicked();
                });
            })
        } else {
            ui.centered_and_justified(|ui| {
                *generate_current_layer = ui.button("Generate").clicked();
            });
        }
    }
}

fn set_parameters_for_layer(
    gen_config: &mut GenConfig,
    lua: &mut Lua,
    lua_field_radius_a: &mut LuaField,
    lua_field_radius_b: &mut LuaField,
    lua_field_tilt: &mut LuaField,
    lua_field_center_offset_x: &mut LuaField,
    lua_field_center_offset_y: &mut LuaField,
    lua_field_squircle_parameter: &mut LuaField,
    link_radii: bool,
    layer: isize,
) {
    lua.globals().set("layer", layer).unwrap();
    lua.globals().set("l", layer).unwrap();

    lua_field_radius_a.eval(lua, &mut gen_config.radius_a);

    if link_radii {
        lua_field_radius_a.eval(lua, &mut gen_config.radius_b);
    } else {
        lua_field_radius_b.eval(lua, &mut gen_config.radius_b);
    }

    lua_field_tilt.eval(lua, &mut gen_config.tilt);

    lua_field_squircle_parameter.eval(lua, &mut gen_config.squircle_parameter);

    lua_field_center_offset_x.eval(lua, &mut gen_config.center_offset_x);

    lua_field_center_offset_y.eval(lua, &mut gen_config.center_offset_y);
}
