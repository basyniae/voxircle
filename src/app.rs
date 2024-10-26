use std::collections::VecDeque;
use std::default::Default;

use eframe::egui::{self};
use eframe::egui::{Direction, Layout};
use eframe::emath::Align;
use mlua::Lua;

use crate::app::data_structures::sampled_parameters::SampledParameters;
use crate::app::sampling::{SampleCombineMethod, SampleDistributeMethod};
use crate::app::ui_layer_navigation::ui_layer_navigation;
use crate::app::ui_sampling::ui_sampling;
use crate::app::ui_viewport::ui_viewport;
use crate::app::update_logic::{blocks_update, parameters_update, sampling_points_update};
use crate::app::update_metrics::update_metrics;
use data_structures::blocks::Blocks;
use data_structures::layer_config::LayerConfig;
use data_structures::zvec::ZVec;
use lua_field::LuaField;

mod colors;
mod data_structures;
mod formatting;
mod generation;
mod lua_field;
mod math;
mod metrics;
mod plotting;
mod sampling;
mod ui_generation;
mod ui_layer_navigation;
mod ui_options;
mod ui_sampling;
mod ui_viewport;
mod update_logic;
mod update_metrics;

pub struct App {
    // Layer management
    current_layer: isize,
    layer_lowest: isize,
    layer_highest: isize,

    stack_layer_config: ZVec<LayerConfig>, // Store the configuration for each layer, handily indexed by integers
    stack_sampled_parameters: ZVec<SampledParameters>, // Store the configuration for each layer, handily indexed by integers
    stack_blocks: ZVec<Blocks>,                        // Store the blocks for each layer

    recompute_metrics: bool, // If the current layer has changed, recompute the metrics. By update order, this needs to be a global variable
    // longterm: there is no need for auto_recompute_metrics right now... though it might be good if recomputing metrics gets slow later on

    // Metrics
    nr_blocks_total: u64,
    nr_blocks_interior: u64,
    nr_blocks_boundary: u64,
    boundary_2d: Blocks,
    interior_2d: Blocks,
    complement_2d: Blocks,
    boundary_3d: ZVec<Blocks>,
    interior_3d: ZVec<Blocks>,
    convex_hull: Vec<[f64; 2]>,
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

    global_bounding_box: [[f64; 2]; 2], // Is for viewport zoom. Update with metrics

    // Zoom options (used for double click to reset zoom)
    reset_zoom_once: bool,
    reset_zoom_continuous: bool,

    // Lua fields
    lua: Lua, // Lua instance (only initialized once)
    // Longterm: for easily adding more shapes with potentially variable inputs, make this attached to the algorithm?
    // longterm: Option to run an external lua file
    // longterm: sliders for "Dummy variables" that can be referenced in code (for easier visual tweaking)
    lua_field_radius_a: LuaField,
    lua_field_radius_b: LuaField,
    lua_field_tilt: LuaField,
    lua_field_center_offset_x: LuaField,
    lua_field_center_offset_y: LuaField,
    lua_field_squircle_parameter: LuaField,
}

// longterm: save program state (with SERDE) as a JSON (for when working for multiple sessions on a single project)
// longterm: Export schematics (there is a rust crate for this)
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
            layers_enabled: false,
            lock_stack_size: false,

            // Code mode
            code_enabled: false,
            parameters_current_layer_sample_once: true, // on startup, get the parameters from the current configuration
            parameters_current_layer_sample_auto: false, // (Can only be turned off/on when sampling is enabled)
            parameters_current_layer_is_outdated: false,
            parameters_all_layers_sample_once: false,
            parameters_all_layers_sample_auto: true, // (Can only be turned off when sampling is
            // enabled, leave on to automatically get the parameters from the code/sliders)
            parameters_all_layers_is_outdated: false,

            // Sampling
            sampling_enabled: false,
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

            global_bounding_box: [[0.0; 2]; 2],

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
        // Side panel
        egui::SidePanel::right("options-panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let id = ui.make_persistent_id("parameters_collapsable");
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    true,
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
                    false,
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
                        &mut self.only_sample_half_of_bottom_layer,
                        &mut self.only_sample_half_of_top_layer,
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
                });

                let id = ui.make_persistent_id("viewport_options_collapsable");
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    true,
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

        sampling_points_update(
            self.only_sample_half_of_bottom_layer,
            self.only_sample_half_of_top_layer,
            self.nr_samples_per_layer,
            self.sample_distribute_method,
            &mut self.stack_sampling_points,
            &mut self.sampling_points_compute_once,
            self.sampling_points_compute_auto,
            &mut self.sampling_points_is_outdated,
            &mut self.parameters_current_layer_is_outdated,
            &mut self.parameters_all_layers_is_outdated,
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
            update_metrics(
                self.current_layer,
                self.layer_lowest,
                self.layer_highest,
                self.stack_blocks.get(self.current_layer).unwrap(),
                self.stack_blocks.clone(),
                self.stack_layer_config.clone(),
                &mut self.nr_blocks_total,
                &mut self.nr_blocks_interior,
                &mut self.nr_blocks_boundary,
                &mut self.boundary_2d,
                &mut self.interior_2d,
                &mut self.complement_2d,
                &mut self.boundary_3d,
                &mut self.interior_3d,
                &mut self.convex_hull,
                &mut self.outer_corners,
                &mut self.global_bounding_box,
            )
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
                        //self.blocks_all.get_build_sequence() //longterm: Redo build sequence, note it doesn't make sense for *tilted* superellipses (or non-centered ones?)
                    )
                )
            })
        });

        // Layer navigation bar (top)
        if self.layers_enabled {
            egui::TopBottomPanel::top("layer-navigation").show(ctx, |ui| {
                let (old_layer, has_layer_stack_changed, _has_layer_changed) = ui_layer_navigation(
                    ui,
                    &mut self.current_layer,
                    &mut self.layer_lowest,
                    &mut self.layer_highest,
                    self.lock_stack_size,
                );

                if has_layer_stack_changed {
                    // The sampling points are (possibly) out of date.
                    // This happens certainly if the stack grows, and if the stack shrinks only if
                    //  only_sample_half_of_bottom_layer or only_sample_half_of_top_layer is true
                    self.sampling_points_is_outdated = true;

                    // Resize all the stack objects
                    {
                        self.stack_layer_config.resize(
                            self.layer_lowest,
                            self.layer_highest,
                            self.stack_layer_config.get(old_layer).unwrap().clone(),
                        );

                        self.stack_sampled_parameters.resize(
                            self.layer_lowest,
                            self.layer_highest,
                            self.stack_sampled_parameters
                                .get(old_layer)
                                .unwrap()
                                .clone(),
                        );

                        self.stack_blocks.resize(
                            self.layer_lowest,
                            self.layer_highest,
                            self.stack_blocks.get(old_layer).unwrap().clone(),
                        );

                        self.stack_sampling_points.resize(
                            self.layer_lowest,
                            self.layer_highest,
                            self.stack_sampling_points.get(old_layer).unwrap().clone(),
                        );
                    }
                }

                // Updating the field state when the bounds increase is not necessary,
                //  as the changing sampling points invalidates the field states already.
            });
        }

        // Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            ui_viewport(
                ui,
                self.global_bounding_box,
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
                self.boundary_2d.clone(), // fixme: this is weird. should be unnecessary
                self.interior_2d.clone(),
                self.complement_2d.clone(),
                self.boundary_3d.get(self.current_layer),
                self.interior_3d.get(self.current_layer),
                &self.convex_hull,
                &self.outer_corners,
            )
        });
    }
}
