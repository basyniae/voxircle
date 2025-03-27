use crate::app::control::Control;
use crate::app::generation::line::LineFields;
use crate::app::generation::shape_type::ShapeType;
use crate::app::generation::squircle::SquircleFields;
use crate::app::param_config::ParamConfig;
use crate::app::update::metrics::Metrics;
use crate::app::view::View;
use data_structures::blocks::Blocks;
use data_structures::zvec::ZVec;
use eframe::egui::{Direction, Layout};
use eframe::emath::Align;
use generation::squircle::squircle_params::SquircleParams;
use generation::AllParams;
use sampling::layer_parameters::LayerParameters;
use sampling::{sampling_points_update, SampleCombineMethod, SampleDistributeMethod};
use std::collections::VecDeque;
use std::default::Default;
use ui::generation::ui_generation;
use ui::layer_navigation::ui_layer_navigation;
use ui::options::ui_options;
use ui::sampling::ui_sampling;
use ui::viewport::ui_viewport;
use ui::viewport_options::ui_viewport_options;
use update::{blocks_update, parameters_update};

mod colors;
mod control;
mod data_structures;
mod formatting;
mod generation;
mod math;
mod metrics;
mod param_config;
mod param_field;
mod plotting;
mod sampling;
mod ui;
mod update;
mod view;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

// longterm: Option to run an external rhai file
// longterm: sliders for "Dummy variables" that can be referenced in code (for easier visual tweaking)
pub struct App {
    // Layer management
    current_layer: isize,
    layer_lowest: isize,
    layer_highest: isize,

    squircle_fields: SquircleFields,
    line_fields: LineFields,

    // shape type that is currently visible and editable
    stack_shape_type: ZVec<ShapeType>,
    stack_shape_sliders: ZVec<AllParams>,
    stack_layer_parameters: ZVec<LayerParameters>,

    // additional configuration options
    param_config: ParamConfig,

    stack_blocks: ZVec<Blocks>,

    // Metrics
    metrics: Metrics,
    has_shape_changed: bool,  // has the shape changed this frame
    metrics_control: Control, // If the current layer has changed, recompute the metrics. By update order, this needs to be a global variable

    // Generate new shape on this layer automatically from the provided parameters
    blocks_current_layer_control: Control,
    blocks_all_layers_control: Control,

    layers_enabled: bool,
    lock_stack_size: bool,
    code_enabled: bool,

    // Parameter control
    parameters_current_layer_control: Control,
    parameters_all_layers_control: Control,

    // Sampling
    sampling_enabled: bool,
    only_sample_half_of_bottom_layer: bool,
    only_sample_half_of_top_layer: bool,
    nr_samples_per_layer: usize,
    sample_combine_method: SampleCombineMethod,
    sample_distribute_method: SampleDistributeMethod,
    stack_sampling_points: ZVec<Vec<f64>>,
    sampling_points_control: Control,

    // Viewport options
    view: View,

    // Zoom options (used for double click to reset zoom)
    reset_zoom_once: bool,
    reset_zoom_continuous: bool,
}

// longterm: save program state (with SERDE) as a JSON (for when working for multiple sessions on a single project)
// longterm: Export schematics (there is a rust crate for this)
impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.style_mut(|style| {
            style.spacing.slider_width = 200.0;
            style.spacing.combo_width = 200.0;
        });

        // Defaults should be such that we get useful output on startup
        Self {
            // Start on layer zero with no additional layers initialized
            current_layer: 0,
            layer_lowest: 0,
            layer_highest: 0,

            squircle_fields: Default::default(),
            line_fields: Default::default(),

            // Initialize for single layer (it will get overridden on the first update)
            stack_shape_sliders: ZVec::new(
                VecDeque::from(vec![AllParams::Squircle(SquircleParams::default())]),
                0,
            ),
            stack_layer_parameters: ZVec::new(
                VecDeque::from(vec![LayerParameters::new_from(ShapeType::Squircle)]),
                0,
            ),

            stack_shape_type: ZVec::new(VecDeque::from(vec![ShapeType::Squircle]), 0),
            has_shape_changed: false,
            param_config: Default::default(),

            stack_blocks: ZVec::new(VecDeque::from(vec![Blocks::default()]), 0),

            // Compute the metrics on the first update
            metrics_control: Control::AUTO_UPDATE,

            // Initialize empty metrics
            metrics: Default::default(),

            // Initialize on simplest working mode of operation
            blocks_current_layer_control: Control::AUTO_UPDATE,
            blocks_all_layers_control: Control::FIRST_FRAME_UPDATE,
            layers_enabled: false,
            lock_stack_size: false,

            // Code mode
            code_enabled: false,
            parameters_current_layer_control: Control::FIRST_FRAME_UPDATE,
            parameters_all_layers_control: Control::AUTO_UPDATE,

            // Sampling
            sampling_enabled: false,
            only_sample_half_of_bottom_layer: false, // todo: think about defaults
            only_sample_half_of_top_layer: false,
            nr_samples_per_layer: 1,
            sample_combine_method: SampleCombineMethod::AnySamples,
            sample_distribute_method: SampleDistributeMethod::IncludeEndpoints,
            stack_sampling_points: ZVec::new(VecDeque::from([vec![0.0]]), 0), // start with middle sample
            sampling_points_control: Control::AUTO_UPDATE,

            // Simplest working configuration
            view: Default::default(),

            // Start with continuously updating zoom
            reset_zoom_once: false,
            reset_zoom_continuous: true,
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
                    ui_options(
                        ui,
                        &mut self
                            .stack_layer_parameters
                            .get_mut(self.current_layer)
                            .unwrap()
                            .algorithm,
                        self.stack_shape_sliders
                            .get_mut(self.current_layer)
                            .unwrap(),
                        &mut self.squircle_fields,
                        &mut self.line_fields,
                        &mut self.parameters_current_layer_control,
                        &mut self.parameters_all_layers_control,
                        &self.stack_sampling_points,
                        self.code_enabled,
                        &mut self.stack_shape_type.get_mut(self.current_layer).unwrap(),
                        &mut self.param_config,
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
                        &mut self.sampling_points_control,
                    );

                    ui.label(format!(
                        "Total number of samples for all layers: {}",
                        self.stack_sampling_points.data.iter().fold(
                            0,
                            |acc, samples_for_single_layer| {
                                acc + samples_for_single_layer.len()
                            },
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
                })
                .body(|ui| {
                    ui_viewport_options(
                        ui,
                        self.layers_enabled,
                        &self.metrics.symmetry_type,
                        &mut self.view,
                    )
                });

                ui.separator();

                ui_generation(
                    ui,
                    &mut self.blocks_current_layer_control,
                    &mut self.blocks_all_layers_control,
                    &mut self.parameters_current_layer_control,
                    &mut self.parameters_all_layers_control,
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
            &mut self.sampling_points_control,
            &mut self.parameters_current_layer_control,
            &mut self.parameters_all_layers_control,
            self.layer_lowest,
            self.layer_highest,
        );

        parameters_update(
            &mut self.stack_shape_sliders,
            &mut self.stack_layer_parameters,
            &self.stack_sampling_points,
            &mut self.parameters_current_layer_control,
            &mut self.parameters_all_layers_control,
            &mut self.blocks_current_layer_control,
            &mut self.blocks_all_layers_control,
            self.current_layer,
            self.layer_lowest,
            self.layer_highest,
            &mut self.squircle_fields,
            &mut self.line_fields,
            &self.param_config,
        );

        blocks_update(
            &self.stack_layer_parameters,
            &mut self.stack_blocks,
            &mut self.blocks_current_layer_control,
            &mut self.blocks_all_layers_control,
            &mut self.metrics_control,
            self.current_layer,
            self.layer_lowest,
            &self.sample_combine_method,
        );

        if self.metrics_control.update() {
            self.metrics.update(
                self.current_layer,
                self.layer_lowest,
                self.layer_highest,
                self.stack_blocks.get(self.current_layer).unwrap(),
                &self.stack_blocks,
                &self.stack_shape_sliders,
            )
        }

        // Status bar (bottom)
        egui::TopBottomPanel::bottom("status-bar").show(ctx, |ui| {
            ui.with_layout(Layout {
                main_dir: Direction::RightToLeft,
                main_wrap: true,
                main_align: Align::Center,
                main_justify: true,
                cross_align: Align::Center,
                cross_justify: false,
            }, |ui| {
                // Easier to format as single string (want it centered)
                ui.label(
                    format!(
                        // "nr. blocks: {}, nr. boundary blocks: {}, nr. interior blocks: {}, {}, build sequence: {:?}, program by Basyniae",
                        "nr. blocks: {}, nr. boundary blocks: {}, nr. interior blocks: {}, {}, Voxircle v{} by Basyniae",
                        formatting::format_block_count(self.metrics.nr_blocks_total),
                        formatting::format_block_count(self.metrics.nr_blocks_boundary),
                        formatting::format_block_count(self.metrics.nr_blocks_interior),
                        formatting::format_block_diameter(self.stack_blocks.get_mut(self.current_layer).unwrap().get_diameters()),
                        PKG_VERSION
                    )
                );
            })
        });

        // Layer navigation bar (top)
        if self.layers_enabled {
            egui::TopBottomPanel::top("layer-navigation").show(ctx, |ui| {
                let (old_layer, has_layer_stack_changed) = ui_layer_navigation(
                    ui,
                    &mut self.current_layer,
                    &mut self.layer_lowest,
                    &mut self.layer_highest,
                    self.lock_stack_size,
                );

                if has_layer_stack_changed || self.has_shape_changed {
                    // The sampling points are (possibly) out of date.
                    // This happens certainly if the stack grows, and if the stack shrinks only if
                    //  only_sample_half_of_bottom_layer or only_sample_half_of_top_layer is true
                    self.sampling_points_control.set_outdated();

                    // Resize all the stack objects
                    {
                        macro_rules! resize {
                            ($x:ident) => {
                                self.$x.resize(
                                    self.layer_lowest,
                                    self.layer_highest,
                                    &self.$x.get(old_layer).unwrap().clone(),
                                )
                            };
                        }

                        resize!(stack_shape_type);
                        resize!(stack_shape_sliders);
                        resize!(stack_layer_parameters);
                        resize!(stack_sampling_points);
                        resize!(stack_blocks);
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
                self.stack_shape_sliders.get(self.current_layer).unwrap(),
                self.stack_layer_parameters.get(self.current_layer).unwrap(),
                &self.stack_blocks.get(self.current_layer),
                self.sampling_enabled,
                &self.view,
                self.current_layer,
                &mut self.reset_zoom_once,
                &mut self.reset_zoom_continuous,
                &self.metrics,
            );
        });
    }
}
