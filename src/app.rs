use std::collections::VecDeque;
use std::default::Default;

use eframe::egui::{self};
use eframe::egui::{Direction, Layout};
use eframe::emath::Align;
use eframe::epaint::Color32;
use mlua::Lua;

use crate::app::data_structures::sampled_parameters::SampledParameters;
use crate::app::sampling::{SampleCombineMethod, SampleDistributeMethod};
use crate::app::ui_sampling::ui_sampling;
use crate::app::ui_viewport::ui_viewport;
use crate::app::update_logic::{blocks_update, parameters_update, sampling_points_update};
use data_structures::blocks::Blocks;
use data_structures::layer_config::LayerConfig;
use data_structures::zvec::ZVec;
use lua_field::LuaField;
use math::convex_hull::get_convex_hull;
use metrics::boundary_3d::boundary_3d;

mod data_structures;
mod formatting;
mod generation;
mod lua_field;
mod math;
mod metrics;
mod plotting;
mod sampling;
mod ui_generation;
mod ui_options;
mod ui_sampling;
mod ui_viewport;
mod update_logic;

// Colors based on Blender Minimal Dark scheme, 3D Viewport
const COLOR_BACKGROUND: Color32 = Color32::from_rgb(28, 28, 28); // middle background color (dark gray)
const COLOR_WIRE: Color32 = Color32::from_rgb(33, 33, 33); // "Wire" color (gray)
const COLOR_FACE: Color32 = Color32::from_rgb(161, 163, 164); // Face color (light gray)
const COLOR_LIME: Color32 = Color32::from_rgb(0, 255, 47); // "Active object" color (lime)
const COLOR_DARK_GREEN: Color32 = Color32::from_rgb(6, 137, 30); // Slightly decreased HSL saturation, decreased saturation from Lime
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

    stack_layer_config: ZVec<LayerConfig>, // Store the configuration for each layer, handily indexed by integers
    stack_sampled_parameters: ZVec<SampledParameters>, // Store the configuration for each layer, handily indexed by integers
    stack_blocks: ZVec<Blocks>,                        // Store the blocks for each layer

    recompute_metrics: bool, // If the current layer has changed, recompute the metrics. By update order, this needs to be a global variable
    // there is no need for auto_recompute_metrics right now... though it might be good to implement if recomputing metrics gets slow later on

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
    blocks_current_layer_generate_once: bool,
    blocks_current_layer_generate_auto: bool,
    blocks_current_layer_is_outdated: bool,

    blocks_all_layers_generate_once: bool,
    blocks_all_layers_generate_auto: bool,
    blocks_all_layers_is_outdated: bool,

    single_radius: bool,
    layers_enabled: bool,
    lock_stack_size: bool,

    // Code mode
    code_enabled: bool,

    parameters_current_layer_sample_once: bool,
    parameters_current_layer_sample_auto: bool,
    parameters_current_layer_is_outdated: bool,

    parameters_all_layers_sample_once: bool,
    parameters_all_layers_sample_auto: bool,
    parameters_all_layers_is_outdated: bool,

    // Sampling
    sampling_enabled: bool,
    only_sample_half_of_bottom_layer: bool,
    only_sample_half_of_top_layer: bool,
    nr_samples_per_layer: usize,
    sample_combine_method: SampleCombineMethod,
    sample_distribute_method: SampleDistributeMethod,
    stack_sampling_points: ZVec<Vec<f64>>,

    sampling_points_compute_once: bool,
    sampling_points_compute_auto: bool,
    sampling_points_is_outdated: bool,

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
    //todo: for easily adding more shapes with potentially variable inputs, make this attached to the algorithm?
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
            stack_layer_config: ZVec::new(VecDeque::from(vec![LayerConfig::default()]), 0),
            stack_sampled_parameters: ZVec::new(
                VecDeque::from(vec![SampledParameters::default()]),
                0,
            ),
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
            blocks_current_layer_generate_auto: true,
            blocks_current_layer_generate_once: true,
            blocks_current_layer_is_outdated: true,
            blocks_all_layers_generate_auto: false,
            blocks_all_layers_generate_once: false,
            blocks_all_layers_is_outdated: false,
            single_radius: true,
            layers_enabled: true, // debug: make default false
            lock_stack_size: false,

            // Code mode
            code_enabled: true, // debug: make default false
            parameters_current_layer_sample_once: true, // on startup, get the parameters from the current configuration
            parameters_current_layer_sample_auto: false, // (Can only be turned off/on when sampling is enabled)
            parameters_current_layer_is_outdated: false,
            parameters_all_layers_sample_once: false,
            parameters_all_layers_sample_auto: true, // (Can only be turned off when sampling is
            // enabled, leave on to automatically get the parameters from the code/sliders)
            parameters_all_layers_is_outdated: false,

            // Sampling
            sampling_enabled: true, // debug: make default false
            only_sample_half_of_bottom_layer: false, // todo: think about defaults
            only_sample_half_of_top_layer: false,
            nr_samples_per_layer: 1,
            sample_combine_method: SampleCombineMethod::AnySamples,
            sample_distribute_method: SampleDistributeMethod::IncludeEndpoints,
            stack_sampling_points: ZVec::new(VecDeque::from([vec![0.0]]), 0), // start with middle sample
            sampling_points_compute_once: true,
            sampling_points_compute_auto: true,
            sampling_points_is_outdated: false,

            // Simplest working configuration
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
        // Options panel
        egui::SidePanel::right("options-panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let id = ui.make_persistent_id("parameters_collapsable");
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    false, //debug: default true
                )
                .show_header(ui, |ui| {
                    ui.label(egui::RichText::new("Parameters").strong().size(15.0));
                })
                .body(|ui| {
                    ui_options::ui_options(
                        ui,
                        self.stack_layer_config.get_mut(self.current_layer).unwrap(),
                        &mut self.single_radius,
                        self.code_enabled,
                        &mut self.lua,
                        &mut self.lua_field_radius_a,
                        &mut self.lua_field_radius_b,
                        &mut self.lua_field_tilt,
                        &mut self.lua_field_center_offset_x,
                        &mut self.lua_field_center_offset_y,
                        &mut self.lua_field_squircle_parameter,
                        &self.stack_sampling_points,
                        &mut self.parameters_current_layer_is_outdated,
                        &mut self.parameters_all_layers_is_outdated,
                    );
                });

                let id = ui.make_persistent_id("layers_collapsable");
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    false,
                )
                .show_header(ui, |ui| {
                    ui.checkbox(
                        &mut self.layers_enabled,
                        egui::RichText::new("Layers").strong().size(15.0),
                    );
                })
                .body(|ui| {
                    ui.add_enabled(
                        self.layers_enabled,
                        egui::Checkbox::new(&mut self.lock_stack_size, "Lock stack size"),
                    );
                });

                let id = ui.make_persistent_id("code_collapsable");
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    false,
                )
                .show_header(ui, |ui| {
                    ui.checkbox(
                        &mut self.code_enabled,
                        egui::RichText::new("Code").strong().size(15.0),
                    );
                })
                .body(|ui| {
                    ui.add_enabled(self.layers_enabled, egui::Label::new("No options here yet"));
                });

                let id = ui.make_persistent_id("sampling_collapsable");
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    true, //debug: set default to false (this is for testing)
                )
                .show_header(ui, |ui| {
                    if ui
                        .checkbox(
                            &mut self.sampling_enabled,
                            egui::RichText::new("Sampling").strong().size(15.0),
                        )
                        .changed()
                        & !self.sampling_enabled
                    {
                        self.nr_samples_per_layer = 1; // set number of samples to 1 if sampling is off
                    };
                })
                .body(|ui| {
                    ui_sampling(
                        ui,
                        self.sampling_enabled,
                        &mut self.only_sample_half_of_top_layer,
                        &mut self.only_sample_half_of_bottom_layer,
                        &mut self.nr_samples_per_layer,
                        &mut self.sample_combine_method,
                        &mut self.sample_distribute_method,
                        &mut self.sampling_points_compute_once,
                        &mut self.sampling_points_compute_auto,
                        &mut self.sampling_points_is_outdated,
                    );

                    ui.label(format!(
                        "Total number of samples for all layers: {}",
                        self.stack_sampling_points.data.iter().fold(
                            0,
                            |acc, samples_for_single_layer| {
                                acc + samples_for_single_layer.len()
                            }
                        )
                    ));

                    // debug: remove label
                    ui.label(format!("Sampling points: {:?}", self.stack_sampling_points))
                });

                let id = ui.make_persistent_id("viewport_options_collapsable");
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    false, //debug: true
                )
                .show_header(ui, |ui| {
                    ui.label(egui::RichText::new("Viewport").strong().size(15.0));
                    // you can put checkboxes or whatever here
                })
                .body(|ui| {
                    ui.columns(2, |columns| {
                        columns[0].checkbox(&mut self.view_blocks, "Blocks");
                        columns[0].checkbox(&mut self.view_complement, "Complement");
                        columns[0].checkbox(&mut self.view_convex_hull, "Convex hull");
                        columns[0].checkbox(&mut self.view_outer_corners, "Outer corners");

                        columns[1].checkbox(&mut self.view_boundary_2d, "Layer Boundary");
                        columns[1].checkbox(&mut self.view_interior_2d, "Layer Interior");
                        columns[1].add_enabled(
                            self.layers_enabled,
                            egui::Checkbox::new(&mut self.view_boundary_3d, "3D Boundary"),
                        );
                        columns[1].add_enabled(
                            self.layers_enabled,
                            egui::Checkbox::new(&mut self.view_interior_3d, "3D Interior"),
                        );
                    });
                    ui.add_enabled(
                        self.single_radius,
                        egui::Checkbox::new(&mut self.view_intersect_area, "Intersect area"),
                    );
                });

                ui.separator();

                ui_generation::ui_generation(
                    ui,
                    &mut self.blocks_current_layer_generate_once,
                    &mut self.blocks_current_layer_generate_auto,
                    &mut self.blocks_all_layers_generate_once,
                    &mut self.blocks_all_layers_generate_auto,
                    &mut self.parameters_current_layer_sample_once,
                    &mut self.parameters_current_layer_sample_auto,
                    &mut self.parameters_all_layers_sample_once,
                    &mut self.parameters_all_layers_sample_auto,
                    self.layers_enabled,
                    self.code_enabled,
                    self.sampling_enabled,
                );
            });
        });

        // Activates if the sampling options have changed (this update) or if the stack grows
        //  (previous update). The points may also have to be recomputed if the stack shrinks,
        //  when half_of_bottom or half_of_top layer options are implemented. Fixme.
        sampling_points_update(
            self.only_sample_half_of_bottom_layer,
            self.only_sample_half_of_top_layer,
            self.nr_samples_per_layer,
            self.sample_distribute_method,
            &mut self.stack_sampling_points,
            &mut self.sampling_points_compute_once,
            self.sampling_points_compute_auto,
            &mut self.sampling_points_is_outdated,
            self.layer_lowest,
            self.layer_highest,
        );

        parameters_update(
            &mut self.stack_layer_config,
            &mut self.stack_sampled_parameters,
            &self.stack_sampling_points,
            &mut self.parameters_current_layer_sample_once,
            self.parameters_current_layer_sample_auto,
            &mut self.parameters_current_layer_is_outdated,
            &mut self.parameters_all_layers_sample_once,
            self.parameters_all_layers_sample_auto,
            &mut self.parameters_all_layers_is_outdated,
            &mut self.blocks_current_layer_is_outdated,
            &mut self.blocks_all_layers_is_outdated,
            self.current_layer,
            self.layer_lowest,
            self.layer_highest,
            self.single_radius,
            &mut self.lua,
            &mut self.lua_field_radius_a,
            &mut self.lua_field_radius_b,
            &mut self.lua_field_tilt,
            &mut self.lua_field_center_offset_x,
            &mut self.lua_field_center_offset_y,
            &mut self.lua_field_squircle_parameter,
        );

        blocks_update(
            &self.stack_sampled_parameters,
            &mut self.stack_blocks,
            &mut self.blocks_current_layer_generate_once,
            self.blocks_current_layer_generate_auto,
            &mut self.blocks_current_layer_is_outdated,
            &mut self.blocks_all_layers_generate_once,
            self.blocks_all_layers_generate_auto,
            &mut self.blocks_all_layers_is_outdated,
            &mut self.recompute_metrics,
            self.current_layer,
            self.layer_lowest,
            self.sample_combine_method,
        );

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
        if self.layers_enabled {
            egui::TopBottomPanel::top("layer-navigation").show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    // bookkeeping for updating the configuration
                    let old_layer = self.current_layer;
                    let prev_layer_lowest = self.layer_lowest;
                    let prev_layer_highest = self.layer_highest;

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
                            ui.add_enabled(
                                !self.lock_stack_size,
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
                                if !self.lock_stack_size
                                    || self.current_layer - 1 >= self.layer_lowest
                                {
                                    self.current_layer = self.current_layer - 1;
                                }
                            }
                            if self.lock_stack_size {
                                ui.add(
                                    egui::DragValue::new(&mut self.current_layer)
                                        .speed(0.05)
                                        .clamp_range(self.layer_lowest..=self.layer_highest),
                                );
                            } else {
                                ui.add(egui::DragValue::new(&mut self.current_layer).speed(0.05));
                            }

                            if ui
                                .add(
                                    egui::Button::new(">")
                                        .min_size(egui::Vec2::from([controls_width, height])),
                                )
                                .clicked()
                            {
                                if !self.lock_stack_size
                                    || self.current_layer + 1 <= self.layer_highest
                                {
                                    self.current_layer = self.current_layer + 1;
                                }
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
                            ui.add_enabled(
                                !self.lock_stack_size,
                                egui::DragValue::new(&mut self.layer_highest).speed(0.05),
                            );
                        });
                        response
                    });

                    // Check if enough (empty) layers are initialized, else initialize more
                    self.layer_lowest = self.layer_lowest.min(self.current_layer);
                    self.layer_highest = self.layer_highest.max(self.current_layer);

                    self.stack_layer_config.resize(
                        self.layer_lowest,
                        self.layer_highest,
                        self.stack_layer_config.get(old_layer).unwrap().clone(),
                    );

                    self.stack_blocks.resize(
                        self.layer_lowest,
                        self.layer_highest,
                        self.stack_blocks.get(old_layer).unwrap().clone(),
                    );

                    //todo: is the following necessary?
                    self.stack_sampled_parameters.resize(
                        self.layer_lowest,
                        self.layer_highest,
                        self.stack_sampled_parameters
                            .get(old_layer)
                            .unwrap()
                            .clone(),
                    );

                    self.stack_sampling_points.resize(
                        self.layer_lowest,
                        self.layer_highest,
                        self.stack_sampling_points.get(old_layer).unwrap().clone(),
                    );

                    // update field state when the bounds increase
                    if prev_layer_lowest > self.layer_lowest
                        || prev_layer_highest < self.layer_highest
                    {
                        self.lua_field_radius_a
                            .update_field_state(&mut self.lua, &self.stack_sampling_points);
                        self.lua_field_radius_b
                            .update_field_state(&mut self.lua, &self.stack_sampling_points);
                        self.lua_field_tilt
                            .update_field_state(&mut self.lua, &self.stack_sampling_points);
                        self.lua_field_center_offset_x
                            .update_field_state(&mut self.lua, &self.stack_sampling_points);
                        self.lua_field_center_offset_y
                            .update_field_state(&mut self.lua, &self.stack_sampling_points);
                        self.lua_field_squircle_parameter
                            .update_field_state(&mut self.lua, &self.stack_sampling_points);

                        self.sampling_points_is_outdated = true;
                        self.recompute_metrics = true; // todo: check...
                    }
                })
            });
        }

        // Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            ui_viewport(
                ui,
                &self.stack_layer_config,
                self.stack_layer_config.get(self.current_layer).unwrap(),
                self.stack_sampled_parameters
                    .get(self.current_layer)
                    .unwrap(),
                self.stack_blocks.get(self.current_layer).unwrap(),
                self.sampling_enabled,
                self.view_blocks,
                self.view_boundary_2d,
                self.view_interior_2d,
                self.view_complement,
                self.view_intersect_area,
                self.view_boundary_3d,
                self.view_interior_3d,
                self.view_convex_hull,
                self.view_outer_corners,
                &mut self.reset_zoom_once,
                &mut self.reset_zoom_continuous,
                &self.boundary_2d,
                &self.interior_2d,
                &self.complement_2d,
                self.boundary_3d.get(self.current_layer),
                self.interior_3d.get(self.current_layer),
                &self.convex_hull,
                &self.outer_corners,
            )
        });
    }
}
