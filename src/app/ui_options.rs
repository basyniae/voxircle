use std::f64::consts::PI;

use eframe::egui;
use eframe::egui::{Align, Layout, Ui};
use mlua::Lua;

use crate::app::data_structures::layer_config::LayerConfig;
use crate::app::data_structures::zvec::ZVec;
use crate::app::generation::Algorithm;
use crate::app::lua_field::LuaField;

/// document
pub fn ui_options(
    ui: &mut Ui,
    current_layer_config: &mut LayerConfig,
    single_radius: &mut bool,
    lua_mode: bool,
    lua: &mut Lua,
    lua_field_radius_a: &mut LuaField,
    lua_field_radius_b: &mut LuaField,
    lua_field_tilt: &mut LuaField,
    lua_field_center_offset_x: &mut LuaField,
    lua_field_center_offset_y: &mut LuaField,
    lua_field_squircle_parameter: &mut LuaField,
    sampling_points: &ZVec<Vec<f64>>,
    can_update_current_layer_parameters: &mut bool,
    can_update_all_layer_parameters: &mut bool,
) {
    // Select algorithm
    egui::ComboBox::from_label("Algorithm")
        .selected_text(format!("{:?}", current_layer_config.algorithm))
        // TODO: change formatting to text
        // TODO: easily change algorithm for all layers
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut current_layer_config.algorithm,
                Algorithm::CenterPoint,
                "Centerpoint",
            );
            ui.selectable_value(
                &mut current_layer_config.algorithm,
                Algorithm::Conservative,
                "Conservative",
            );
            ui.selectable_value(
                &mut current_layer_config.algorithm,
                Algorithm::Contained,
                "Contained",
            );
            ui.selectable_value(
                &mut current_layer_config.algorithm,
                Algorithm::Percentage(0.5),
                "Percentage",
            );
        });

    // additional algorithm-specific options + description
    match current_layer_config.algorithm {
        Algorithm::CenterPoint => {
            ui.label("Include a particular block iff its centerpoint is in the ellipse");
        }
        Algorithm::Conservative => {
            ui.label(
                "Include a particular block in the voxelization iff it has nonempty intersection with the ellipse"
            );
        }
        Algorithm::Contained => {
            ui.label("Include a particular block iff it is fully contained in the ellipse");
        }
        Algorithm::Percentage(percentage) => {
            //TODO: implement ellipse, superellipse
            ui.label(
                format!(
                    "Include a particular block in the voxelization iff more than {:.0}% of it is contained in the circle. Ellipses and squircles not implemented.",
                    100.0 * percentage
                )
            );
            let mut perc_slider = percentage.clone();
            if ui
                .add(
                    egui::Slider::new(&mut perc_slider, 0.0..=1.0)
                        .text("")
                        .fixed_decimals(2)
                        .custom_formatter(|n, _| {
                            format!("{:.0}%", n * 100.0) //  formatting of percentage slider
                        }),
                )
                .changed()
            {
                current_layer_config.algorithm = Algorithm::Percentage(perc_slider);
            };
        }
        Algorithm::Empty => {
            ui.label("Include no blocks in the voxelization");
        }
    }

    // Radius
    ui.separator();

    ui.checkbox(single_radius, "Single radius");

    if *single_radius {
        if ui
            .add(
                egui::Slider::new(&mut current_layer_config.radius_a, 0.0..=30.0)
                    .text("Radius")
                    .clamp_to_range(false)
                    .custom_formatter(|param, _| format!("{:.02}", param))
                    .drag_value_speed(0.03),
            )
            .changed()
        {
            // the code is now invalid
            lua_field_radius_a.update_field_state(lua, sampling_points);
            lua_field_radius_b.update_field_state(lua, sampling_points);

            *can_update_current_layer_parameters = true;
        };

        // lua
        if lua_mode {
            lua_field_radius_a.show(ui, lua, sampling_points);
        }

        current_layer_config.radius_b = current_layer_config.radius_a;
    } else {
        // radius a
        if ui
            .add(
                egui::Slider::new(&mut current_layer_config.radius_a, 0.0..=30.0)
                    .text("Radius A")
                    .clamp_to_range(false)
                    .custom_formatter(|param, _| format!("{:.02}", param))
                    .drag_value_speed(0.03),
            )
            .changed()
        {
            lua_field_radius_a.update_field_state(lua, sampling_points);
            *can_update_current_layer_parameters = true;
        }
        if lua_mode {
            lua_field_radius_a.show(ui, lua, sampling_points);
        }

        // radius b
        if ui
            .add(
                egui::Slider::new(&mut current_layer_config.radius_b, 0.0..=30.0)
                    .text("Radius B")
                    .clamp_to_range(false)
                    .custom_formatter(|param, _| format!("{:.02}", param))
                    .drag_value_speed(0.03),
            )
            .changed()
        {
            lua_field_radius_b.update_field_state(lua, sampling_points);
            *can_update_current_layer_parameters = true;
        }
        if lua_mode {
            lua_field_radius_b.show(ui, lua, sampling_points);
        }

        //TODO: Make circular slider for more intuitive controls (need to build this myapp probably)
    }

    //tilt
    if ui
        .add(
            egui::Slider::new(&mut current_layer_config.tilt, -6.28..=6.28)
                .text("Tilt (radians)")
                .fixed_decimals(2)
                .drag_value_speed(0.01),
        )
        .changed()
    {
        lua_field_tilt.update_field_state(lua, sampling_points);
        *can_update_current_layer_parameters = true;
    };

    // Particular values
    ui.allocate_ui_with_layout(
        egui::Vec2::from([100.0, 200.0]),
        Layout::left_to_right(Align::Min),
        |ui| {
            if ui.button("0°").clicked() {
                current_layer_config.tilt = 0.0;
                lua_field_tilt.update_field_state(lua, sampling_points);
                *can_update_current_layer_parameters = true;
            }
            if ui.button("30°").clicked() {
                current_layer_config.tilt = PI / 6.0;
                lua_field_tilt.update_field_state(lua, sampling_points);
                *can_update_current_layer_parameters = true;
            }
            if ui.button("45°").clicked() {
                current_layer_config.tilt = PI / 4.0;
                lua_field_tilt.update_field_state(lua, sampling_points);
                *can_update_current_layer_parameters = true;
            }
            if ui.button("1:2").clicked() {
                current_layer_config.tilt = 0.5_f64.atan();
                lua_field_tilt.update_field_state(lua, sampling_points);
                *can_update_current_layer_parameters = true;
            }
            if ui.button("1:3").clicked() {
                current_layer_config.tilt = 0.333333333333_f64.atan();
                lua_field_tilt.update_field_state(lua, sampling_points);
                *can_update_current_layer_parameters = true;
            }
            if ui.button("2:3").clicked() {
                current_layer_config.tilt = 0.666666666666_f64.atan();
                lua_field_tilt.update_field_state(lua, sampling_points);
                *can_update_current_layer_parameters = true;
            }
            if ui.button("1:4").clicked() {
                current_layer_config.tilt = 0.25_f64.atan();
                lua_field_tilt.update_field_state(lua, sampling_points);
                *can_update_current_layer_parameters = true;
            }
        },
    );
    if lua_mode {
        lua_field_tilt.show(ui, lua, sampling_points);
    }

    // Squircle parameter
    // due to the scale of the parameter this is all a bit awkward... Introduce a temporary variable for controlling it
    {
        let mut squircle_ui_parameter = current_layer_config.get_squircle_ui_parameter();
        ui.separator();
        if ui
            .add(
                egui::Slider::new(&mut squircle_ui_parameter, 0.0..=1.0)
                    .text("Squircicity")
                    .custom_formatter(|param, _| format!("{:.02}", 1.0 / (1.0 - param) - 1.0))
                    .custom_parser(|s| s.parse::<f64>().map(|t| 1.0 - 1.0 / (t + 1.0)).ok()),
            )
            .changed()
        {
            lua_field_squircle_parameter.update_field_state(lua, sampling_points);
            *can_update_current_layer_parameters = true;
        };

        // Default values

        // Aim: Make choice of squircle parameter easy. there are distinct values at 2/3 and 1/3 we want to be exact
        ui.allocate_ui_with_layout(
            egui::Vec2::from([100.0, 200.0]),
            Layout::left_to_right(Align::Min),
            |ui| {
                if ui.button("Circle").clicked() {
                    squircle_ui_parameter = 0.666666666666666;
                    lua_field_squircle_parameter.update_field_state(lua, sampling_points);
                    *can_update_current_layer_parameters = true;
                }
                if ui.button("Diamond").clicked() {
                    squircle_ui_parameter = 0.5;
                    lua_field_squircle_parameter.update_field_state(lua, sampling_points);
                    *can_update_current_layer_parameters = true;
                }
                if ui.button("Square").clicked() {
                    squircle_ui_parameter = 1.0;
                    lua_field_squircle_parameter.update_field_state(lua, sampling_points);
                    *can_update_current_layer_parameters = true;
                }
            },
        );
        current_layer_config.squircle_parameter = 1.0 / (1.0 - squircle_ui_parameter) - 1.0;
    }
    // now kill the temporary variable

    if lua_mode {
        lua_field_squircle_parameter.show(ui, lua, sampling_points);
    }

    // Centerpoint
    ui.separator();
    if ui
        .add(
            egui::Slider::new(&mut current_layer_config.center_offset_x, -1.0..=1.0)
                .text("x offset"),
        )
        .changed()
    {
        lua_field_center_offset_x.update_field_state(lua, sampling_points);
        *can_update_current_layer_parameters = true;
    }
    if lua_mode {
        lua_field_center_offset_x.show(ui, lua, sampling_points);
    }
    if ui
        .add(
            egui::Slider::new(&mut current_layer_config.center_offset_y, -1.0..=1.0)
                .text("y offset"),
        )
        .changed()
    {
        lua_field_center_offset_y.update_field_state(lua, sampling_points);
        *can_update_current_layer_parameters = true;
    };
    if lua_mode {
        lua_field_center_offset_y.show(ui, lua, sampling_points);
    }

    // Add odd and even buttons (also good so people understand what the abstraction "offset center" actually means)
    ui.allocate_ui_with_layout(
        egui::Vec2::from([100.0, 200.0]),
        Layout::left_to_right(Align::Min),
        |ui| {
            if ui.button("Even center").clicked() {
                current_layer_config.center_offset_x = 0.0;
                current_layer_config.center_offset_y = 0.0;
                lua_field_center_offset_x.update_field_state(lua, sampling_points);
                lua_field_center_offset_y.update_field_state(lua, sampling_points);
                *can_update_current_layer_parameters = true;
            }
            if ui.button("Odd center").clicked() {
                current_layer_config.center_offset_x = 0.5;
                current_layer_config.center_offset_y = 0.5;
                lua_field_center_offset_x.update_field_state(lua, sampling_points);
                lua_field_center_offset_y.update_field_state(lua, sampling_points);
                *can_update_current_layer_parameters = true;
            }
        },
    );
}
