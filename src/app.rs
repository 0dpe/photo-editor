use eframe::egui_wgpu;

use crate::render::callback::ImagePaintCallback;
use crate::render::resources::{Config, ImageRenderResources};

/// Side panel width (clamped when the window is narrow).
const SIDE_PANEL_WIDTH: f32 = 120.0;
const SIDE_PANEL_MIN_WIDTH: f32 = 80.0;

/// Matches the old winit handler: one wheel line ≈ `10.0 * 0.02` → 20% zoom step.
const LINE_SCROLL_ZOOM_SCALE: f32 = 0.02;
const LINE_SCROLL_UNIT: f32 = 10.0;
/// egui converts line scroll to points using ~40 on native; use that everywhere so web
/// (where `line_scroll_speed` is 8) does not get 5× stronger zoom.
const POINTS_PER_WHEEL_LINE: f32 = 40.0;
const POINT_TO_LINE_UNIT: f32 = LINE_SCROLL_UNIT / POINTS_PER_WHEEL_LINE;
const MAX_SCROLL_AMOUNT_PER_FRAME: f32 = 15.0;

pub struct PhotoEditorApp {
    config: Config,
    cursor_pos: glam::Vec2,
    last_viewport_size: glam::Vec2,
    gpu_ready: bool,
}

impl PhotoEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.global_style_mut(|style| {
            style.spacing.window_margin = egui::Margin::ZERO;
        });

        let mut gpu_ready = false;
        let mut config = Config::default_view(glam::Vec2::ONE);

        if let Some(wgpu_render_state) = cc.wgpu_render_state.as_ref() {
            let resources = ImageRenderResources::new(wgpu_render_state);
            config = Config::default_view(resources.image_size());
            wgpu_render_state
                .renderer
                .write()
                .callback_resources
                .insert(resources);
            gpu_ready = true;
        }

        Self {
            config,
            cursor_pos: glam::Vec2::ZERO,
            last_viewport_size: glam::Vec2::ONE,
            gpu_ready,
        }
    }

    fn fit_image_to_screen(&mut self) {
        self.config.fit_to_viewport(self.last_viewport_size);
    }

    fn apply_scroll_zoom(&mut self, scroll_amount: f32, view_center: glam::Vec2) {
        let zoom_factor = 1.0 + scroll_amount * LINE_SCROLL_ZOOM_SCALE;
        let old_zoom = self.config.zoom;
        let new_zoom = (old_zoom * zoom_factor).clamp(0.01, 100.0);
        let actual_zoom_ratio = new_zoom / old_zoom;
        let mouse_offset = self.cursor_pos - view_center;
        self.config.pan = mouse_offset - (mouse_offset - self.config.pan) * actual_zoom_ratio;
        self.config.zoom = new_zoom;
    }

    fn scroll_amount_from_input(ui: &egui::Ui) -> f32 {
        ui.input(|input| {
            let mut amount = 0.0;
            for event in &input.events {
                if let egui::Event::MouseWheel { unit, delta, .. } = event {
                    amount += match unit {
                        egui::MouseWheelUnit::Line => delta.y * LINE_SCROLL_UNIT,
                        egui::MouseWheelUnit::Point | egui::MouseWheelUnit::Page => {
                            delta.y * POINT_TO_LINE_UNIT
                        }
                    };
                }
            }
            if amount != 0.0 {
                return amount.clamp(-MAX_SCROLL_AMOUNT_PER_FRAME, MAX_SCROLL_AMOUNT_PER_FRAME);
            }

            let smooth_y = input.smooth_scroll_delta.y;
            if smooth_y != 0.0 {
                return (smooth_y * POINT_TO_LINE_UNIT)
                    .clamp(-MAX_SCROLL_AMOUNT_PER_FRAME, MAX_SCROLL_AMOUNT_PER_FRAME);
            }
            0.0
        })
    }

    fn side_panel_width(viewport_width: f32) -> f32 {
        SIDE_PANEL_WIDTH
            .min(viewport_width * 0.45)
            .max(SIDE_PANEL_MIN_WIDTH.min(viewport_width.max(1.0)))
    }

    fn image_viewport(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let size = rect.size();
        if size.x < 1.0 || size.y < 1.0 {
            return;
        }

        self.last_viewport_size = glam::Vec2::new(size.x, size.y);

        let id = ui.id().with("image_viewport");
        let response = ui.interact(rect, id, egui::Sense::click_and_drag());

        if let Some(pos) = response.hover_pos() {
            self.cursor_pos = glam::Vec2::new(pos.x - rect.min.x, pos.y - rect.min.y);
        }

        let view_center = glam::Vec2::new(rect.width() * 0.5, rect.height() * 0.5);

        if response.hovered() {
            let scroll_amount = Self::scroll_amount_from_input(ui);
            if scroll_amount != 0.0 {
                self.apply_scroll_zoom(scroll_amount, view_center);
                ui.input_mut(|input| {
                    input.smooth_scroll_delta = egui::Vec2::ZERO;
                });
            }
        }

        if response.dragged() {
            let delta = response.drag_delta();
            self.config.pan.x += delta.x;
            self.config.pan.y += delta.y;
        }

        if self.gpu_ready {
            let viewport_size = (size.x.round() as u32, size.y.round() as u32);
            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                rect,
                ImagePaintCallback {
                    config: self.config,
                    viewport_size,
                },
            ));
        }
    }

    fn controls_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("View");
        ui.separator();
        if ui.button("Reset view").clicked() {
            self.fit_image_to_screen();
        }
        ui.separator();
        ui.label(format!("Zoom: {:.2}x", self.config.zoom));
        ui.add_space(8.0);
        ui.label("Drag to pan");
        ui.label("Scroll to zoom");
    }
}

impl eframe::App for PhotoEditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let viewport = ui.max_rect();

        if !self.gpu_ready {
            ui.colored_label(
                egui::Color32::YELLOW,
                "WebGPU is not available. Image view cannot be shown.",
            );
            return;
        }

        // Full-window image behind the controls.
        self.image_viewport(ui, viewport);

        let panel_w = Self::side_panel_width(viewport.width());
        if panel_w < 1.0 {
            return;
        }

        let panel_rect = egui::Rect::from_min_max(
            egui::pos2(viewport.max.x - panel_w, viewport.min.y),
            viewport.max,
        );

        ui.scope_builder(egui::UiBuilder::new().max_rect(panel_rect), |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_black_alpha(180))
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    self.controls_panel(ui);
                });
        });
    }
}
