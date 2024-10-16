use std::sync::Arc;

use glyphon::{
    fontdb, Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use wgpu::TextureFormat;

pub struct Text {
    pub text_buffer: Buffer,
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub cache: Cache,
    pub viewport: Viewport,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
}

impl Text {
    pub fn init_text(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) -> Self {
        //let mut font_system = FontSystem::new();
        let mut font_system = FontSystem::new_with_fonts(
            vec![fontdb::Source::Binary(Arc::new(include_bytes!(
                "../assets/Inter.ttc"
            )))]
            .into_iter(),
        );
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, TextureFormat::Rgba8UnormSrgb);
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(12.0, 12.0));

        text_buffer.set_size(&mut font_system, Some(width as f32), Some(height as f32));
        text_buffer.set_text(
            &mut font_system,
            "lolitop v 0.1",
            Attrs::new().family(Family::Name("Inter")),
            Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(&mut font_system, false);
        Self {
            text_buffer,
            font_system,
            swash_cache,
            cache,
            viewport,
            atlas,
            text_renderer,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text_buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::Name("Inter")),
            Shaping::Advanced,
        );
        self.text_buffer
            .shape_until_scroll(&mut self.font_system, false);
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        self.text_buffer.set_size(
            &mut self.font_system,
            Some(width as f32),
            Some(height as f32),
        );
        self.viewport.update(
            &queue,
            Resolution {
                width: width,
                height: height,
            },
        );
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
    ) {
        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [TextArea {
                    buffer: &self.text_buffer,
                    left: 0.0,
                    top: 0.0,
                    scale: 2.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 2000,
                        bottom: 2000,
                    },
                    default_color: Color::rgba(255, 255, 255, 255),
                    custom_glyphs: &[],
                }],
                &mut self.swash_cache,
            )
            .unwrap();
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.text_renderer
            .render(&self.atlas, &self.viewport, &mut pass)
            .unwrap();
        // self.atlas.trim();
    }
}
