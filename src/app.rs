pub mod gen_config;
pub mod gen_output;
mod generation;
pub mod helpers;
mod lua_field;
mod metrics;

use self::generation::Algorithm;
use self::helpers::convex_hull::{get_convex_hull, line_segments_from_conv_hull};
use crate::app::helpers::boundary_3d::boundary_3d;
use crate::app::helpers::exact_squircle_bounds::exact_squircle_bounds;
use crate::app::helpers::linear_algebra::Vec2;
use crate::app::helpers::square_max::square_max;
use crate::app::lua_field::LuaField;
use crate::formatting;
use eframe::egui::{self, Vec2b};
use eframe::egui::{Direction, Layout};
use eframe::emath::Align;
use eframe::epaint::{Color32, Stroke};
use egui_plot::{
    uniform_grid_spacer, HLine, Line, Plot, PlotBounds, PlotPoint, PlotPoints, Points, Text, VLine,
};
use gen_config::GenConfig;
use gen_output::GenOutput;
use helpers::blocks::Blocks;
use helpers::plotting;
use mlua::Lua;
use std::collections::VecDeque;
use std::default::Default;
use std::f64::consts::PI;

// Colors based on Blender Minimal Dark scheme, 3D Viewport
const COLOR_BACKGROUND: Color32 = Color32::from_rgb(28, 28, 28); // middle background color (dark gray)
const COLOR_WIRE: Color32 = Color32::from_rgb(33, 33, 33); // "Wire" color (gray)
const COLOR_FACE: Color32 = Color32::from_rgb(161, 163, 164); // Face color (light gray)
const COLOR_LIME: Color32 = Color32::from_rgb(0, 255, 47); // "Active object" color (lime)
const COLOR_LIGHT_BLUE: Color32 = Color32::from_rgb(0, 217, 255); // "Object selected" color (light blue)
const COLOR_ORANGE: Color32 = Color32::from_rgb(255, 133, 0); // "Grease Pencil Vertex Select" color (orange)
const COLOR_DARK_ORANGE: Color32 = Color32::from_rgb(204, 106, 0); // Darker shade of orange
const COLOR_PURPLE: Color32 = Color32::from_rgb(179, 104, 186); // Darker shade of orange
const COLOR_YELLOW: Color32 = Color32::from_rgb(255, 242, 0); // "Edge Angle Text" color (yellow)
const COLOR_X_AXIS: Color32 = Color32::from_rgb(123, 34, 34); // x-axis color (red)
const COLOR_Y_AXIS: Color32 = Color32::from_rgb(44, 107, 44); // y-axis color (green)

pub struct App {
    layer: isize,
    // Go from stack number (nonnegative) to layer number (integer) by (-1)^(n+1) floor((n+1)/2)
    // Go from layer number to stack number by 2|l| - 1/2 (sgn(l) + 1)
    layer_lowest: isize,
    layer_highest: isize,

    grid_size: usize,
    global_bounding_box: [[f64; 2]; 2],

    stack_gen_config: VecDeque<GenConfig>, // In the order layer_lowest, layer_lowest + 1, ...
    current_gen_config: GenConfig,

    stack_gen_output: VecDeque<GenOutput>,
    current_gen_output: GenOutput,

    nr_blocks_total: u64,
    nr_blocks_interior: u64,
    nr_blocks_boundary: u64,

    boundary_3d: Vec<Blocks>,

    auto_generate: bool,
    layer_config_changed: bool,
    layer_output_changed: bool,

    view_voxelization: bool,
    view_complement: bool,

    view_boundary_2d: bool,
    view_interior_2d: bool,
    view_boundary_3d: bool,
    view_interior_3d: bool,

    view_intersect_area: bool,

    view_convex_hull: bool,
    view_outer_corners: bool,
    convex_hull: Vec<[f64; 2]>,
    outer_corners: Vec<[f64; 2]>,
    reset_zoom_once: bool,
    reset_zoom: bool,

    circle_mode: bool,

    lua: Lua,
    lua_field_radius_a: LuaField,
    lua_field_radius_b: LuaField,
    lua_field_tilt: LuaField,
    lua_field_center_offset_x: LuaField,
    lua_field_center_offset_y: LuaField,
    lua_field_squircle_parameter: LuaField,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.style_mut(|style| {
            style.spacing.slider_width = 200.0;
            style.spacing.combo_width = 200.0;
        });

        // persist lua between layer switching and frames and so on
        let lua = Lua::new();
        // give lua as little information as possible about the configurations... handle that all in rust
        lua.globals().set("layer", 0).unwrap();

        // Defaults should be such that we get useful output on startup
        // esp. some positive integral radius, auto generate on, and view blocks on
        Self {
            layer: 0,
            layer_lowest: 0,
            layer_highest: 0,

            grid_size: 10,
            global_bounding_box: [[10.0; 2]; 2],

            stack_gen_config: VecDeque::from([Default::default()]),
            current_gen_config: Default::default(),

            stack_gen_output: VecDeque::from([Default::default()]),
            current_gen_output: Default::default(),

            nr_blocks_total: Default::default(),
            nr_blocks_interior: Default::default(),
            nr_blocks_boundary: Default::default(),

            boundary_3d: vec![Default::default()],

            auto_generate: true,
            layer_config_changed: true,
            layer_output_changed: true,

            view_voxelization: true,
            view_complement: false,

            view_boundary_2d: false,
            view_interior_2d: false,
            view_boundary_3d: false,
            view_interior_3d: false,

            view_intersect_area: false,

            view_convex_hull: false,
            view_outer_corners: false,
            convex_hull: Default::default(),
            outer_corners: Default::default(),

            reset_zoom_once: false,
            reset_zoom: true,

            circle_mode: true,

            lua,
            lua_field_radius_a: LuaField::new(true, true),
            lua_field_radius_b: LuaField::new(true, true),
            lua_field_tilt: LuaField::new(true, false),
            lua_field_center_offset_x: LuaField::new(true, false),
            lua_field_center_offset_y: LuaField::new(true, false),
            lua_field_squircle_parameter: LuaField::new(false, true),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Options panel
        egui::SidePanel::right("options-panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Generation");
                ui.separator();

                // Select algorithm
                if egui::ComboBox
                ::from_label("Algorithm")
                    .selected_text(format!("{:?}", self.current_gen_config.algorithm))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.current_gen_config.algorithm,
                            Algorithm::CenterPoint,
                            "Center point",
                        );
                        ui.selectable_value(
                            &mut self.current_gen_config.algorithm,
                            Algorithm::Conservative,
                            "Conservative",
                        );
                        ui.selectable_value(
                            &mut self.current_gen_config.algorithm,
                            Algorithm::Contained,
                            "Contained");
                        ui.selectable_value(
                            &mut self.current_gen_config.algorithm,
                            Algorithm::Percentage(0.5),
                            "Percentage",
                        );
                    }).response.changed() { self.layer_config_changed |= true };

                // additional algorithm-specific options + description
                match self.current_gen_config.algorithm {
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
                    Algorithm::Percentage(percentage) => { //TODO: implement ellipse, superellipse
                        ui.label(
                            format!(
                                "Include a particular block in the voxelization iff more than {:.0}% of it is contained in the circle. Ellipses and squircles not implemented.",
                                100.0 * percentage
                            )
                        );
                        let mut perc_slider = percentage.clone();
                        if ui.add(
                            egui::Slider
                            ::new(&mut perc_slider, 0.0..=1.0)
                                .text("")
                                .fixed_decimals(2)
                                .custom_formatter(|n, _| {
                                    format!("{:.0}%", n * 100.0) //  formatting of percentage slider
                                })
                        ).changed() {
                            self.layer_config_changed |= true
                        };
                        self.current_gen_config.algorithm = Algorithm::Percentage(perc_slider);
                    }
                }

                // Radius
                ui.separator();
                if ui.checkbox(&mut self.circle_mode, "Circle mode").changed() {
                    self.layer_config_changed |= true;
                };
                if self.circle_mode {
                    if ui.add(
                        egui::Slider
                        ::new(&mut self.current_gen_config.radius_a, 0.0..=30.0)
                            .text("Radius")
                            .clamp_to_range(false)
                            .custom_formatter(|param, _| {
                                format!("{:.02}", param)
                            })
                    ).changed() {
                        self.layer_config_changed |= true;
                        self.lua_field_radius_a.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                    };

                    self.current_gen_config.radius_b = self.current_gen_config.radius_a;
                    self.current_gen_config.tilt = 0.0;

                    // lua
                    self.lua_field_radius_a.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);
                } else {
                    // radius a
                    if ui.add(
                        egui::Slider
                        ::new(&mut self.current_gen_config.radius_a, 0.0..=30.0)
                            .text("Radius A")
                            .clamp_to_range(false)
                            .custom_formatter(|param, _| {
                                format!("{:.02}", param)
                            })
                    ).changed() {
                        self.layer_config_changed |= true;
                        self.lua_field_radius_a.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                    };
                    self.lua_field_radius_a.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);

                    // radius b
                    if ui.add(
                        egui::Slider
                        ::new(&mut self.current_gen_config.radius_b, 0.0..=30.0)
                            .text("Radius B")
                            .clamp_to_range(false)
                            .custom_formatter(|param, _| {
                                format!("{:.02}", param)
                            })
                    ).changed() {
                        self.layer_config_changed |= true;
                        self.lua_field_radius_b.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                    };
                    self.lua_field_radius_b.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);

                    if ui.add(
                        egui::Slider
                        ::new(&mut self.current_gen_config.tilt, -6.28..=6.28)
                            .text("Tilt (radians)")
                            .fixed_decimals(2)
                    ).changed() {
                        self.layer_config_changed |= true;
                        self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                    };


                    // Particular values
                    ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                    {
                        if ui.button("0°").clicked() {
                            self.current_gen_config.tilt = 0.0;
                            self.layer_config_changed |= true;
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                        }
                        if ui.button("30°").clicked() {
                            self.current_gen_config.tilt = PI / 6.0;
                            self.layer_config_changed |= true;
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                        }
                        if ui.button("45°").clicked() {
                            self.current_gen_config.tilt = PI / 4.0;
                            self.layer_config_changed |= true;
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                        }
                        if ui.button("1:2").clicked() {
                            self.current_gen_config.tilt = 0.5_f64.atan();
                            self.layer_config_changed |= true;
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                        }
                        if ui.button("1:3").clicked() {
                            self.current_gen_config.tilt = 0.333333333333_f64.atan();
                            self.layer_config_changed |= true;
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                        }
                        if ui.button("2:3").clicked() {
                            self.current_gen_config.tilt = 0.666666666666_f64.atan();
                            self.layer_config_changed |= true;
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                        }
                        if ui.button("1:4").clicked() {
                            self.current_gen_config.tilt = 0.25_f64.atan();
                            self.layer_config_changed |= true;
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                        }
                    });
                    self.lua_field_tilt.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);

                    //TODO: Make circular slider for more intuitive controls (need to build this myself probably)
                }

                // Squircle parameter
                // due to the scale of the parameter this is all a bit awkward... Introduce a temporary variable for controlling it
                {
                    let mut squircle_ui_parameter = self.current_gen_config.get_squircle_ui_parameter();
                    ui.separator();
                    if ui.add(egui::Slider::new(&mut squircle_ui_parameter, 0.0..=1.0)
                        .text("Squircicity")
                        .custom_formatter(|param, _| {
                            format!("{:.02}", 1.0 / (1.0 - param) - 1.0)
                        })
                        .custom_parser(|s| {
                            s.parse::<f64>().map(|t| {
                                1.0 - 1.0 / (t + 1.0)
                            }).ok()
                        })
                    ).changed() {
                        self.layer_config_changed |= true;
                        self.lua_field_squircle_parameter.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                    };

                    // Default values

                    // Aim: Make choice of squircle parameter easy. there are distinct values at 2/3 and 1/3 we want to be exact
                    ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                    {
                        if ui.button("Circle").clicked() {
                            squircle_ui_parameter = 0.666666666666666;
                            self.layer_config_changed |= true;
                            self.lua_field_squircle_parameter.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                        }
                        if ui.button("Diamond").clicked() {
                            squircle_ui_parameter = 0.5;
                            self.layer_config_changed |= true;
                            self.lua_field_squircle_parameter.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                        }
                        if ui.button("Square").clicked() {
                            squircle_ui_parameter = 1.0;
                            self.layer_config_changed |= true;
                            self.lua_field_squircle_parameter.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                        }
                    });
                    self.current_gen_config.squircle_parameter = 1.0 / (1.0 - squircle_ui_parameter) - 1.0;
                }
                // now kill the temporary variable

                self.lua_field_squircle_parameter.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);


                // Centerpoint
                ui.separator();
                if ui.add(egui::Slider::new(&mut self.current_gen_config.center_offset_x, -1.0..=1.0).text("x offset")).changed() {
                    self.layer_config_changed |= true;
                    self.lua_field_center_offset_x.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                };
                self.lua_field_center_offset_x.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);
                if ui.add(egui::Slider::new(&mut self.current_gen_config.center_offset_y, -1.0..=1.0).text("y offset")).changed() {
                    self.layer_config_changed |= true;
                    self.lua_field_center_offset_y.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                };
                self.lua_field_center_offset_y.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);
                // Add odd and even buttons (also good so people understand what the abstraction "offset center" actually means)
                ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                {
                    if ui.button("Even center").clicked() {
                        self.current_gen_config.center_offset_x = 0.0;
                        self.current_gen_config.center_offset_y = 0.0;
                        self.layer_config_changed |= true;
                        self.lua_field_center_offset_x.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        self.lua_field_center_offset_y.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                    }
                    if ui.button("Odd center").clicked() {
                        self.current_gen_config.center_offset_x = 0.5;
                        self.current_gen_config.center_offset_y = 0.5;
                        self.layer_config_changed |= true;
                        self.lua_field_center_offset_x.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        self.lua_field_center_offset_y.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest)
                    }
                });


                // Viewport options
                ui.separator();
                ui.separator();
                ui.heading("View options");

                ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                {
                    ui.checkbox(&mut self.view_voxelization, "Voxelization");
                    ui.checkbox(&mut self.view_complement, "Complement");
                });
                ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                {
                    ui.checkbox(&mut self.view_convex_hull, "Convex hull");
                    ui.checkbox(&mut self.view_outer_corners, "Outer corners");
                });
                ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                {
                    ui.add_enabled(self.circle_mode, egui::Checkbox::new(&mut self.view_intersect_area, "Intersect area"));
                });
                ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                {
                    ui.checkbox(&mut self.view_boundary_2d, "Boundary 2D");
                    ui.checkbox(&mut self.view_interior_2d, "Interior 2D");
                });
                ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                {
                    ui.checkbox(&mut self.view_boundary_3d, "Boundary 3D");
                    ui.checkbox(&mut self.view_interior_3d, "Interior 3D");
                });


                // Generate action
                ui.separator();

                ui.columns(2, |columns| {
                    columns[0].checkbox(&mut self.auto_generate, "Auto generate");
                    columns[0].centered_and_justified(|ui| {
                        if self.layer_config_changed {
                            // always zoom correctly if the layer config has changed
                            self.global_bounding_box = square_max(
                                self.stack_gen_config
                                    .iter()
                                    .map(|g_c| exact_squircle_bounds(g_c, 1.1))
                                    .fold(
                                        [
                                            [f64::INFINITY, f64::INFINITY],
                                            [f64::NEG_INFINITY, f64::NEG_INFINITY],
                                        ],
                                        |a, b| square_max(a, b),
                                    ),
                                exact_squircle_bounds(&self.current_gen_config, 1.1),
                            );

                            // make sure the config stored on the stack matches the current config
                            self.stack_gen_config[(self.layer - self.layer_lowest) as usize] = self.current_gen_config.clone();
                        }

                        if (ui.button("Generate current layer").clicked() || self.auto_generate) && self.layer_config_changed {
                            self.grid_size = (f64::max(
                                f64::max(self.global_bounding_box[0][0], self.global_bounding_box[0][1]),
                                f64::max(self.global_bounding_box[1][0], self.global_bounding_box[1][1]))
                                .ceil() as usize) * 2 + 4;

                            // Generate from circle with selected algorithm
                            self.current_gen_output = self.current_gen_config.generate(self.grid_size);

                            self.layer_output_changed |= true;
                            self.layer_config_changed = false;
                        }
                    });

                    if columns[1].button("Set parameters by code").clicked() {
                        for layer in self.layer_lowest..=self.layer_highest {
                            self.lua.globals().set("layer", layer).unwrap();

                            self.lua_field_radius_a.eval(
                                &mut self.lua,
                                &mut self.stack_gen_config.get_mut((layer - self.layer_lowest) as usize).unwrap().radius_a,
                            );
                            if self.circle_mode { // force both radii to be equal, so generate them with the same code
                                self.lua_field_radius_a.eval(
                                    &mut self.lua,
                                    &mut self.stack_gen_config.get_mut((layer - self.layer_lowest) as usize).unwrap().radius_b,
                                );
                            } else {
                                self.lua_field_radius_b.eval(
                                    &mut self.lua,
                                    &mut self.stack_gen_config.get_mut((layer - self.layer_lowest) as usize).unwrap().radius_b,
                                );
                            }

                            self.lua_field_tilt.eval(
                                &mut self.lua,
                                &mut self.stack_gen_config.get_mut((layer - self.layer_lowest) as usize).unwrap().tilt,
                            );

                            self.lua_field_squircle_parameter.eval(
                                &mut self.lua,
                                &mut self.stack_gen_config.get_mut((layer - self.layer_lowest) as usize).unwrap().squircle_parameter,
                            );

                            self.lua_field_center_offset_x.eval(
                                &mut self.lua,
                                &mut self.stack_gen_config.get_mut((layer - self.layer_lowest) as usize).unwrap().center_offset_x,
                            );
                            self.lua_field_center_offset_y.eval(
                                &mut self.lua,
                                &mut self.stack_gen_config.get_mut((layer - self.layer_lowest) as usize).unwrap().center_offset_y,
                            );
                        }
                        self.lua_field_radius_a.register_success();
                        self.lua_field_radius_b.register_success();
                        self.lua_field_tilt.register_success();
                        self.lua_field_squircle_parameter.register_success();
                        self.lua_field_center_offset_x.register_success();
                        self.lua_field_center_offset_y.register_success();

                        self.current_gen_config = self.stack_gen_config[(self.layer - self.layer_lowest) as usize].clone();

                        self.layer_config_changed |= true;
                    }

                    columns[1].centered_and_justified(|ui| {
                        if ui.button("Generate all layers").clicked() {

                            // Generate all layers
                            self.stack_gen_output = self.stack_gen_config.iter().map(|config| config.generate(self.grid_size)).collect();

                            // Update current layer
                            self.current_gen_config = self.stack_gen_config[(self.layer - self.layer_lowest) as usize].clone();
                            self.current_gen_output = self.stack_gen_output[(self.layer - self.layer_lowest) as usize].clone();

                            // update metrics for this layer
                            self.nr_blocks_total = self.current_gen_output.blocks_all.get_nr_blocks();
                            self.nr_blocks_interior = self.current_gen_output.blocks_interior.get_nr_blocks();
                            self.nr_blocks_boundary = self.current_gen_output.blocks_boundary.get_nr_blocks();

                            self.outer_corners = self.current_gen_output.blocks_all.get_outer_corners();
                            self.convex_hull = get_convex_hull(&self.outer_corners);

                            self.layer_config_changed = false;
                            self.layer_output_changed |= true;
                        }
                    });

                    if self.layer_output_changed {
                        // update stack with new information
                        self.stack_gen_output[(self.layer - self.layer_lowest) as usize] = self.current_gen_output.clone();

                        // update metrics for this layer
                        self.nr_blocks_total = self.current_gen_output.blocks_all.get_nr_blocks();
                        self.nr_blocks_interior = self.current_gen_output.blocks_interior.get_nr_blocks();
                        self.nr_blocks_boundary = self.current_gen_output.blocks_boundary.get_nr_blocks();

                        self.outer_corners = self.current_gen_output.blocks_all.get_outer_corners();
                        self.convex_hull = get_convex_hull(&self.outer_corners);

                        self.boundary_3d = boundary_3d(
                            &self.stack_gen_output,
                            self.layer_lowest,
                            self.layer_highest,
                            true,
                            true,
                        );

                        self.layer_output_changed = false;
                    }
                });
            });

        });

        // Status bar (bottom)
        egui::TopBottomPanel::bottom("status-bar").show(ctx, |ui| {
            ui.with_layout(Layout {
                main_dir: Direction::LeftToRight,
                main_wrap: true,
                main_align: Align::Center,
                main_justify: true,
                cross_align: Align::Center,
                cross_justify: true,
            }, |ui| {
                // Easier to format as single string (want it centered)
                ui.label(
                    format!(
                        // "nr. blocks: {}, nr. boundary blocks: {}, nr. interior blocks: {}, {}, build sequence: {:?}, program by Basyniae",
                        "nr. blocks: {}, nr. boundary blocks: {}, nr. interior blocks: {}, {}, program by Basyniae",
                        formatting::format_block_count(self.nr_blocks_total),
                        formatting::format_block_count(self.nr_blocks_boundary),
                        formatting::format_block_count(self.nr_blocks_interior),
                        formatting::format_block_diameter(self.current_gen_output.blocks_all.get_diameters()),
                        //self.blocks_all.get_build_sequence() //FIXME: Redo, note it doesn't make sense for *tilted* superellipses (or non-centered ones?)
                    )
                )
            })
        });

        // Layer navigation bar (top)
        egui::TopBottomPanel::top("layer-navigation").show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                let mut has_layer_changed = false;
                let mut has_stack_changed = false;
                let prev_layer_lowest = self.layer_lowest;
                let prev_layer_highest = self.layer_highest;

                let prev_layer = self.layer;
                // Finicky due to not being able to know the size of the widget in advance
                // so do a pretty good prediction
                let height = ui.style().spacing.interact_size.y;
                let controls_width = height + 10.0;
                let main_width = ui.style().spacing.interact_size.x; // Incorrect for >4 digits (which is unlikely to occur)
                let padding = ui.style().spacing.button_padding.x; //Button size is text size plus this on each side

                let (rect, response) = ui.allocate_exact_size(
                    [
                        4.0 * controls_width + 3.0 * main_width + padding * 12.0,
                        height,
                    ]
                    .into(),
                    egui::Sense::click(),
                );
                ui.put(rect, |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::DragValue::new(&mut self.layer_lowest).speed(0.05))
                            .changed()
                        {
                            has_stack_changed = true
                        }

                        if ui
                            .add(
                                egui::Button::new("|<")
                                    .min_size(egui::Vec2::from([controls_width, height])),
                            )
                            .clicked()
                        {
                            self.layer = self.layer_lowest;
                            has_layer_changed = true;
                            has_stack_changed |= (self.layer < self.layer_lowest)
                                || (self.layer > self.layer_highest);
                        }

                        if ui
                            .add(
                                egui::Button::new("<")
                                    .min_size(egui::Vec2::from([controls_width, height])),
                            )
                            .clicked()
                        {
                            self.layer = self.layer.saturating_sub(1);
                            has_layer_changed = true;
                            has_stack_changed |= (self.layer < self.layer_lowest)
                                || (self.layer > self.layer_highest);
                        }

                        let central_field =
                            ui.add(egui::DragValue::new(&mut self.layer).speed(0.05));
                        if central_field.clicked() || central_field.drag_released() {
                            has_layer_changed = true;
                            has_stack_changed |= (self.layer < self.layer_lowest)
                                || (self.layer > self.layer_highest);
                        }

                        if ui
                            .add(
                                egui::Button::new(">")
                                    .min_size(egui::Vec2::from([controls_width, height])),
                            )
                            .clicked()
                        {
                            self.layer = self.layer.saturating_add(1);
                            has_layer_changed = true;
                            has_stack_changed |= (self.layer < self.layer_lowest)
                                || (self.layer > self.layer_highest);
                        }

                        if ui
                            .add(
                                egui::Button::new(">|")
                                    .min_size(egui::Vec2::from([controls_width, height])),
                            )
                            .clicked()
                        {
                            self.layer = self.layer_highest;
                            has_layer_changed = true;
                            has_stack_changed |= (self.layer < self.layer_lowest)
                                || (self.layer > self.layer_highest);
                        }

                        if ui
                            .add(egui::DragValue::new(&mut self.layer_highest).speed(0.05))
                            .changed()
                        {
                            has_stack_changed = true
                        }
                    });
                    response
                });

                if has_layer_changed {
                    // Save old configuration to stack
                    self.stack_gen_config[(prev_layer - self.layer_lowest) as usize] =
                        self.current_gen_config.clone();
                    self.stack_gen_output[(prev_layer - self.layer_lowest) as usize] =
                        self.current_gen_output.clone();
                }

                // Check if enough (empty) layers are initialized, else initialize more. Also delete old layers.
                if has_stack_changed {
                    // Update lower and upper bounds
                    self.layer_lowest = self.layer_lowest.min(self.layer);
                    self.layer_highest = self.layer_highest.max(self.layer);

                    // change on the upper bound
                    if prev_layer_highest < self.layer_highest {
                        // upper bound increases, extend
                        self.stack_gen_config.append(&mut VecDeque::from(
                            (prev_layer_highest..self.layer_highest) // array of length we want to add
                                .map(|_| self.current_gen_config.clone())
                                .collect::<Vec<GenConfig>>(),
                        ));
                        self.stack_gen_output.append(&mut VecDeque::from(
                            (prev_layer_highest..self.layer_highest) // array of length we want to add
                                .map(|_| self.current_gen_output.clone())
                                .collect::<Vec<GenOutput>>(),
                        ));
                    } else if prev_layer_highest > self.layer_highest {
                        // upper bound decreases, truncate
                        self.stack_gen_config
                            .truncate((self.layer_highest - prev_layer_lowest + 1) as usize);
                        self.stack_gen_output
                            .truncate((self.layer_highest - prev_layer_lowest + 1) as usize);
                    }

                    // change on the lower bound
                    if prev_layer_lowest > self.layer_lowest {
                        // lower bound decreases, extend
                        for _ in self.layer_lowest..prev_layer_lowest {
                            self.stack_gen_config
                                .push_front(self.current_gen_config.clone());
                            self.stack_gen_output
                                .push_front(self.current_gen_output.clone());
                        }
                    } else if prev_layer_lowest < self.layer_lowest {
                        // lower bound increases, truncate
                        for _ in prev_layer_lowest..self.layer_lowest {
                            self.stack_gen_config.pop_front();
                            self.stack_gen_output.pop_front();
                        }
                    }

                    // update field state when the bounds increase
                    if prev_layer_lowest > self.layer_lowest
                        || prev_layer_highest < self.layer_highest
                    {
                        self.lua_field_radius_a.update_field_state(
                            &mut self.lua,
                            self.layer_lowest,
                            self.layer_highest,
                        );
                        self.lua_field_radius_b.update_field_state(
                            &mut self.lua,
                            self.layer_lowest,
                            self.layer_highest,
                        );
                        self.lua_field_tilt.update_field_state(
                            &mut self.lua,
                            self.layer_lowest,
                            self.layer_highest,
                        );
                        self.lua_field_center_offset_x.update_field_state(
                            &mut self.lua,
                            self.layer_lowest,
                            self.layer_highest,
                        );
                        self.lua_field_center_offset_y.update_field_state(
                            &mut self.lua,
                            self.layer_lowest,
                            self.layer_highest,
                        );
                        self.lua_field_squircle_parameter.update_field_state(
                            &mut self.lua,
                            self.layer_lowest,
                            self.layer_highest,
                        );
                    }
                }

                // Load the new layer and save the layer we left
                if has_layer_changed {
                    // Set Gen Config to whatever was stored at that location
                    self.current_gen_config =
                        self.stack_gen_config[(self.layer - self.layer_lowest) as usize].clone();
                    self.current_gen_output =
                        self.stack_gen_output[(self.layer - self.layer_lowest) as usize].clone();

                    self.layer_config_changed |= true;
                    self.layer_output_changed |= true;
                }
            })
        });

        // Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.visuals_mut().extreme_bg_color = COLOR_BACKGROUND;
            ui.visuals_mut().faint_bg_color = Color32::RED;

            Plot::new("my_plot")
                .data_aspect(1.0) // so that squares in the rasterization always look square in the viewport
                // Grid lines of increasing thickness at distance 1.0, 5.0, 10.0 for counting
                .x_grid_spacer(uniform_grid_spacer(|_gridinput| [1.0, 5.0, 10.0]))
                .y_grid_spacer(uniform_grid_spacer(|_gridinput| [1.0, 5.0, 10.0]))
                .allow_boxed_zoom(false)
                // We don't need this, there's a maximal reasonable zoom in level and the reasonable zoom out level is only as big as the circle we're generating
                .auto_bounds(Vec2b::from([false, false]))
                .allow_double_click_reset(false) // we have to implement this ourselves
                .label_formatter(move |_name, mouse_coord| {
                    // if !name.is_empty() {  // Can condition formatting only on name of object! So if we want to have different tooltips for different objects this is what we must do
                    //     format!("{}: {:.*}%", name, 1, value.y)
                    // } else {
                    //     "".to_owned()
                    // } // TODO: think about integer coords for odd & even circles (no +/- zero for even circles)... ideally have it dep. only on...
                    format!(
                        "{0:.0}, {1:.0}",
                        mouse_coord.x.trunc(),
                        mouse_coord.y.trunc()
                    ) // Use trunc instead of floor for symmetry preservation around the axis! Nasty but works
                })
                .show_axes([false, false]) // Don't show number axes
                .show(ui, |plot_ui| {
                    // Reset zoom (approximates default behaviour, but we get to specify the action of automatic zooming
                    if self.reset_zoom_once || self.reset_zoom {
                        plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                            self.global_bounding_box[0],
                            self.global_bounding_box[1],
                        ));
                        self.reset_zoom_once = false
                    }

                    if plot_ui.response().clicked() || plot_ui.response().drag_started() {
                        self.reset_zoom = false
                    }

                    if plot_ui.response().double_clicked() {
                        self.reset_zoom = true // not sure if best to reset zoom once or reset zoom continuously
                    }

                    // * Viewport plotting * //
                    if self.view_voxelization {
                        for coord in self.current_gen_output.blocks_all.get_block_coords() {
                            plot_ui.polygon(
                                plotting::square_at_coords(coord)
                                    .stroke(Stroke {
                                        width: 1.0,
                                        color: COLOR_WIRE,
                                    })
                                    .fill_color(COLOR_FACE),
                            );
                        }
                    }

                    if self.view_boundary_2d {
                        for coord in self.current_gen_output.blocks_boundary.get_block_coords() {
                            plot_ui.polygon(
                                plotting::square_at_coords(coord)
                                    .stroke(Stroke {
                                        width: 1.0,
                                        color: COLOR_WIRE,
                                    })
                                    .fill_color(COLOR_LIGHT_BLUE),
                            );
                        }
                    }

                    if self.view_interior_2d {
                        for coord in self.current_gen_output.blocks_interior.get_block_coords() {
                            plot_ui.polygon(
                                plotting::square_at_coords(coord)
                                    .stroke(Stroke {
                                        width: 1.0,
                                        color: COLOR_WIRE,
                                    })
                                    .fill_color(COLOR_YELLOW),
                            );
                        }
                    }

                    if self.view_boundary_3d {
                        // need to check for this, because of update order the layer isn't generated yet
                        if self
                            .boundary_3d
                            .get((self.layer - self.layer_lowest) as usize)
                            .is_some()
                        {
                            for coord in self.boundary_3d[(self.layer - self.layer_lowest) as usize]
                                .get_block_coords()
                            {
                                plot_ui.polygon(
                                    plotting::square_at_coords(coord)
                                        .stroke(Stroke {
                                            width: 1.0,
                                            color: COLOR_WIRE,
                                        })
                                        .fill_color(COLOR_PURPLE),
                                );
                            }
                        }
                    }

                    if self.view_complement {
                        for coord in self.current_gen_output.blocks_complement.get_block_coords() {
                            plot_ui.polygon(
                                plotting::square_at_coords(coord)
                                    .stroke(Stroke {
                                        width: 1.0,
                                        color: COLOR_WIRE,
                                    })
                                    .fill_color(COLOR_ORANGE),
                            );
                        }
                    }

                    // Plot center
                    plot_ui.points(
                        Points::new(vec![[
                            self.current_gen_config.center_offset_x,
                            self.current_gen_config.center_offset_y,
                        ]])
                        .radius(5.0)
                        .color(COLOR_LIME),
                    );

                    // Plot target shape
                    plot_ui.line(
                        plotting::superellipse_at_coords(
                            self.current_gen_config.center_offset_x,
                            self.current_gen_config.center_offset_y,
                            self.current_gen_config.radius_a,
                            self.current_gen_config.radius_b,
                            self.current_gen_config.tilt,
                            self.current_gen_config.squircle_parameter,
                        )
                        .color(COLOR_LIME),
                    );

                    // Plot x and y axes through the center of the shape
                    plot_ui.hline(
                        HLine::new(self.current_gen_config.center_offset_y)
                            .color(COLOR_X_AXIS)
                            .width(2.0),
                    );
                    plot_ui.vline(
                        VLine::new(self.current_gen_config.center_offset_x)
                            .color(COLOR_Y_AXIS)
                            .width(2.0),
                    );

                    // Plot rotated x and y axes for nonzero tilt (dark orange and purple)
                    if self.current_gen_config.tilt != 0.0 {
                        let bounds = plot_ui.plot_bounds();
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                bounds,
                                self.current_gen_config.tilt,
                                self.current_gen_config.center_offset_x,
                                self.current_gen_config.center_offset_y,
                            )
                            .color(COLOR_DARK_ORANGE),
                        );
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                bounds,
                                self.current_gen_config.tilt + PI / 2.0,
                                self.current_gen_config.center_offset_x,
                                self.current_gen_config.center_offset_y,
                            )
                            .color(COLOR_PURPLE),
                        );
                    }

                    if self.view_intersect_area {
                        let grid_size = (2.0
                            * 1.42
                            * f64::max(
                                self.current_gen_config.radius_a,
                                self.current_gen_config.radius_b,
                            ))
                        .ceil() as usize
                            + 4;

                        let square = Blocks {
                            blocks: (0..grid_size.pow(2)).map(|_| true).collect(),
                            grid_size,
                            origin: Vec2::from([(grid_size / 2) as f64, (grid_size / 2) as f64]),
                        };
                        for coord in square.get_block_coords() {
                            let cell_center = [coord[0] + 0.5, coord[1] + 0.5];
                            let mut x_center =
                                cell_center[0] - self.current_gen_config.center_offset_x;
                            let mut y_center =
                                cell_center[1] - self.current_gen_config.center_offset_y;

                            // Dihedral symmetry swaps (see percentage.rs for explanation)
                            if x_center < 0.0 {
                                x_center = -x_center;
                            }
                            if y_center < 0.0 {
                                y_center = -y_center;
                            }
                            if x_center > y_center {
                                (y_center, x_center) = (x_center, y_center);
                            }

                            plot_ui.text(Text::new(PlotPoint::from(cell_center), {
                                let value = generation::percentage::cell_disk_intersection_area(
                                    self.current_gen_config
                                        .radius_a
                                        .max(self.current_gen_config.radius_b),
                                    x_center,
                                    y_center,
                                );

                                if value == 0.0 {
                                    // Don't show zero intersect area
                                    "".to_string()
                                } else {
                                    format!("{:.2}", value)
                                }
                            }));
                        }
                    }

                    // Perhaps better to use the plot_ui.shape
                    if self.view_convex_hull {
                        for i in line_segments_from_conv_hull(self.convex_hull.clone()) {
                            let pts: PlotPoints = (0..=1).map(|t| i[t]).collect();
                            plot_ui.line(Line::new(pts).color(COLOR_ORANGE));
                        }
                    }

                    // Plot outer corners of block
                    if self.view_outer_corners {
                        for [i, j] in &self.outer_corners {
                            plot_ui.points(
                                Points::new(vec![[*i, *j]])
                                    .radius(3.0)
                                    .color(COLOR_DARK_ORANGE),
                            );
                        }
                    }
                });
        });
    }
}
