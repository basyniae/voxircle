mod generation;
pub mod helpers;
mod metrics;

use self::generation::Algorithm;
use self::helpers::convex_hull::{get_convex_hull, line_segments_from_conv_hull};
use crate::app::helpers::gen_config::GenConfig;
use crate::app::helpers::linear_algebra::{Mat2, Vec2};
use crate::formatting;
use eframe::egui::{self, Vec2b};
use eframe::egui::{Direction, Layout};
use eframe::emath::Align;
use eframe::epaint::{Color32, Stroke};
use egui_plot::{
    uniform_grid_spacer, HLine, Line, Plot, PlotBounds, PlotPoint, PlotPoints, Points, Text, VLine,
};
use helpers::blocks::Blocks;
use helpers::plotting;
use std::default::Default;
use std::f64::consts::PI;
use std::ops::Not;

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
    current_layer_gen_config: GenConfig,
    stack_gen_config: Vec<GenConfig>, // In the order 0, 1, -1, 2, -2, 3, -3, ...
    current_layer: isize,
    current_stack_index: usize,
    // Go from stack number (nonnegative) to layer number (integer) by (-1)^(n+1) floor((n+1)/2)
    // Go from layer number to stack number by 2|l| - 1/2 (sgn(l) + 1)
    stack_lower_bound: isize,
    stack_upper_bound: isize,

    blocks_all: Blocks,
    blocks_boundary: Blocks,
    blocks_interior: Blocks,
    blocks_complement: Blocks,

    nr_blocks_total: u64,
    nr_blocks_interior: u64,
    nr_blocks_boundary: u64,

    auto_generate: bool,

    view_blocks_all: bool,
    view_blocks_boundary: bool,
    view_blocks_interior: bool,

    view_complement: bool,
    view_intersect_area: bool,

    view_convex_hull: bool,
    view_outer_corners: bool,
    convex_hull: Vec<[f64; 2]>,
    outer_corners: Vec<[f64; 2]>,
    reset_zoom_once: bool,
    reset_zoom: bool,
}

// Defaults should be such that we get useful output on startup
// esp. some positive integral radius, auto generate on, and view blocks on
impl Default for App {
    fn default() -> Self {
        Self {
            current_layer_gen_config: Default::default(),
            stack_gen_config: vec![Default::default()],
            current_layer: 0,
            current_stack_index: 0,

            stack_lower_bound: 0,
            stack_upper_bound: 0,

            blocks_all: Default::default(),
            blocks_boundary: Default::default(),
            blocks_interior: Default::default(),
            blocks_complement: Default::default(),

            nr_blocks_total: Default::default(),
            nr_blocks_interior: Default::default(),
            nr_blocks_boundary: Default::default(),

            auto_generate: true,

            view_blocks_all: true,
            view_blocks_boundary: false,
            view_blocks_interior: false,
            view_intersect_area: false,
            view_complement: false,

            view_convex_hull: false,
            view_outer_corners: false,
            convex_hull: Default::default(),
            outer_corners: Default::default(),

            reset_zoom_once: false,
            reset_zoom: true,
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.style_mut(|style| {
            style.spacing.slider_width = 200.0;
            style.spacing.combo_width = 200.0;
        });
        Self::default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Options panel
        egui::SidePanel::right("options-panel").show(ctx, |ui| {
            ui.heading("Generation");
            ui.separator();

            // Select algorithm
            egui::ComboBox
                ::from_label("Algorithm")
                .selected_text(format!("{:?}", self.current_layer_gen_config.algorithm))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.current_layer_gen_config.algorithm,
                        Algorithm::CenterPoint,
                        "Center point"
                    );
                    ui.selectable_value(
                        &mut self.current_layer_gen_config.algorithm,
                        Algorithm::Conservative,
                        "Conservative"
                    );
                    ui.selectable_value(
                        &mut self.current_layer_gen_config.algorithm,
                        Algorithm::Contained,
                        "Contained");
                    ui.selectable_value(
                        &mut self.current_layer_gen_config.algorithm,
                        Algorithm::Percentage(0.5),
                        "Percentage"
                    );
                    ui.selectable_value( // TODO: Remove, this is for debugging
                        &mut self.current_layer_gen_config.algorithm,
                                         Algorithm::Empty,
                                         "Empty");
                });

            // additional algorithm-specific options + description
            match self.current_layer_gen_config.algorithm {
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
                    self.current_layer_gen_config.algorithm = Algorithm::Percentage(perc_slider);
                }
                Algorithm::Empty => {
                    ui.label("Include no blocks in the voxelization");
                }
            }

            // Radius
            ui.separator();
            ui.checkbox(&mut self.current_layer_gen_config.circle_mode, "Circle mode");
            if self.current_layer_gen_config.circle_mode {
                ui.add(
                    egui::Slider
                    ::new(&mut self.current_layer_gen_config.radius_a, 0.0..=30.0)
                        .text("Radius")
                        .clamp_to_range(false)
                        .custom_formatter(|param, _| {
                            format!("{:.02}", param)
                        })
                );

                self.current_layer_gen_config.radius_b = self.current_layer_gen_config.radius_a;

                self.current_layer_gen_config.radius_minor = self.current_layer_gen_config.radius_a;
                self.current_layer_gen_config.radius_major = self.current_layer_gen_config.radius_a;

                self.current_layer_gen_config.tilt = 0.0;
            }
            else {
                // radius a
                ui.add(
                    egui::Slider
                    ::new(&mut self.current_layer_gen_config.radius_a, 0.0..=30.0)
                        .text("Radius A")
                        .clamp_to_range(false)
                        .custom_formatter(|param, _| {
                            format!("{:.02}", param)
                        })
                );

                // radius b
                ui.add(
                    egui::Slider
                    ::new(&mut self.current_layer_gen_config.radius_b, 0.0..=30.0)
                        .text("Radius B")
                        .clamp_to_range(false)
                        .custom_formatter(|param, _| {
                            format!("{:.02}", param)
                        })
                );

                self.current_layer_gen_config.radius_major = f64::max(self.current_layer_gen_config.radius_a, self.current_layer_gen_config.radius_b);
                self.current_layer_gen_config.radius_minor = f64::min(self.current_layer_gen_config.radius_a, self.current_layer_gen_config.radius_b);

                ui.add(
                    egui::Slider
                    ::new(&mut self.current_layer_gen_config.tilt, -6.28..=6.28)
                        .text("Tilt (radians)")
                        .fixed_decimals(2)
                );

                // Default values
                ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(egui::Align::Min), |ui|
                {
                    if ui.button("0°").clicked() {
                        self.current_layer_gen_config.tilt = 0.0;
                    }
                    if ui.button("30°").clicked() {
                        self.current_layer_gen_config.tilt = PI/6.0;
                    }
                    if ui.button("45°").clicked() {
                        self.current_layer_gen_config.tilt = PI/4.0;
                    }
                    if ui.button("1:2").clicked() {
                        self.current_layer_gen_config.tilt = 0.5_f64.atan();
                    }
                    if ui.button("1:3").clicked() {
                        self.current_layer_gen_config.tilt = 0.333333333333_f64.atan();
                    }
                    if ui.button("2:3").clicked() {
                        self.current_layer_gen_config.tilt = 0.666666666666_f64.atan();
                    }
                    if ui.button("1:4").clicked() {
                        self.current_layer_gen_config.tilt = 0.25_f64.atan();
                    }
                });

                //TODO: Make circular slider for more intuitive controls (need to build this myself probably)
            }


            // Compute sqrt of quadratic form of ellipse
            let c = self.current_layer_gen_config.tilt.cos();
            let s = self.current_layer_gen_config.tilt.sin();
            self.current_layer_gen_config.sqrt_quad_form = Mat2::from_rows(1.0 /self.current_layer_gen_config.radius_a * Vec2::from([c,s]), 1.0 / self.current_layer_gen_config.radius_b * Vec2::from([-s, c]));

            // Squircle parameter
            ui.separator();
            ui.add(egui::Slider::new(&mut self.current_layer_gen_config.squircle_ui_parameter, 0.0..=1.0)
                .text("Squircle parameter")
                .custom_formatter(|param, _| {
                    format!("{:.02}", 1.0/(1.0 - param) - 1.0)
                })
               .custom_parser(|s| {
                   s.parse::<f64>().map(|t| {
                       1.0 - 1.0 / (t + 1.0)
                   }).ok()
               })
            );

            // Default values
            ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(egui::Align::Min), |ui|
            {
                if ui.button("Circle").clicked() {
                    self.current_layer_gen_config.squircle_ui_parameter = 0.666666666666666;
                }
                if ui.button("Diamond").clicked() {
                    self.current_layer_gen_config.squircle_ui_parameter = 0.5;
                }
                if ui.button("Square").clicked() {
                    self.current_layer_gen_config.squircle_ui_parameter = 1.0;
                }
            });

            // Aim: Make choice of squircle parameter easy. there are distinct values at 2/3 and 1/3 we want to be exact
            self.current_layer_gen_config.squircle_parameter = 1.0/(1.0 - self.current_layer_gen_config.squircle_ui_parameter) - 1.0;

            // Centerpoint
            ui.separator();
            ui.add(egui::Slider::new(&mut self.current_layer_gen_config.center_offset_x, -1.0..=1.0).text("x offset"));
            ui.add(egui::Slider::new(&mut self.current_layer_gen_config.center_offset_y, -1.0..=1.0).text("y offset"));
            // Add odd and even buttons (also good so people understand what the abstraction "offset center" actually means)
            ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(egui::Align::Min), |ui|
            {
                if ui.button("Even center").clicked() {
                    self.current_layer_gen_config.center_offset_x = 0.0;
                    self.current_layer_gen_config.center_offset_y = 0.0;
                }
                if ui.button("Odd center").clicked() {
                    self.current_layer_gen_config.center_offset_x = 0.5;
                    self.current_layer_gen_config.center_offset_y = 0.5;
                }
            });

            // Viewport options
            ui.separator();
            ui.separator();
            ui.heading("View options");

            ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(egui::Align::Min), |ui|
            {
                ui.checkbox(&mut self.view_blocks_all, "All");
                ui.checkbox(&mut self.view_blocks_boundary, "Boundary");
                ui.checkbox(&mut self.view_blocks_interior, "Interior");
                ui.checkbox(&mut self.view_complement, "Complement");
            });
            ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(egui::Align::Min), |ui|
            {
                ui.checkbox(&mut self.view_convex_hull, "Convex hull");
                ui.checkbox(&mut self.view_outer_corners, "Outer corners");
            });
            ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(egui::Align::Min), |ui|
            {
                ui.checkbox(&mut self.view_intersect_area, "Intersect area (Circles only)");
            });

            // Generate action
            ui.separator();
            ui.checkbox(&mut self.auto_generate, "Auto generate");

            ui.with_layout(Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                if ui.button("Generate").clicked() || self.auto_generate {
                    // on generation

                    // Generate from circle with selected algorithm
                    [self.blocks_all, self.blocks_interior, self.blocks_boundary, self.blocks_complement] = self.current_layer_gen_config.generate();

                    // update metrics
                    self.nr_blocks_total = self.blocks_all.get_nr_blocks();
                    self.nr_blocks_interior = self.blocks_interior.get_nr_blocks();
                    self.nr_blocks_boundary = self.blocks_boundary.get_nr_blocks();

                    self.outer_corners = self.blocks_all.get_outer_corners();
                    self.convex_hull = get_convex_hull(&self.outer_corners);
                }
            });
        });

        // Status bar
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
                        // TODO: Better to run these once every time a new shape is generated. But it's not like we're running into performance issues
                        formatting::format_block_diameter(self.blocks_all.get_diameters()),
                        //self.blocks_all.get_build_sequence() //FIXME: Redo, note it doesn't make sense for *tilted* superellipses (or non-centered ones?)
                    )
                )
            })
        });

        // Layer navigation bar
        egui::TopBottomPanel::top("layer-navigation").show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                let mut has_changed = false;
                let old_stack_index = self.current_stack_index;
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
                        ui.add(egui::DragValue::new(&mut self.stack_lower_bound).speed(0.05));
                        if ui
                            .add(
                                egui::Button::new("|<")
                                    .min_size(egui::Vec2::from([controls_width, height])),
                            )
                            .clicked()
                        {
                            self.current_layer = self.stack_lower_bound;
                            has_changed = true;
                        }
                        if ui
                            .add(
                                egui::Button::new("<")
                                    .min_size(egui::Vec2::from([controls_width, height])),
                            )
                            .clicked()
                        {
                            self.current_layer = self.current_layer.saturating_sub(1);
                            has_changed = true;
                        }
                        let central_field =
                            ui.add(egui::DragValue::new(&mut self.current_layer).speed(0.05));
                        if central_field.clicked() || central_field.drag_released() {
                            has_changed = true
                        }

                        if ui
                            .add(
                                egui::Button::new(">")
                                    .min_size(egui::Vec2::from([controls_width, height])),
                            )
                            .clicked()
                        {
                            self.current_layer = self.current_layer.saturating_add(1);
                            has_changed = true
                        }
                        if ui
                            .add(
                                egui::Button::new(">|")
                                    .min_size(egui::Vec2::from([controls_width, height])),
                            )
                            .clicked()
                        {
                            self.current_layer = self.stack_upper_bound;
                            has_changed = true
                        }
                        ui.add(egui::DragValue::new(&mut self.stack_upper_bound).speed(0.05));
                    });
                    response
                });

                // Check if enough (empty) layers are initialized, else initialize more
                if has_changed {
                    self.current_stack_index = layer_number_to_stack_number(self.current_layer);

                    if self.current_stack_index >= self.stack_gen_config.len() {
                        self.stack_gen_config.extend(
                            (0..=(self.current_stack_index - self.stack_gen_config.len())).map(
                                |_| GenConfig {
                                    ..self.current_layer_gen_config
                                },
                            ),
                        )
                    }

                    // Save configuration to stack
                    self.stack_gen_config[old_stack_index] = self.current_layer_gen_config;

                    // Set Gen Config to whatever was stored at that location
                    self.current_layer_gen_config = self.stack_gen_config[self.current_stack_index];

                    // Update lower and upper bounds
                    self.stack_lower_bound = self.stack_lower_bound.min(self.current_layer);
                    self.stack_upper_bound = self.stack_upper_bound.max(self.current_layer);
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
                        let [min, max] = self.blocks_all.get_padded_bounds(1.0);
                        plot_ui.set_plot_bounds(PlotBounds::from_min_max(min, max));
                        self.reset_zoom_once = false
                    }

                    if plot_ui.response().clicked() || plot_ui.response().drag_started() {
                        self.reset_zoom = false
                    }

                    if plot_ui.response().double_clicked() {
                        self.reset_zoom = true // not sure if best to reset zoom once or reset zoom continuously
                    }

                    // * Viewport plotting * //
                    if self.view_blocks_all {
                        for coord in self.blocks_all.get_block_coords() {
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

                    if self.view_blocks_boundary {
                        for coord in self.blocks_boundary.get_block_coords() {
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

                    if self.view_blocks_interior {
                        for coord in self.blocks_interior.get_block_coords() {
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

                    if self.view_complement {
                        for coord in self.blocks_complement.get_block_coords() {
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
                            self.current_layer_gen_config.center_offset_x,
                            self.current_layer_gen_config.center_offset_y,
                        ]])
                        .radius(5.0)
                        .color(COLOR_LIME),
                    );

                    // Plot target shape
                    plot_ui.line(
                        plotting::superellipse_at_coords(
                            self.current_layer_gen_config.center_offset_x,
                            self.current_layer_gen_config.center_offset_y,
                            self.current_layer_gen_config.radius_a,
                            self.current_layer_gen_config.radius_b,
                            self.current_layer_gen_config.tilt,
                            self.current_layer_gen_config.squircle_parameter,
                        )
                        .color(COLOR_LIME),
                    );

                    // Plot x and y axes through the center of the shape
                    plot_ui.hline(
                        HLine::new(self.current_layer_gen_config.center_offset_y)
                            .color(COLOR_X_AXIS)
                            .width(2.0),
                    );
                    plot_ui.vline(
                        VLine::new(self.current_layer_gen_config.center_offset_x)
                            .color(COLOR_Y_AXIS)
                            .width(2.0),
                    );

                    // Plot rotated x and y axes for nonzero tilt (dark orange and purple)
                    if self.current_layer_gen_config.tilt != 0.0 {
                        let bounds = plot_ui.plot_bounds();
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                bounds,
                                self.current_layer_gen_config.tilt,
                                self.current_layer_gen_config.center_offset_x,
                                self.current_layer_gen_config.center_offset_y,
                            )
                            .color(COLOR_DARK_ORANGE),
                        );
                        plot_ui.line(
                            plotting::tilted_line_in_bounds(
                                bounds,
                                self.current_layer_gen_config.tilt + PI / 2.0,
                                self.current_layer_gen_config.center_offset_x,
                                self.current_layer_gen_config.center_offset_y,
                            )
                            .color(COLOR_PURPLE),
                        );
                    }

                    if self.view_intersect_area {
                        let grid_size = (2.0
                            * 1.42
                            * f64::max(
                                self.current_layer_gen_config.radius_a,
                                self.current_layer_gen_config.radius_b,
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
                                cell_center[0] - self.current_layer_gen_config.center_offset_x;
                            let mut y_center =
                                cell_center[1] - self.current_layer_gen_config.center_offset_y;

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
                                    self.current_layer_gen_config.radius_major,
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

fn layer_number_to_stack_number(layer_number: isize) -> usize {
    if layer_number == 0 {
        0
    } else {
        (2 * layer_number.abs() - (layer_number.signum() + 1) / 2) as usize
    }
}

fn stack_number_to_layer_number(stack_number: usize) -> isize {
    let stack_number = stack_number as isize;
    if stack_number == 0 {
        0
    } else if stack_number % 2 == 1 {
        1 + stack_number / 2
    } else {
        -stack_number / 2
    }
}
