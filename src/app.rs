mod generation;
mod helpers;
mod metrics;

use std::default::Default;
use std::f64::consts::PI;
use std::ops::Not;
use eframe::epaint::{ Color32, Stroke };
use eframe::egui::{ self };
use eframe::egui::{ Layout, Direction };
use egui_plot::{
    uniform_grid_spacer,
    HLine,
    Line,
    Plot,
    PlotPoint,
    PlotPoints,
    Points,
    Polygon,
    Text,
    VLine,
};

use crate::data_structures::Blocks;

use self::generation::{ generate_all_blocks, Algorithm };
use self::helpers::convex_hull::{ get_convex_hull, line_segments_from_conv_hull };

pub struct App {
    algorithm: Algorithm,

    radius: f64,
    radius_integral: u64,
    radius_fractional: f64,
    center_offset_x: f64,
    center_offset_y: f64,

    blocks_all: Blocks,
    blocks_boundary: Blocks,
    blocks_interior: Blocks,

    nr_blocks_total: u64,
    nr_blocks_interior: u64,
    nr_blocks_boundary: u64,

    auto_generate: bool,

    view_blocks_all: bool,
    view_blocks_boundary: bool,
    view_blocks_interior: bool,

    view_intersect_area: bool,

    view_convex_hull: bool,
    convex_hull: Vec<[f64; 2]>,
    outer_corners: Vec<[f64; 2]>,
}

// Defaults should be such that we get useful output on startup
// esp. some positive integral radius, auto generate on, and view blocks on
impl Default for App {
    fn default() -> Self {
        Self {
            algorithm: Default::default(),

            radius: Default::default(),
            radius_integral: 5,
            radius_fractional: Default::default(),

            center_offset_x: Default::default(),
            center_offset_y: Default::default(),

            blocks_all: Default::default(),
            blocks_boundary: Default::default(),
            blocks_interior: Default::default(),

            nr_blocks_total: Default::default(),
            nr_blocks_interior: Default::default(),
            nr_blocks_boundary: Default::default(),

            auto_generate: true,

            view_blocks_all: true,
            view_blocks_boundary: false,
            view_blocks_interior: false,
            view_intersect_area: false,

            view_convex_hull: false,
            convex_hull: Default::default(),
            outer_corners: Default::default(),
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
            ui.label("Options for generation of circle");
            ui.separator();

            // Select algorithm
            egui::ComboBox
                ::from_label("Algorithm")
                .selected_text(format!("{:?}", self.algorithm))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.algorithm,
                        Algorithm::Conservative,
                        "Conservative"
                    );
                    ui.selectable_value(
                        &mut self.algorithm,
                        Algorithm::Percentage(0.5),
                        "Percentage"
                    );
                    ui.selectable_value(&mut self.algorithm, Algorithm::Contained, "Contained");
                    ui.selectable_value(
                        &mut self.algorithm,
                        Algorithm::CenterPoint,
                        "Center point"
                    );
                    ui.selectable_value(&mut self.algorithm, Algorithm::Square, "Square");
                    ui.selectable_value(&mut self.algorithm, Algorithm::Diamond, "Diamond");
                });

            // additional algorithm options + description
            match self.algorithm {
                Algorithm::Conservative => {
                    ui.label(
                        "Include a particular block in the voxelization iff it has nonempty intersection with the disk"
                    );
                }
                Algorithm::Percentage(percentage) => {
                    ui.label(
                        format!(
                            "Include a particular block in the voxelization iff more than {:.0}% of it is contained in the disk",
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
                    self.algorithm = Algorithm::Percentage(perc_slider);
                }
                Algorithm::Contained => {
                    ui.label("Include a particular block iff it is fully contained in the disk");
                }
                Algorithm::CenterPoint => {
                    ui.label("Include a particular block iff its centerpoint is in the disk");
                }
                Algorithm::Square => {
                    ui.label(
                        "Include a particular block iff its centerpoint is in a square of the specified radius"
                    );
                }
                Algorithm::Diamond => {
                    ui.label(
                        "Include a particular block iff its centerpoint is in a diamond of the specified radius"
                    );
                }
            }

            // Radius
            ui.separator();
            ui.add(
                egui::Slider
                    ::new(&mut self.radius_integral, 0..=30)
                    .text("Integral radius")
                    .clamp_to_range(false)
            );

            ui.add(
                egui::Slider
                    ::new(&mut self.radius_fractional, -1.0..=1.0)
                    .text("Fractional radius")
                    .fixed_decimals(2)
            );

            self.radius = (&(self.radius_integral as f64) + &self.radius_fractional).clamp(
                0.0,
                f64::MAX
            );
            ui.label(format!("Radius: {:.02}", self.radius));

            // Centerpoint
            ui.separator();
            ui.add(egui::Slider::new(&mut self.center_offset_x, -1.0..=1.0).text("x offset"));
            ui.add(egui::Slider::new(&mut self.center_offset_y, -1.0..=1.0).text("y offset"));
            // Add odd and even buttons (also good so people understand what the abstraction "offset center" actually means)
            if ui.button("Even circle").clicked() {
                self.center_offset_x = 0.0;
                self.center_offset_y = 0.0;
            }
            if ui.button("Odd circle").clicked() {
                self.center_offset_x = 0.5;
                self.center_offset_y = 0.5;
            }

            // Viewport options TODO: Arrange nicely (two column layout will do the job)
            ui.separator();
            ui.checkbox(&mut self.view_blocks_all, "View all blocks");
            ui.checkbox(&mut self.view_blocks_boundary, "View boundary blocks");
            ui.checkbox(&mut self.view_blocks_interior, "View interior blocks");
            ui.checkbox(&mut self.view_intersect_area, "View intersect area");
            ui.checkbox(&mut self.view_convex_hull, "View convex hull");

            // Generate action
            ui.separator();
            ui.checkbox(&mut self.auto_generate, "Auto generate");

            ui.with_layout(Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                if ui.button("Generate").clicked() || self.auto_generate {
                    // on generation
                    // Generate from circle with selected algorithm
                    self.blocks_all = generate_all_blocks(
                        &self.algorithm,
                        self.radius,
                        self.center_offset_x,
                        self.center_offset_y
                    );

                    // run preprocessing
                    self.blocks_interior = self.blocks_all.get_interior();
                    self.blocks_boundary = Blocks {
                        // boundary is in all but not in interior (so all && interior.not())
                        blocks: self.blocks_all.blocks
                            .iter()
                            .zip(self.blocks_interior.blocks.iter())
                            .map(|(all, interior)| *all && interior.not())
                            .collect(),
                        edge_length: self.blocks_all.edge_length,
                        origin: self.blocks_all.origin,
                    };

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
            ui.with_layout(Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                // Easier to format as single string (want it centered)
                ui.label(
                    format!(
                        "nr. blocks: {}, nr. boundary blocks: {}, nr. interior blocks: {}, {}, build sequence: {:?}, program by Basyniae",
                        format_block_count(self.nr_blocks_total),
                        format_block_count(self.nr_blocks_boundary),
                        format_block_count(self.nr_blocks_interior),
                        // TODO: Better to run these once every time a new shape is generated. But it's not like we're running into performance issues
                        format_block_diamter(self.blocks_all.get_diameters()),
                        self.blocks_all.get_build_sequence()
                    )
                )
            })
        });

        // Viewport

        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("Viewport");

            Plot::new("my_plot")
                .data_aspect(1.0) // so that squares in the rasterization always look square in the viewport
                // Grid lines of increasing thickness at distance 1.0, 5.0, 10.0 for counting
                .x_grid_spacer(uniform_grid_spacer(|_gridinput| { [1.0, 5.0, 10.0] }))
                .y_grid_spacer(uniform_grid_spacer(|_gridinput| { [1.0, 5.0, 10.0] }))
                // .clamp_grid(true) // Clamp grid to the figure (so we don't have lines outside it) I don't think we want that
                .allow_boxed_zoom(false) // we shouldn't need this, there's a maximal reasonable zoom in level and the reasonable zoom out level is only as big as the circle we're generating
                // .coordinates_formatter(egui_plot::Corner::RightBottom //  this is showing the coords in a fixed place on the screen... we wanted to edit the formatting of the coords floating around the cursor
                //     CoordinatesFormatter::with_decimals(5))
                .label_formatter(move |_name, mouse_coord| {
                    // if !name.is_empty() {  // Can condition formatting only on name of object! So if we want to have different tooltips for different objects this is what we must do
                    //     format!("{}: {:.*}%", name, 1, value.y)
                    // } else {
                    //     "".to_owned()
                    // } // FIXME: think about integer coords for odd & even circles (no +/- zero for even circles)... ideally have it dep. only on
                    format!("{0:.0}, {1:.0}", mouse_coord.x.trunc(), mouse_coord.y.trunc()) // Use trunc instead of floor for symmtery preservation around the axis! Nasty but works
                })
                .include_x(self.radius + 1.0)
                .include_x(-self.radius - 1.0)
                .show(ui, |plot_ui| {
                    // * Viewport plotting * //
                    if self.view_blocks_all {
                        for coord in self.blocks_all.get_block_coords() {
                            plot_ui.polygon(
                                square_at_coords(coord)
                                    .stroke(Stroke { width: 1.0, color: Color32::BLACK })
                                    .fill_color(Color32::WHITE)
                            );
                        }
                    }

                    if self.view_blocks_boundary {
                        for coord in self.blocks_boundary.get_block_coords() {
                            plot_ui.polygon(
                                square_at_coords(coord)
                                    .stroke(Stroke { width: 1.0, color: Color32::BLACK })
                                    .fill_color(Color32::RED)
                            );
                        }
                    }

                    if self.view_blocks_interior {
                        for coord in self.blocks_interior.get_block_coords() {
                            plot_ui.polygon(
                                square_at_coords(coord)
                                    .stroke(Stroke { width: 1.0, color: Color32::BLACK })
                                    .fill_color(Color32::LIGHT_RED)
                            );
                        }
                    }

                    // Plot true center, true circle, and horizontal + vertical lines through true center
                    plot_ui.points(
                        Points::new(vec![[self.center_offset_x, self.center_offset_y]]).radius(5.0)
                    );
                    plot_ui.line(
                        circle_at_coords(self.center_offset_x, self.center_offset_y, self.radius)
                    );
                    plot_ui.hline(HLine::new(self.center_offset_y));
                    plot_ui.vline(VLine::new(self.center_offset_x));

                    // TODO: Don't display zeros. Dynamically size grid
                    if self.view_intersect_area {
                        let square = generate_all_blocks(
                            &Algorithm::Square,
                            10.0,
                            self.center_offset_x,
                            self.center_offset_y
                        );
                        for coord in square.get_block_coords() {
                            let cell_center = [coord[0] + 0.5, coord[1] + 0.5];
                            let mut x_center = cell_center[0] - self.center_offset_x;
                            let mut y_center = cell_center[1] - self.center_offset_y;

                            // Dihedral symmetry swaps (see percentage.rs for explanation
                            if x_center < 0.0 {
                                x_center = -x_center;
                            }
                            if y_center < 0.0 {
                                y_center = -y_center;
                            }
                            if x_center > y_center {
                                (y_center, x_center) = (x_center, y_center);
                            }

                            plot_ui.text(
                                Text::new(
                                    PlotPoint::from(cell_center),
                                    format!(
                                        "{:.2}",
                                        generation::percentage::cell_disk_intersection_area(
                                            self.radius,
                                            x_center,
                                            y_center
                                        )
                                    )
                                )
                            );
                        }
                    }

                    // Prehaps better to use the plot_ui.shape
                    if self.view_convex_hull {
                        for i in line_segments_from_conv_hull(self.convex_hull.clone()) {
                            let pts: PlotPoints = (0..=1).map(|t| i[t]).collect();
                            plot_ui.line(Line::new(pts).color(Color32::RED));
                        }
                    }

                    for [i, j] in &self.outer_corners {
                        plot_ui.points(
                            Points::new(vec![[*i, *j]])
                                .radius(3.0)
                                .color(Color32::BLUE)
                        );
                    }

                    // plot_ui.points(Points::new(PlotPoints::from(value)).radius(3.0).color(Color32::BLUE))
                });
        });
    }
}

fn square_at_coords(coord: [f64; 2]) -> Polygon {
    // Specifiying the bottom left coordinates of the square.
    let x = coord[0];
    let y = coord[1];

    let square_pts = PlotPoints::new(
        vec![[x + 0.0, y + 0.0], [x + 0.0, y + 1.0], [x + 1.0, y + 1.0], [x + 1.0, y + 0.0]]
    );

    Polygon::new(square_pts).name("square".to_owned())
}

fn circle_at_coords(center_x: f64, center_y: f64, radius: f64) -> Line {
    let circlepts: PlotPoints = (0..1000)
        .map(|i| {
            let t = ((i as f64) * (2.0 * PI)) / 1000.0;
            [center_x + radius * t.cos(), center_y + radius * t.sin()]
        })
        .collect();

    Line::new(circlepts)
}

fn format_block_count(nr_blocks: u64) -> String {
    if nr_blocks <= 64 {
        format!("{}", nr_blocks)
    } else {
        format!("{} = {}s{}", nr_blocks, nr_blocks.div_euclid(64), nr_blocks.rem_euclid(64))
    }
}

fn format_block_diamter(diameters: [u64; 2]) -> String {
    if diameters[0] == diameters[1] {
        format!("block diameter: {}", diameters[0])
    } else {
        format!("block diameters: {}x by {}y", diameters[0], diameters[1])
    }
}
