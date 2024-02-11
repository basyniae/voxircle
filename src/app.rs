mod generation;
mod preprocessing;
mod metrics;

use std::default::Default;
use std::f64::consts::PI;
use std::ops::Not;
use eframe::egui::text_selection::visuals;
use eframe::epaint::{Color32, Stroke};
use eframe::egui::{self, Visuals};
use eframe::egui::{Layout, Direction};
use egui_plot::{uniform_grid_spacer, HLine, Line, Plot, PlotPoints, Points, Polygon, VLine};

use crate::data_structures::Blocks;

use self::generation::{generate_all_blocks, Algorithm};

#[derive(Default)]
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
            egui::ComboBox::from_label("Algorithm")
                .selected_text(format!("{:?}", self.algorithm))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.algorithm, Algorithm::Conservative, "Conservative");
                    ui.selectable_value(&mut self.algorithm, Algorithm::Percentage(0.5), "Percentage");
                    ui.selectable_value(&mut self.algorithm, Algorithm::Contained, "Contained");
                    ui.selectable_value(&mut self.algorithm, Algorithm::CenterPoint, "Center point");
                    ui.selectable_value(&mut self.algorithm, Algorithm::Square, "Square");
                    ui.selectable_value(&mut self.algorithm, Algorithm::Diamond, "Diamond");
                }
            );
            
            // additional algorithm options + description
            match self.algorithm {
                Algorithm::Conservative => {
                    ui.label("Include a particular block in the voxelization iff it has nonempty intersection with the disk");
                },
                Algorithm::Percentage(percentage) => {
                    ui.label(format!("Include a particular block in the voxelization iff more than {:.0}% of it is contained in the disk", 100.0*percentage));
                    let mut perc_slider = percentage.clone();
                    ui.add(egui::Slider::new(&mut perc_slider, 0.0..=1.0).text("").fixed_decimals(2).custom_formatter(|n, _| {
                        format!("{:.0}%", n*100.0) //  formatting of percentage slider
                    }));
                    self.algorithm = Algorithm::Percentage(perc_slider);
                },
                Algorithm::Contained => {
                    ui.label("Include a particular block iff it is fully contained in the disk");
                },
                Algorithm::CenterPoint => {
                    ui.label("Include a particular block iff its centerpoint is in the disk");
                },
                Algorithm::Square => {
                    ui.label("Include a particular block iff it is in a square of the specified radius");
                },
                Algorithm::Diamond => {
                    ui.label("Include a particular block iff it is in a diamond of the specified radius");
                },
            };

            // Radius
            ui.separator();
            ui.add(egui::Slider::new(&mut self.radius_integral, 0..=30)
            .text("Integral radius")
            .clamp_to_range(false));

            ui.add(egui::Slider::new(&mut self.radius_fractional, 0.0..=1.0)
            .text("Fractional radius")
            .fixed_decimals(2));

            self.radius = (&(self.radius_integral as f64) + &self.radius_fractional).clamp(0.0, f64::MAX);
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

            // Viewport options
            ui.separator();
            ui.checkbox(&mut self.view_blocks_all, "View all blocks");
            ui.checkbox(&mut self.view_blocks_boundary, "View boundary blocks");
            ui.checkbox(&mut self.view_blocks_interior, "View interior blocks");

            // Generate action
            ui.separator();
            ui.checkbox(&mut self.auto_generate, "Auto generate");

            ui.with_layout(Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                if ui.button("Generate").clicked() || self.auto_generate { // on generation
                    // Generate from circle with selected algorithm
                    self.blocks_all = generate_all_blocks(&self.algorithm, self.radius, self.center_offset_x, self.center_offset_y);

                    // run preprocessing
                    self.blocks_interior = self.blocks_all.get_interior();
                    self.blocks_boundary = Blocks {
                        // boundary is in all but not in interior (so all && interior.not())
                        blocks: self.blocks_all.blocks.iter().zip(self.blocks_interior.blocks.iter()).map(|(all, interior)| *all && interior.not()).collect(),
                        edge_length: self.blocks_all.edge_length,
                        origin: self.blocks_all.origin
                    };

                    // update metrics
                    self.nr_blocks_total = self.blocks_all.get_nr_blocks();
                    self.nr_blocks_interior = self.blocks_interior.get_nr_blocks();
                    self.nr_blocks_boundary = self.blocks_boundary.get_nr_blocks();
                }
            });
            
        });

        // Status bar
        egui::TopBottomPanel::bottom("status-bar").show(ctx, |ui| {
            ui.with_layout(Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                // Easier to format as single string (want it centered)
                ui.label(format!(
                    "nr. blocks: {}, nr. boundary blocks: {}, nr. interior blocks: {}, build sequence: #, program by Basyniae", 
                    self.nr_blocks_total, 
                    self.nr_blocks_boundary, 
                    self.nr_blocks_interior))
            })
        });

        // Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("Viewport");

            Plot::new("my_plot")
            .data_aspect(1.0) // so that squares in the rasterization always look square in the viewport
            // Grid lines of increasing thickness at distance 1.0, 5.0, 10.0 for counting
            .x_grid_spacer(uniform_grid_spacer(|_gridinput| {
                [1.0,5.0,10.0]
            }))
            .y_grid_spacer(uniform_grid_spacer(|_gridinput| {
                [1.0,5.0,10.0]
            }))
            // .clamp_grid(true) // Clamp grid to the figure (so we don't have lines outside it) I don't think we want that
            .allow_boxed_zoom(false) // we shouldn't need this, there's a maximal reasonable zoom in level and the reasonable zoom out level is only as big as the circle we're generating
            // .coordinates_formatter(egui_plot::Corner::RightBottom //  this is showing the coords in a fixed place on the screen... we wanted to edit the formatting of the coords floating around the cursor
            //     CoordinatesFormatter::with_decimals(5))
            .label_formatter(|_name, value| {
                // if !name.is_empty() {  // Can condition formatting only on name of object! So if we want to have different tooltips for different objects this is what we must do
                //     format!("{}: {:.*}%", name, 1, value.y)
                // } else {
                //     "".to_owned()
                // }
                format!("{0:.0}, {1:.0}", value.x.trunc(), value.y.trunc()) // Use trunc instead of floor for symmtery preservation around the axis! Nasty but work
            })
            .include_x(self.radius + 1.0)
            .include_x(-self.radius - 1.0)
            .show(ui, |plot_ui| {

                // Plot all blocks
                if self.view_blocks_all {
                    for coord in self.blocks_all.get_block_coords() {
                        plot_ui.polygon(square_at_coords(coord).stroke(Stroke {width: 1.0, color: Color32::BLACK}).fill_color(Color32::WHITE));
                    }
                }

                if self.view_blocks_boundary {
                    for coord in self.blocks_boundary.get_block_coords() {
                        plot_ui.polygon(square_at_coords(coord).stroke(Stroke {width: 1.0, color: Color32::BLACK}).fill_color(Color32::RED));
                    }
                }

                if self.view_blocks_interior {
                    for coord in self.blocks_interior.get_block_coords() {
                        plot_ui.polygon(square_at_coords(coord).stroke(Stroke {width: 1.0, color: Color32::BLACK}).fill_color(Color32::BLUE));
                    }
                }

                // Plot true center, true circle, and horizontal + vertical lines through true center
                plot_ui.points(Points::new(vec![[self.center_offset_x, self.center_offset_y]]).radius(5.0));
                plot_ui.line(circle_at_coords(self.center_offset_x, self.center_offset_y, self.radius));
                plot_ui.hline(HLine::new(self.center_offset_y));
                plot_ui.vline(VLine::new(self.center_offset_x));


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
        [x + 1.0, y + 0.0]]);
    
    Polygon::new(square_pts).name("square".to_owned())
}

fn circle_at_coords(center_x: f64, center_y: f64, radius: f64) -> Line {
    let circlepts: PlotPoints = (0..1000).map(|i| {
        let t = i as f64 * (2.0 * PI) / 1000.0;
        [center_x + radius * t.cos(), center_y + radius * t.sin()]
    }).collect();

    Line::new(circlepts)
}