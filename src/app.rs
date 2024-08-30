use std::collections::VecDeque;
use std::default::Default;
use std::f64::consts::PI;

use eframe::egui::{self, Vec2b};
use eframe::egui::{Direction, Layout};
use eframe::emath::Align;
use eframe::epaint::{Color32, Stroke};
use egui_plot::{
    HLine, Line, Plot, PlotBounds, PlotPoint, PlotPoints, Points, Text, uniform_grid_spacer, VLine,
};
use mlua::Lua;

use data_structures::blocks::Blocks;
use data_structures::gen_config::GenConfig;
use data_structures::zvec::ZVec;
use generation::Algorithm;
use lua_field::LuaField;
use math::convex_hull::{get_convex_hull, line_segments_from_conv_hull};
use math::exact_squircle_bounds::exact_squircle_bounds;
use math::square_max::square_max;
use metrics::boundary_3d::boundary_3d;

mod data_structures;
mod formatting;
mod generation;
mod lua_field;
mod math;
mod metrics;
mod plotting;

// Colors based on Blender Minimal Dark scheme, 3D Viewport
const COLOR_BACKGROUND: Color32 = Color32::from_rgb(28, 28, 28); // middle background color (dark gray)
const COLOR_WIRE: Color32 = Color32::from_rgb(33, 33, 33); // "Wire" color (gray)
const COLOR_FACE: Color32 = Color32::from_rgb(161, 163, 164); // Face color (light gray)
const COLOR_LIME: Color32 = Color32::from_rgb(0, 255, 47); // "Active object" color (lime)
const COLOR_LIGHT_BLUE: Color32 = Color32::from_rgb(0, 217, 255); // "Object selected" color (light blue)
const COLOR_ORANGE: Color32 = Color32::from_rgb(255, 133, 0); // "Grease Pencil Vertex Select" color (orange)
const COLOR_DARK_ORANGE: Color32 = Color32::from_rgb(204, 106, 0); // Darker shade of orange
const COLOR_MUTED_ORANGE: Color32 = Color32::from_rgb(212, 148, 78); // Darker shade of orange
const COLOR_PURPLE: Color32 = Color32::from_rgb(179, 104, 186); // Dark purple
const COLOR_YELLOW: Color32 = Color32::from_rgb(255, 242, 0); // "Edge Angle Text" color (yellow)
const COLOR_X_AXIS: Color32 = Color32::from_rgb(123, 34, 34); // x-axis color (red)
const COLOR_Y_AXIS: Color32 = Color32::from_rgb(44, 107, 44); // y-axis color (green)

pub struct App {
    // Layer management
    current_layer: isize,
    layer_lowest: isize,
    layer_highest: isize,

    stack_gen_config: ZVec<GenConfig>, // Store the configuration for each layer, handily indexed by integers
    stack_blocks: ZVec<Blocks>,        // Store the blocks for each layer

    recompute_metrics: bool, // If the current layer has changed, recompute the metrics. By update order, this needs to be a global variable

    // Metrics
    nr_blocks_total: u64,
    nr_blocks_interior: u64,
    nr_blocks_boundary: u64,
    boundary_2d: Blocks,
    interior_2d: Blocks,
    complement_2d: Blocks,
    boundary_3d: ZVec<Blocks>,
    interior_3d: ZVec<Blocks>,
    convex_hull: Vec<[f64; 2]>, //todo: check update orders and such
    outer_corners: Vec<[f64; 2]>,

    // Generate new shape on this layer automatically from the provided parameters
    auto_generate_current_layer: bool,
    auto_generate_all_layers: bool,
    circle_mode: bool,
    layer_mode: bool,
    lua_mode: bool,

    // Viewport options
    view_blocks: bool,
    view_boundary_2d: bool,
    view_interior_2d: bool,
    view_complement: bool,
    view_intersect_area: bool,
    view_boundary_3d: bool,
    view_interior_3d: bool,
    view_convex_hull: bool,
    view_outer_corners: bool,

    // Zoom options (used for double click to reset zoom)
    reset_zoom_once: bool,
    reset_zoom_continuous: bool,

    // Lua fields
    lua: Lua, // Lua instance (only initialized once)
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
        lua.globals().set("l", 0).unwrap(); // short layer alias

        // Make math functions global for easier access (so `sqrt` instead of `math.sqrt`)
        lua.load(
            r#"
                for k, v in pairs(math) do
                  _G[k] = v
                end
            "#,
        )
        .exec()
        .unwrap();

        // Defaults should be such that we get useful output on startup
        Self {
            // Start on layer zero with no additional layers initialized
            current_layer: 0,
            layer_lowest: 0,
            layer_highest: 0,

            // Initialize for single layer (it will get overridden on the first update)
            stack_gen_config: ZVec::new(VecDeque::from(vec![GenConfig::default()]), 0),
            stack_blocks: ZVec::new(VecDeque::from(vec![Blocks::default()]), 0),

            // Compute the metrics on the first update
            recompute_metrics: true,

            // Initialize empty metrics
            nr_blocks_total: Default::default(),
            nr_blocks_interior: Default::default(),
            nr_blocks_boundary: Default::default(),
            boundary_2d: Default::default(),
            interior_2d: Default::default(),
            complement_2d: Default::default(),
            boundary_3d: ZVec::new(VecDeque::from(vec![Blocks::default()]), 0),
            interior_3d: ZVec::new(VecDeque::from(vec![Blocks::default()]), 0),
            convex_hull: Default::default(),
            outer_corners: Default::default(),

            // Initialize on simplest working mode of operation
            auto_generate_current_layer: true,
            auto_generate_all_layers: false,
            circle_mode: true,
            layer_mode: false,
            lua_mode: false,

            // ""
            view_blocks: true,
            view_boundary_2d: false,
            view_interior_2d: false,
            view_intersect_area: false,
            view_complement: false,
            view_boundary_3d: false,
            view_interior_3d: false,
            view_convex_hull: false,
            view_outer_corners: false,

            // Start with continuously updating zoom
            reset_zoom_once: false,
            reset_zoom_continuous: true,

            // Standard initializations, finite or nonnegative as necessary and sensible for the data type
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
        // Options panel TODO: make scroll area
        egui::SidePanel::right("options-panel").show(ctx, |ui| {

            // viewport options
            egui::TopBottomPanel::bottom("viewport-panel").show_inside(ui, |ui| {
                ui.heading("View");

                ui.columns(2, |columns| {
                    columns[0].checkbox(&mut self.view_blocks, "Blocks");
                    columns[0].checkbox(&mut self.view_complement, "Complement");
                    columns[0].checkbox(&mut self.view_convex_hull, "Convex hull");
                    columns[0].checkbox(&mut self.view_outer_corners, "Outer corners");
                    columns[1].checkbox(&mut self.view_boundary_2d, "Layer Boundary");
                    columns[1].checkbox(&mut self.view_interior_2d, "Layer Interior");
                    columns[1].add_enabled(self.layer_mode, egui::Checkbox::new(&mut self.view_boundary_3d, "3D Boundary"));
                    columns[1].add_enabled(self.layer_mode, egui::Checkbox::new(&mut self.view_interior_3d, "3D Interior"));
                });

                ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                {
                    ui.add_enabled(self.circle_mode, egui::Checkbox::new(&mut self.view_intersect_area, "Intersect area"));
                });
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Options");
                ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                {
                    ui.checkbox(&mut self.circle_mode, "Circle mode");
                    if ui.checkbox(&mut self.layer_mode, "Layer mode").changed() {
                        self.lua_mode &= self.layer_mode; //  if layer mode is turned off, also turn off code mode
                    };
                    ui.add_enabled(self.layer_mode, egui::Checkbox::new(&mut self.lua_mode, "Code mode"));
                });

                // Select algorithm
                egui::ComboBox
                ::from_label("Algorithm")
                    .selected_text(format!("{:?}", self.stack_gen_config.get_mut(self.current_layer).unwrap().algorithm)) // TODO: change formatting to text
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.stack_gen_config.get_mut(self.current_layer).unwrap().algorithm,
                            Algorithm::CenterPoint,
                            "Center point",
                        );
                        ui.selectable_value(
                            &mut self.stack_gen_config.get_mut(self.current_layer).unwrap().algorithm,
                            Algorithm::Conservative,
                            "Conservative",
                        );
                        ui.selectable_value(
                            &mut self.stack_gen_config.get_mut(self.current_layer).unwrap().algorithm,
                            Algorithm::Contained,
                            "Contained");
                        ui.selectable_value(
                            &mut self.stack_gen_config.get_mut(self.current_layer).unwrap().algorithm,
                            Algorithm::Percentage(0.5),
                            "Percentage",
                        );
                    });

                // additional algorithm-specific options + description
                match self.stack_gen_config.get_mut(self.current_layer).unwrap().algorithm {
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
                        ui.add(
                            egui::Slider
                            ::new(&mut perc_slider, 0.0..=1.0)
                                .text("")
                                .fixed_decimals(2)
                                .custom_formatter(|n, _| {
                                    format!("{:.0}%", n * 100.0) //  formatting of percentage slider
                                })
                        );
                        self.stack_gen_config.get_mut(self.current_layer).unwrap().algorithm = Algorithm::Percentage(perc_slider);
                    }
                    Algorithm::Empty => {
                        ui.label("Include no blocks in the voxelization");
                    }
                }

                // Radius
                ui.separator();

                if self.circle_mode {
                    if ui.add(
                        egui::Slider
                        ::new(&mut self.stack_gen_config.get_mut(self.current_layer).unwrap().radius_a, 0.0..=30.0)
                            .text("Radius")
                            .clamp_to_range(false)
                            .custom_formatter(|param, _| {
                                format!("{:.02}", param)
                            })
                    ).changed() {
                        // the code is now invalid
                        self.lua_field_radius_a.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        self.lua_field_radius_b.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                    };

                    // lua
                    if self.lua_mode {
                        self.lua_field_radius_a.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);
                    }

                    self.stack_gen_config.get_mut(self.current_layer).unwrap().radius_b = self.stack_gen_config.get(self.current_layer).unwrap().radius_a;

                    self.stack_gen_config.get_mut(self.current_layer).unwrap().tilt = 0.0;
                } else {
                    // radius a
                    if ui.add(
                        egui::Slider
                        ::new(&mut self.stack_gen_config.get_mut(self.current_layer).unwrap().radius_a, 0.0..=30.0)
                            .text("Radius A")
                            .clamp_to_range(false)
                            .custom_formatter(|param, _| {
                                format!("{:.02}", param)
                            })
                    ).changed() {
                        self.lua_field_radius_a.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                    }
                    if self.lua_mode {
                        self.lua_field_radius_a.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);
                    }

                    // radius b
                    if ui.add(
                        egui::Slider
                        ::new(&mut self.stack_gen_config.get_mut(self.current_layer).unwrap().radius_b, 0.0..=30.0)
                            .text("Radius B")
                            .clamp_to_range(false)
                            .custom_formatter(|param, _| {
                                format!("{:.02}", param)
                            })
                    ).changed() {
                        self.lua_field_radius_b.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                    }
                    if self.lua_mode {
                        self.lua_field_radius_b.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);
                    }

                    if ui.add(
                        egui::Slider
                        ::new(&mut self.stack_gen_config.get_mut(self.current_layer).unwrap().tilt, -6.28..=6.28)
                            .text("Tilt (radians)")
                            .fixed_decimals(2)
                    ).changed() {
                        self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                    };


                    // Particular values
                    ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                    {
                        if ui.button("0°").clicked() {
                            self.stack_gen_config.get_mut(self.current_layer).unwrap().tilt = 0.0;
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        }
                        if ui.button("30°").clicked() {
                            self.stack_gen_config.get_mut(self.current_layer).unwrap().tilt = PI / 6.0;
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        }
                        if ui.button("45°").clicked() {
                            self.stack_gen_config.get_mut(self.current_layer).unwrap().tilt = PI / 4.0;
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        }
                        if ui.button("1:2").clicked() {
                            self.stack_gen_config.get_mut(self.current_layer).unwrap().tilt = 0.5_f64.atan();
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        }
                        if ui.button("1:3").clicked() {
                            self.stack_gen_config.get_mut(self.current_layer).unwrap().tilt = 0.333333333333_f64.atan();
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        }
                        if ui.button("2:3").clicked() {
                            self.stack_gen_config.get_mut(self.current_layer).unwrap().tilt = 0.666666666666_f64.atan();
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        }
                        if ui.button("1:4").clicked() {
                            self.stack_gen_config.get_mut(self.current_layer).unwrap().tilt = 0.25_f64.atan();
                            self.lua_field_tilt.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        }
                    });
                    if self.lua_mode {
                        self.lua_field_tilt.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);
                    }

                    //TODO: Make circular slider for more intuitive controls (need to build this myself probably)
                }

                // Squircle parameter
                // due to the scale of the parameter this is all a bit awkward... Introduce a temporary variable for controlling it
                {
                    let mut squircle_ui_parameter = self.stack_gen_config.get_mut(self.current_layer).unwrap().get_squircle_ui_parameter();
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
                        self.lua_field_squircle_parameter.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                    };

                    // Default values

                    // Aim: Make choice of squircle parameter easy. there are distinct values at 2/3 and 1/3 we want to be exact
                    ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                    {
                        if ui.button("Circle").clicked() {
                            squircle_ui_parameter = 0.666666666666666;
                            self.lua_field_squircle_parameter.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        }
                        if ui.button("Diamond").clicked() {
                            squircle_ui_parameter = 0.5;
                            self.lua_field_squircle_parameter.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        }
                        if ui.button("Square").clicked() {
                            squircle_ui_parameter = 1.0;
                            self.lua_field_squircle_parameter.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        }
                    });
                    self.stack_gen_config.get_mut(self.current_layer).unwrap().squircle_parameter = 1.0 / (1.0 - squircle_ui_parameter) - 1.0;
                }
                // now kill the temporary variable

                if self.lua_mode {
                    self.lua_field_squircle_parameter.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);
                }


                // Centerpoint
                ui.separator();
                if ui.add(egui::Slider::new(&mut self.stack_gen_config.get_mut(self.current_layer).unwrap().center_offset_x, -1.0..=1.0).text("x offset")).changed() {
                    self.lua_field_center_offset_x.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                }
                if self.lua_mode {
                    self.lua_field_center_offset_x.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);
                }
                if ui.add(egui::Slider::new(&mut self.stack_gen_config.get_mut(self.current_layer).unwrap().center_offset_y, -1.0..=1.0).text("y offset")).changed() {
                    self.lua_field_center_offset_y.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                };
                if self.lua_mode {
                    self.lua_field_center_offset_y.show(ui, &mut self.lua, self.layer_lowest, self.layer_highest);
                }
                // Add odd and even buttons (also good so people understand what the abstraction "offset center" actually means)
                ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(Align::Min), |ui|
                {
                    if ui.button("Even center").clicked() {
                        self.stack_gen_config.get_mut(self.current_layer).unwrap().center_offset_x = 0.0;
                        self.stack_gen_config.get_mut(self.current_layer).unwrap().center_offset_y = 0.0;
                        self.lua_field_center_offset_x.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        self.lua_field_center_offset_y.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                    }
                    if ui.button("Odd center").clicked() {
                        self.stack_gen_config.get_mut(self.current_layer).unwrap().center_offset_x = 0.5;
                        self.stack_gen_config.get_mut(self.current_layer).unwrap().center_offset_y = 0.5;
                        self.lua_field_center_offset_x.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                        self.lua_field_center_offset_y.update_field_state(&mut self.lua, self.layer_lowest, self.layer_highest);
                    }
                });


                // Generate action
                ui.separator();
                
                if self.layer_mode {
                    ui.checkbox(&mut self.auto_generate_current_layer, "Auto-generate current layer");
                    ui.checkbox(&mut self.auto_generate_all_layers, "Auto-generate all layers");
                } else {
                    ui.checkbox(&mut self.auto_generate_current_layer, "Auto-generate");
                }

                if self.lua_mode {
                    ui.columns(2, |columns| {
                        if columns[0].button("Set parameters for current layers by code").clicked() {
                            self.lua.globals().set("layer", self.current_layer).unwrap();
                            self.lua.globals().set("l", self.current_layer).unwrap();

                            self.lua_field_radius_a.eval(
                                &mut self.lua,
                                &mut self.stack_gen_config.get_mut(self.current_layer).unwrap().radius_a,
                            );

                            if self.circle_mode {
                                self.lua_field_radius_a.eval(
                                    &mut self.lua,
                                    &mut self.stack_gen_config.get_mut(self.current_layer).unwrap().radius_b,
                                );
                            } else {
                                self.lua_field_radius_b.eval(
                                    &mut self.lua,
                                    &mut self.stack_gen_config.get_mut(self.current_layer).unwrap().radius_b,
                                );
                            }

                            self.lua_field_tilt.eval(
                                &mut self.lua,
                                &mut self.stack_gen_config.get_mut(self.current_layer).unwrap().tilt,
                            );

                            self.lua_field_squircle_parameter.eval(
                                &mut self.lua,
                                &mut self.stack_gen_config.get_mut(self.current_layer).unwrap().squircle_parameter,
                            );

                            self.lua_field_center_offset_x.eval(
                                &mut self.lua,
                                &mut self.stack_gen_config.get_mut(self.current_layer).unwrap().center_offset_x,
                            );
                            self.lua_field_center_offset_y.eval(
                                &mut self.lua,
                                &mut self.stack_gen_config.get_mut(self.current_layer).unwrap().center_offset_y,
                            );
                            self.lua_field_radius_a.register_success();
                            self.lua_field_radius_b.register_success();
                            self.lua_field_tilt.register_success();
                            self.lua_field_squircle_parameter.register_success();
                            self.lua_field_center_offset_x.register_success();
                            self.lua_field_center_offset_y.register_success();
                        }
                        columns[0].centered_and_justified(|ui| {
                            if ui.button("Generate current layer").clicked() || self.auto_generate_current_layer {
                                // TODO: Only auto generate if the values have changed!

                                // Generate from circle with selected algorithm
                                self.stack_blocks.set(self.current_layer, self.stack_gen_config.get_mut(self.current_layer).unwrap().generate());

                                self.recompute_metrics = true;
                            }
                        });

                        if columns[1].button("Set parameters for all layers by code").clicked() {
                            for layer in self.layer_lowest..=self.layer_highest {
                                self.lua.globals().set("layer", layer).unwrap();
                                self.lua.globals().set("l", layer).unwrap();

                                self.lua_field_radius_a.eval(
                                    &mut self.lua,
                                    &mut self.stack_gen_config.get_mut(layer).unwrap().radius_a,
                                );

                                if self.circle_mode {
                                    self.lua_field_radius_a.eval(
                                        &mut self.lua,
                                        &mut self.stack_gen_config.get_mut(layer).unwrap().radius_b,
                                    );
                                } else {
                                    self.lua_field_radius_b.eval(
                                        &mut self.lua,
                                        &mut self.stack_gen_config.get_mut(layer).unwrap().radius_b,
                                    );
                                }

                                self.lua_field_tilt.eval(
                                    &mut self.lua,
                                    &mut self.stack_gen_config.get_mut(layer).unwrap().tilt,
                                );

                                self.lua_field_squircle_parameter.eval(
                                    &mut self.lua,
                                    &mut self.stack_gen_config.get_mut(layer).unwrap().squircle_parameter,
                                );

                                self.lua_field_center_offset_x.eval(
                                    &mut self.lua,
                                    &mut self.stack_gen_config.get_mut(layer).unwrap().center_offset_x,
                                );
                                self.lua_field_center_offset_y.eval(
                                    &mut self.lua,
                                    &mut self.stack_gen_config.get_mut(layer).unwrap().center_offset_y,
                                );
                            }
                            self.lua_field_radius_a.register_success();
                            self.lua_field_radius_b.register_success();
                            self.lua_field_tilt.register_success();
                            self.lua_field_squircle_parameter.register_success();
                            self.lua_field_center_offset_x.register_success();
                            self.lua_field_center_offset_y.register_success();
                        }
                        columns[1].centered_and_justified(|ui| {
                            if ui.button("Generate all layers").clicked() || self.auto_generate_all_layers {

                                // Generate all layers
                                self.stack_blocks = ZVec::new(self.stack_gen_config.data.iter().map(|config| config.generate()).collect(), self.layer_lowest);

                                self.recompute_metrics = true;
                            }
                        });
                    });
                } else {
                    if self.layer_mode {
                        ui.columns(2, |columns| {
                            columns[0].centered_and_justified(|ui| {
                                if ui.button("Generate current layer").clicked() || self.auto_generate_current_layer {
                                    // TODO: Only auto generate if the values have changed!

                                    // Generate from circle with selected algorithm
                                    self.stack_blocks.set(self.current_layer, self.stack_gen_config.get_mut(self.current_layer).unwrap().generate());

                                    self.recompute_metrics = true;
                                }
                            });

                            columns[1].centered_and_justified(|ui| {
                                if ui.button("Generate all layers").clicked() || self.auto_generate_all_layers {

                                    // Generate all layers
                                    self.stack_blocks = ZVec::new(self.stack_gen_config.data.iter().map(|config| config.generate()).collect(), self.layer_lowest);

                                    self.recompute_metrics = true;
                                }
                            });
                        })
                    } else {
                        ui.centered_and_justified(|ui| {
                            if ui.button("Generate").clicked() || self.auto_generate_current_layer {
                                // TODO: Only auto generate if the values have changed!

                                // Generate from circle with selected algorithm
                                self.stack_blocks.set(self.current_layer, self.stack_gen_config.get_mut(self.current_layer).unwrap().generate());

                                self.recompute_metrics = true;
                            }
                        });
                    }
                }
            });

        });

        if self.recompute_metrics {
            self.recompute_metrics = false;

            // update 2d spatial metrics
            self.interior_2d = self
                .stack_blocks
                .get(self.current_layer)
                .unwrap()
                .get_interior();
            self.boundary_2d = Blocks::new(
                // boundary is in all but not in interior (so all && interior.not())
                self.stack_blocks
                    .get(self.current_layer)
                    .unwrap()
                    .blocks
                    .iter()
                    .zip(self.interior_2d.blocks.iter())
                    .map(|(all, interior)| *all && !interior)
                    .collect(),
                self.interior_2d.grid_size,
            );
            self.complement_2d = self
                .stack_blocks
                .get(self.current_layer)
                .unwrap()
                .get_complement();

            // update 3d spatial metrics
            self.boundary_3d = boundary_3d(
                &self.stack_blocks,
                self.layer_lowest,
                self.layer_highest,
                true,
                true,
            );

            self.interior_3d = ZVec::new(
                (self.layer_lowest..self.layer_highest)
                    .map(|layer| {
                        Blocks::new(
                            self.boundary_3d
                                .get(layer)
                                .unwrap()
                                .blocks
                                .iter()
                                .zip(self.stack_blocks.get(layer).unwrap().blocks)
                                .map(|(is_bdry, is_block)| is_block && !is_bdry)
                                .collect(),
                            self.stack_blocks.get(layer).unwrap().grid_size,
                        )
                    })
                    .collect(),
                self.layer_lowest,
            );

            // update numerical metrics
            self.nr_blocks_total = self
                .stack_blocks
                .get_mut(self.current_layer)
                .unwrap()
                .get_nr_blocks();
            self.nr_blocks_interior = self.interior_2d.get_nr_blocks();
            self.nr_blocks_boundary = self.boundary_2d.get_nr_blocks();

            self.outer_corners = self
                .stack_blocks
                .get_mut(self.current_layer)
                .unwrap()
                .get_outer_corners();
            self.convex_hull = get_convex_hull(&self.outer_corners);
        }

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
                        formatting::format_block_diameter(self.stack_blocks.get_mut(self.current_layer).unwrap().get_diameters()),
                        //self.blocks_all.get_build_sequence() //FIXME: Redo, note it doesn't make sense for *tilted* superellipses (or non-centered ones?)
                    )
                )
            })
        });

        // Layer navigation bar (top)
        if self.layer_mode {
            egui::TopBottomPanel::top("layer-navigation").show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    // bookkeeping for updating the configuration
                    let old_layer = self.current_layer;
                    let prev_layer_lowest = self.layer_lowest;
                    let prev_layer_highest = self.layer_highest;
                    // todo: implement 'lock' which freezes the lowest and highest layer and doesn't allow current layer to be outside that
                    //  for projects in which these bounds are known

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
                            ui.add(
                                egui::DragValue::new(&mut self.layer_lowest).speed(0.05),
                            );
                            if ui
                                .add(
                                    egui::Button::new("|<")
                                        .min_size(egui::Vec2::from([controls_width, height])),
                                )
                                .clicked()
                            {
                                self.current_layer = self.layer_lowest;
                            }
                            if ui
                                .add(
                                    egui::Button::new("<")
                                        .min_size(egui::Vec2::from([controls_width, height])),
                                )
                                .clicked()
                            {
                                self.current_layer = self.current_layer - 1;
                            }
                            ui.add(
                                egui::DragValue::new(&mut self.current_layer).speed(0.05),
                            );

                            if ui
                                .add(
                                    egui::Button::new(">")
                                        .min_size(egui::Vec2::from([controls_width, height])),
                                )
                                .clicked()
                            {
                                self.current_layer = self.current_layer + 1;
                            }
                            if ui
                                .add(
                                    egui::Button::new(">|")
                                        .min_size(egui::Vec2::from([controls_width, height])),
                                )
                                .clicked()
                            {
                                self.current_layer = self.layer_highest;
                            }
                            ui.add(
                                egui::DragValue::new(&mut self.layer_highest).speed(0.05),
                            );
                        });
                        response
                    });

                    // Check if enough (empty) layers are initialized, else initialize more
                    self.layer_lowest = self.layer_lowest.min(self.current_layer);
                    self.layer_highest = self.layer_highest.max(self.current_layer);

                    self.stack_gen_config.resize(
                        self.layer_lowest,
                        self.layer_highest,
                        self.stack_gen_config.get(old_layer).unwrap().clone(),
                    );

                    self.stack_blocks.resize(
                        self.layer_lowest,
                        self.layer_highest,
                        self.stack_blocks.get(old_layer).unwrap().clone(),
                    );

                    self.recompute_metrics = true;

                    // update field state when the bounds increase
                    if prev_layer_lowest > self.layer_lowest || prev_layer_highest < self.layer_highest
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
                })
            });
        }

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
                    if self.reset_zoom_once || self.reset_zoom_continuous {
                        let mut global_bounding_box = self
                            .stack_gen_config
                            .data
                            .iter()
                            .map(|g_c| exact_squircle_bounds(g_c, 1.1))
                            .fold(
                                [
                                    [f64::INFINITY, f64::INFINITY],
                                    [f64::NEG_INFINITY, f64::NEG_INFINITY],
                                ],
                                |a, b| square_max(a, b),
                            );

                        global_bounding_box = square_max(
                            global_bounding_box,
                            exact_squircle_bounds(
                                &self.stack_gen_config.get(self.current_layer).unwrap(),
                                1.1,
                            ),
                        );

                        plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                            global_bounding_box[0],
                            global_bounding_box[1],
                        ));
                        self.reset_zoom_once = false
                    }

                    if plot_ui.response().clicked() || plot_ui.response().drag_started() {
                        self.reset_zoom_continuous = false
                    }

                    if plot_ui.response().double_clicked() {
                        self.reset_zoom_continuous = true // not sure if best to reset zoom once or reset zoom continuously
                    }

                    // * Viewport plotting * //
                    // Draw order should be largest to smallest, so
                    // 1. blocks + complement
                    // 2. 3d boundary
                    // 3. 2d boundary
                    // 4. 2d interior
                    // 5. 3d interior
                    if self.view_blocks {
                        for coord in self
                            .stack_blocks
                            .get_mut(self.current_layer)
                            .unwrap()
                            .get_all_block_coords()
                        {
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

                    if self.view_complement {
                        for coord in self.complement_2d.get_all_block_coords() {
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

                    if self.view_boundary_3d {
                        // need to check for this, because of update order the layer isn't generated yet
                        if self.boundary_3d.get(self.current_layer).is_some() {
                            for coord in self
                                .boundary_3d
                                .get(self.current_layer)
                                .unwrap()
                                .get_all_block_coords()
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

                    if self.view_boundary_2d {
                        for coord in self.boundary_2d.get_all_block_coords() {
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
                        for coord in self.interior_2d.get_all_block_coords() {
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

                    if self.view_interior_3d {
                        // need to check for this, because of update order the layer isn't generated yet
                        if self.interior_3d.get(self.current_layer).is_some() {
                            for coord in self
                                .interior_3d
                                .get(self.current_layer)
                                .unwrap()
                                .get_all_block_coords()
                            {
                                plot_ui.polygon(
                                    plotting::square_at_coords(coord)
                                        .stroke(Stroke {
                                            width: 1.0,
                                            color: COLOR_WIRE,
                                        })
                                        .fill_color(COLOR_MUTED_ORANGE),
                                );
                            }
                        }
                    }

                    // Plot center
                    plot_ui.points(
                        Points::new(vec![[
                            self.stack_gen_config
                                .get_mut(self.current_layer)
                                .unwrap()
                                .center_offset_x,
                            self.stack_gen_config
                                .get_mut(self.current_layer)
                                .unwrap()
                                .center_offset_y,
                        ]])
                        .radius(5.0)
                        .color(COLOR_LIME),
                    );

                    // Plot target shape
                    plot_ui.line(
                        plotting::superellipse_at_coords(
                            self.stack_gen_config
                                .get_mut(self.current_layer)
                                .unwrap()
                                .center_offset_x,
                            self.stack_gen_config
                                .get_mut(self.current_layer)
                                .unwrap()
                                .center_offset_y,
                            self.stack_gen_config
                                .get_mut(self.current_layer)
                                .unwrap()
                                .radius_a,
                            self.stack_gen_config
                                .get_mut(self.current_layer)
                                .unwrap()
                                .radius_b,
                            self.stack_gen_config
                                .get_mut(self.current_layer)
                                .unwrap()
                                .tilt,
                            self.stack_gen_config
                                .get_mut(self.current_layer)
                                .unwrap()
                                .squircle_parameter,
                        )
                        .color(COLOR_LIME),
                    );

                    // Plot x and y axes through the center of the shape
                    plot_ui.hline(
                        HLine::new(
                            self.stack_gen_config
                                .get_mut(self.current_layer)
                                .unwrap()
                                .center_offset_y,
                        )
                        .color(COLOR_X_AXIS)
                        .width(2.0),
                    );
                    plot_ui.vline(
                        VLine::new(
                            self.stack_gen_config
                                .get_mut(self.current_layer)
                                .unwrap()
                                .center_offset_x,
                        )
                        .color(COLOR_Y_AXIS)
                        .width(2.0),
                    );

                    // Plot rotated x and y axes for nonzero tilt (dark orange and purple)
                    if self
                        .stack_gen_config
                        .get_mut(self.current_layer)
                        .unwrap()
                        .tilt
                        != 0.0
                    {
                        let bounds = plot_ui.plot_bounds();
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                bounds,
                                self.stack_gen_config
                                    .get_mut(self.current_layer)
                                    .unwrap()
                                    .tilt,
                                self.stack_gen_config
                                    .get_mut(self.current_layer)
                                    .unwrap()
                                    .center_offset_x,
                                self.stack_gen_config
                                    .get_mut(self.current_layer)
                                    .unwrap()
                                    .center_offset_y,
                            )
                            .color(COLOR_DARK_ORANGE),
                        );
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                bounds,
                                self.stack_gen_config
                                    .get_mut(self.current_layer)
                                    .unwrap()
                                    .tilt
                                    + PI / 2.0,
                                self.stack_gen_config
                                    .get_mut(self.current_layer)
                                    .unwrap()
                                    .center_offset_x,
                                self.stack_gen_config
                                    .get_mut(self.current_layer)
                                    .unwrap()
                                    .center_offset_y,
                            )
                            .color(COLOR_PURPLE),
                        );
                    }

                    if self.view_intersect_area {
                        let grid_size = (2.0
                            * 1.42
                            * f64::max(
                                self.stack_gen_config
                                    .get_mut(self.current_layer)
                                    .unwrap()
                                    .radius_a,
                                self.stack_gen_config
                                    .get_mut(self.current_layer)
                                    .unwrap()
                                    .radius_b,
                            ))
                        .ceil() as usize
                            + 4;

                        let square =
                            Blocks::new((0..grid_size.pow(2)).map(|_| true).collect(), grid_size);

                        for coord in square.get_all_block_coords() {
                            let cell_center = [coord[0] + 0.5, coord[1] + 0.5];
                            let mut x_center = cell_center[0]
                                - self
                                    .stack_gen_config
                                    .get_mut(self.current_layer)
                                    .unwrap()
                                    .center_offset_x;
                            let mut y_center = cell_center[1]
                                - self
                                    .stack_gen_config
                                    .get_mut(self.current_layer)
                                    .unwrap()
                                    .center_offset_y;

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
                                    self.stack_gen_config
                                        .get_mut(self.current_layer)
                                        .unwrap()
                                        .radius_a
                                        .max(
                                            self.stack_gen_config
                                                .get_mut(self.current_layer)
                                                .unwrap()
                                                .radius_b,
                                        ),
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
