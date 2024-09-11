use eframe::egui::Ui;
use mlua::Lua;

use crate::app::data_structures::blocks::Blocks;
use crate::app::data_structures::gen_config::GenConfig;
use crate::app::data_structures::zvec::ZVec;
use crate::app::lua_field::LuaField;

// TODO: Only auto generate if the values have changed
pub fn ui_generation(
    ui: &mut Ui,
    auto_generate_current_layer: &mut bool,
    auto_generate_all_layers: &mut bool,
    circle_mode: bool,
    layer_mode: bool,
    lua_mode: bool,
    stack_gen_config: &mut ZVec<GenConfig>,
    stack_blocks: &mut ZVec<Blocks>,
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
    recompute_metrics: &mut bool,
) {
    if layer_mode {
        ui.checkbox(auto_generate_current_layer, "Auto-generate current layer");
        ui.checkbox(auto_generate_all_layers, "Auto-generate all layers");
    } else {
        ui.checkbox(auto_generate_current_layer, "Auto-generate");
    }

    if lua_mode {
        ui.columns(2, |columns| {
            if columns[0]
                .button("Set parameters for current layer by code")
                .clicked()
            {
                set_parameters_for_layer(
                    stack_gen_config,
                    lua,
                    lua_field_radius_a,
                    lua_field_radius_b,
                    lua_field_tilt,
                    lua_field_center_offset_x,
                    lua_field_center_offset_y,
                    lua_field_squircle_parameter,
                    circle_mode,
                    current_layer,
                );

                lua_field_radius_a.register_success();
                lua_field_radius_b.register_success();
                lua_field_tilt.register_success();
                lua_field_squircle_parameter.register_success();
                lua_field_center_offset_x.register_success();
                lua_field_center_offset_y.register_success();
            }
            columns[0].centered_and_justified(|ui| {
                if ui.button("Generate current layer").clicked() || *auto_generate_current_layer {
                    stack_blocks.set(
                        current_layer,
                        stack_gen_config.get_mut(current_layer).unwrap().generate(),
                    );

                    *recompute_metrics = true;
                }
            });

            if columns[1]
                .button("Set parameters for all layers by code")
                .clicked()
            {
                for layer in layer_lowest..=layer_highest {
                    set_parameters_for_layer(
                        stack_gen_config,
                        lua,
                        lua_field_radius_a,
                        lua_field_radius_b,
                        lua_field_tilt,
                        lua_field_center_offset_x,
                        lua_field_center_offset_y,
                        lua_field_squircle_parameter,
                        circle_mode,
                        layer,
                    )
                }

                lua_field_radius_a.register_success();
                lua_field_radius_b.register_success();
                lua_field_tilt.register_success();
                lua_field_squircle_parameter.register_success();
                lua_field_center_offset_x.register_success();
                lua_field_center_offset_y.register_success();
            }
            columns[1].centered_and_justified(|ui| {
                if ui.button("Generate all layers").clicked() || *auto_generate_all_layers {
                    generate_all_layers(
                        stack_gen_config,
                        stack_blocks,
                        layer_lowest,
                        recompute_metrics,
                    )
                }
            });
        });
    } else {
        if layer_mode {
            ui.columns(2, |columns| {
                columns[0].centered_and_justified(|ui| {
                    if ui.button("Generate current layer").clicked() || *auto_generate_current_layer
                    {
                        stack_blocks.set(
                            current_layer,
                            stack_gen_config.get_mut(current_layer).unwrap().generate(),
                        );

                        *recompute_metrics = true;
                    }
                });

                columns[1].centered_and_justified(|ui| {
                    if ui.button("Generate all layers").clicked() || *auto_generate_all_layers {
                        generate_all_layers(
                            stack_gen_config,
                            stack_blocks,
                            layer_lowest,
                            recompute_metrics,
                        )
                    }
                });
            })
        } else {
            ui.centered_and_justified(|ui| {
                if ui.button("Generate").clicked() || *auto_generate_current_layer {
                    stack_blocks.set(
                        current_layer,
                        stack_gen_config.get_mut(current_layer).unwrap().generate(),
                    );

                    *recompute_metrics = true;
                }
            });
        }
    }
}

fn generate_all_layers(
    stack_gen_config: &mut ZVec<GenConfig>,
    stack_blocks: &mut ZVec<Blocks>,
    layer_lowest: isize,
    recompute_metrics: &mut bool,
) {
    // Generate all layers
    *stack_blocks = ZVec::new(
        stack_gen_config
            .data
            .iter()
            .map(|config| config.generate())
            .collect(),
        layer_lowest,
    );

    *recompute_metrics = true;
}

fn set_parameters_for_layer(
    stack_gen_config: &mut ZVec<GenConfig>,
    lua: &mut Lua,
    lua_field_radius_a: &mut LuaField,
    lua_field_radius_b: &mut LuaField,
    lua_field_tilt: &mut LuaField,
    lua_field_center_offset_x: &mut LuaField,
    lua_field_center_offset_y: &mut LuaField,
    lua_field_squircle_parameter: &mut LuaField,
    circle_mode: bool,
    layer: isize,
) {
    lua.globals().set("layer", layer).unwrap();
    lua.globals().set("l", layer).unwrap();

    lua_field_radius_a.eval(lua, &mut stack_gen_config.get_mut(layer).unwrap().radius_a);

    if circle_mode {
        lua_field_radius_a.eval(lua, &mut stack_gen_config.get_mut(layer).unwrap().radius_b);
    } else {
        lua_field_radius_b.eval(lua, &mut stack_gen_config.get_mut(layer).unwrap().radius_b);
    }

    lua_field_tilt.eval(lua, &mut stack_gen_config.get_mut(layer).unwrap().tilt);

    lua_field_squircle_parameter.eval(
        lua,
        &mut stack_gen_config.get_mut(layer).unwrap().squircle_parameter,
    );

    lua_field_center_offset_x.eval(
        lua,
        &mut stack_gen_config.get_mut(layer).unwrap().center_offset_x,
    );

    lua_field_center_offset_y.eval(
        lua,
        &mut stack_gen_config.get_mut(layer).unwrap().center_offset_y,
    );
}
