use eframe::egui_wgpu::{self, wgpu};

use super::resources::{Config, ImageRenderResources};

pub struct ImagePaintCallback {
    pub config: Config,
    pub viewport_size: (u32, u32),
}

impl egui_wgpu::CallbackTrait for ImagePaintCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &mut ImageRenderResources = callback_resources
            .get_mut::<ImageRenderResources>()
            .expect("ImageRenderResources missing from callback_resources");
        resources.prepare(device, queue, encoder, self.config, self.viewport_size);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &ImageRenderResources = callback_resources
            .get::<ImageRenderResources>()
            .expect("ImageRenderResources missing from callback_resources");
        resources.paint(render_pass);
    }
}
