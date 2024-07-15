mod generation;
pub mod helpers;
mod metrics;

use crate::app::helpers::lin_alg::{Mat2, Vec2};
use helpers::blocks::Blocks;
use eframe::egui::{self};
use eframe::egui::{Direction, Layout};
use eframe::epaint::{Color32, Stroke};
use egui_plot::{
    HLine, Line, Plot, PlotPoint, PlotPoints, Points, Polygon, Text, uniform_grid_spacer, VLine,
};
use std::default::Default;
use std::f64::consts::PI;
use std::ops::Not;
use crate::formatting;
use self::generation::{Algorithm, generate_all_blocks};
use self::helpers::convex_hull::{get_convex_hull, line_segments_from_conv_hull};

pub struct App {
    algorithm: Algorithm,

    radius_a: f64, // These two are specified (a is the x-axis if tilt = 0)
    radius_b: f64,
    radius_major: f64, // The algorithms which do not (yet) support ellipses use larger radius
    radius_minor: f64,
    tilt: f64,

    sqrt_quad_form: Mat2,
    // The square root of the PSD symmetric quadratic form X defining the ellipse:
    //  (x,y)^TX(x,y)=1
    // store [a,b,c,d] for [[a,b],[c,d]] (obviously)

    circle_mode: bool,
    
    squircle_parameter: f64,
    squircle_ui_parameter: f64,

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
            algorithm: Algorithm::Contained,

            radius_a: 2.0, // TODO: change to 5, 5 radius by default
            radius_b: 2.35,
            radius_major: Default::default(),
            radius_minor: Default::default(),

            tilt: 0.75,

            sqrt_quad_form: Mat2::from([1.0, 0.0, 0.0, 1.0]),

            center_offset_x: Default::default(),
            center_offset_y: 0.5,

            circle_mode: false, // TODO: Change default to true (this is for debugging)
            
            squircle_parameter: 0.61, // TODO: Default should be 2 (circle / ellipse mode)
            squircle_ui_parameter: 0.378882, // TODO: Default should be 0.666666666666666

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
                        Algorithm::CenterPoint,
                        "Center point"
                    );
                    ui.selectable_value(
                        &mut self.algorithm,
                        Algorithm::Conservative,
                        "Conservative"
                    );
                    ui.selectable_value(
                        &mut self.algorithm,
                        Algorithm::Contained,
                        "Contained");
                    ui.selectable_value(
                        &mut self.algorithm,
                        Algorithm::Percentage(0.5),
                        "Percentage"
                    );
                });

            // additional algorithm options + description
            match self.algorithm {
                Algorithm::CenterPoint => {
                    ui.label("Include a particular block iff its centerpoint is in the ellipse");
                }
                Algorithm::Conservative => { //TODO: implement superellipse (squircle)
                    ui.label(
                        "Include a particular block in the voxelization iff it has nonempty intersection with the ellipse"
                    );
                }
                Algorithm::Contained => { //TODO: implement superellipse
                    ui.label("Include a particular block iff it is fully contained in the ellipse");
                }
                Algorithm::Percentage(percentage) => { //TODO: implement ellipse, superellipse
                    ui.label(
                        format!(
                            "Include a particular block in the voxelization iff more than {:.0}% of it is contained in the cirlce. Ellipses and squircles not implemented.",
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
            }

            // Radius
            ui.separator();
            ui.checkbox(&mut self.circle_mode, "Circle mode");
            if self.circle_mode {
                ui.add(
                    egui::Slider
                    ::new(&mut self.radius_a, 0.0..=30.0)
                        .text("Radius")
                        .clamp_to_range(false)
                        .custom_formatter(|param, _| {
                            format!("{:.02}", param)
                        })
                );
                
                // TODO: Do we want to change all the b variables and radius to defaults?
                // Yes, take a screenshot if you want to save the settings

                self.radius_b = self.radius_a;

                self.radius_minor = self.radius_a;
                self.radius_major = self.radius_a;

                self.tilt = 0.0;
            }
            else {
                
                
                // radius a
                ui.add(
                    egui::Slider
                    ::new(&mut self.radius_a, 0.0..=30.0)
                        .text("Radius A")
                        .clamp_to_range(false)
                        .custom_formatter(|param, _| {
                            format!("{:.02}", param)
                        })
                );

                // radius b
                ui.add(
                    egui::Slider
                    ::new(&mut self.radius_b, 0.0..=30.0)
                        .text("Radius B")
                        .clamp_to_range(false)
                        .custom_formatter(|param, _| {
                            format!("{:.02}", param)
                        })
                );

                self.radius_major = f64::max(self.radius_a,self.radius_b);
                self.radius_minor = f64::min(self.radius_a,self.radius_b);
                
                // if self.radius_a == self.radius_major {
                //     ui.label(format!("Radius A: {:.02} (major)", self.radius_a));
                //     ui.label(format!("Radius B: {:.02} (minor)", self.radius_b));
                // } else {
                //     ui.label(format!("Radius A: {:.02} (minor)", self.radius_a));
                //     ui.label(format!("Radius B: {:.02} (major)", self.radius_b));
                // }

                ui.add(
                    egui::Slider
                    ::new(&mut self.tilt, -6.28..=6.28)
                        .text("Tilt (radians)")
                        .fixed_decimals(2)
                );

                // Default values
                ui.allocate_ui_with_layout(egui::Vec2::from([100.0, 200.0]), Layout::left_to_right(egui::Align::Min), |ui|
                {
                    if ui.button("0°").clicked() {
                        self.tilt = 0.0;
                    }
                    if ui.button("30°").clicked() {
                        self.tilt = PI/6.0;
                    }
                    if ui.button("45°").clicked() {
                        self.tilt = PI/4.0;
                    }
                    if ui.button("1:2").clicked() {
                        self.tilt = 0.5_f64.atan();
                    }
                    if ui.button("1:3").clicked() {
                        self.tilt = 0.333333333333_f64.atan();
                    }
                    if ui.button("2:3").clicked() {
                        self.tilt = 0.666666666666_f64.atan();
                    }
                    if ui.button("1:4").clicked() {
                        self.tilt = 0.25_f64.atan();
                    }
                });

                //TODO: Make circular slider for more intuitive controls (need to build this myself probably)
            }


            // Compute inv sqrt of quadratic for of ellipse
            let c = self.tilt.cos();
            let s = self.tilt.sin();
            self.sqrt_quad_form = Mat2::from_rows(1.0 /self.radius_a * Vec2::from([c,s]), 1.0 / self.radius_b * Vec2::from([s, -c]));

            // ui.label(format!("Sqrt of quadratic form: {:?}", self.sqrt_quad_form));
            // ui.label(format!("Transpose of sqrt quadratic form: {:?}", self.sqrt_quad_form.transpose()));
            // ui.label(format!("Quadratic form: {:?}", self.sqrt_quad_form.transpose() * self.sqrt_quad_form));

            // Squircle parameter
            ui.separator();
            ui.add(egui::Slider::new(&mut self.squircle_ui_parameter, 0.0..=1.0)
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
                    self.squircle_ui_parameter = 0.666666666666666;
                }
                if ui.button("Diamond").clicked() {
                    self.squircle_ui_parameter = 0.5;
                }
                if ui.button("Square").clicked() {
                    self.squircle_ui_parameter = 1.0;
                }
            });
            // Aim: Make choice of squircle parameter easy. there are distinct values at 2/3 and 1/3 we want to be exact
            self.squircle_parameter = 1.0/(1.0 - self.squircle_ui_parameter) - 1.0;

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
                        Vec2::from([self.center_offset_x, self.center_offset_y]),
                        self.sqrt_quad_form,
                        self.radius_major,
                        self.squircle_parameter
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
                        formatting::format_block_count(self.nr_blocks_total),
                        formatting::format_block_count(self.nr_blocks_boundary),
                        formatting::format_block_count(self.nr_blocks_interior),
                        // TODO: Better to run these once every time a new shape is generated. But it's not like we're running into performance issues
                        formatting::format_block_diameter(self.blocks_all.get_diameters()),
                        [0] //self.blocks_all.get_build_sequence() //FIXME: Redo (also doesn't make sense for ellipses I suppose
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
                .x_grid_spacer(uniform_grid_spacer(|_gridinput| [1.0, 5.0, 10.0]))
                .y_grid_spacer(uniform_grid_spacer(|_gridinput| [1.0, 5.0, 10.0]))
                // .clamp_grid(true) // Clamp grid to the figure (so we don't have lines outside it) I don't think we want that
                .allow_boxed_zoom(false) // we shouldn't need this, there's a maximal reasonable zoom in level and the reasonable zoom out level is only as big as the circle we're generating
                // .coordinates_formatter(egui_plot::Corner::RightBottom //  this is showing the coords in a fixed place on the screen... we wanted to edit the formatting of the coords floating around the cursor
                //     CoordinatesFormatter::with_decimals(5))
                .label_formatter(move |_name, mouse_coord| {
                    // if !name.is_empty() {  // Can condition formatting only on name of object! So if we want to have different tooltips for different objects this is what we must do
                    //     format!("{}: {:.*}%", name, 1, value.y)
                    // } else {
                    //     "".to_owned()
                    // } // FIXME: think about integer coords for odd & even circles (no +/- zero for even circles)... ideally have it dep. only on...
                    format!(
                        "{0:.0}, {1:.0}",
                        mouse_coord.x.trunc(),
                        mouse_coord.y.trunc()
                    ) // Use trunc instead of floor for symmtery preservation around the axis! Nasty but works
                })
                .include_x(self.radius_major + 1.0)
                .include_x(-self.radius_major - 1.0)
                .show(ui, |plot_ui| {
                    // * Viewport plotting * //
                    if self.view_blocks_all {
                        for coord in self.blocks_all.get_block_coords() {
                            plot_ui.polygon(
                                square_at_coords(coord)
                                    .stroke(Stroke {
                                        width: 1.0,
                                        color: Color32::BLACK,
                                    })
                                    .fill_color(Color32::WHITE),
                            );
                        }
                    }

                    if self.view_blocks_boundary {
                        for coord in self.blocks_boundary.get_block_coords() {
                            plot_ui.polygon(
                                square_at_coords(coord)
                                    .stroke(Stroke {
                                        width: 1.0,
                                        color: Color32::BLACK,
                                    })
                                    .fill_color(Color32::RED),
                            );
                        }
                    }

                    if self.view_blocks_interior {
                        for coord in self.blocks_interior.get_block_coords() {
                            plot_ui.polygon(
                                square_at_coords(coord)
                                    .stroke(Stroke {
                                        width: 1.0,
                                        color: Color32::BLACK,
                                    })
                                    .fill_color(Color32::LIGHT_RED),
                            );
                        }
                    }

                    // Plot true center, true circle, and horizontal + vertical lines through true center
                    plot_ui.points(
                        Points::new(vec![[self.center_offset_x, self.center_offset_y]]).radius(5.0),
                    );
                    plot_ui.line(superellipse_at_coords(
                        self.center_offset_x,
                        self.center_offset_y,
                        self.radius_a,
                        self.radius_b,
                        self.tilt,
                        self.squircle_parameter,
                    ));
                    plot_ui.hline(HLine::new(self.center_offset_y));
                    plot_ui.vline(VLine::new(self.center_offset_x));

                    if self.view_intersect_area {
                        let square = generate_all_blocks(
                            &Algorithm::CenterPoint,
                            Vec2::from([self.center_offset_x, self.center_offset_y]),
                            self.sqrt_quad_form,
                            self.radius_minor + 2.0,
                            f64::INFINITY
                        );
                        for coord in square.get_block_coords() {
                            let cell_center = [coord[0] + 0.5, coord[1] + 0.5];
                            let mut x_center = cell_center[0] - self.center_offset_x;
                            let mut y_center = cell_center[1] - self.center_offset_y;

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
                                    self.radius_major,
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

                    // Prehaps better to use the plot_ui.shape
                    if self.view_convex_hull {
                        for i in line_segments_from_conv_hull(self.convex_hull.clone()) {
                            let pts: PlotPoints = (0..=1).map(|t| i[t]).collect();
                            plot_ui.line(Line::new(pts).color(Color32::RED));
                        }
                    }

                    for [i, j] in &self.outer_corners {
                        plot_ui
                            .points(Points::new(vec![[*i, *j]]).radius(3.0).color(Color32::BLUE));
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

    let square_pts = PlotPoints::new(vec![
        [x + 0.0, y + 0.0],
        [x + 0.0, y + 1.0],
        [x + 1.0, y + 1.0],
        [x + 1.0, y + 0.0],
    ]);

    Polygon::new(square_pts).name("square".to_owned())
}

fn superellipse_at_coords( // TODO: make superellipse at coords
                           center_x: f64,
                           center_y: f64,
                           radius_a: f64,
                           radius_b: f64,
                           tilt: f64,
                           squircle_parameter: f64,
) -> Line {
    let circlepts: PlotPoints = (0..=1005)
        // Near the square (squircle_parameter = Infinity) we get weird holes (the parameterization
        //  is not equally spaced), so need a few more points for it to make sense
        .map(|i| {
            let t = ((i as f64) * (2.0 * PI)) / 1000.0;
            let notilt = [
                radius_a * t.cos().abs().powf(2.0/squircle_parameter) * t.cos().signum(),
                radius_b * t.sin().abs().powf(2.0/squircle_parameter) * t.sin().signum()
            ]; // the power is for squircles
            [
                center_x + notilt[0] * tilt.cos() + notilt[1] * tilt.sin(),
                center_y + notilt[0] * tilt.sin() - notilt[1] * tilt.cos(),
            ]
        })
        .collect();

    Line::new(circlepts)
}

