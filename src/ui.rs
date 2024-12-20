pub struct UI {
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}
impl UI {
    pub fn new(
        window: &winit::window::Window,
        surface_format: wgpu::TextureFormat,
        device: &wgpu::Device,
    ) -> Self {
        let egui_context = egui::Context::default();
        let state = egui_winit::State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(2 * 1024), // default dimension is 2048
        );

        let renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 4, false);

        Self { state, renderer }
    }
    pub fn handle(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event);
        self.state.on_mouse_motion(delta)
    }
    pub fn render(
        &mut self,
        window: &winit::window::Window,
        view: &wgpu::TextureView,
        msaa_buffer: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let raw_input = self.state.take_egui_input(window);
        self.state.egui_ctx().begin_pass(raw_input);

        egui::TopBottomPanel::bottom("panel").show(self.state.egui_ctx(), |ui| {
            ui.label("lolitop");
            ui.label("v 0.1");

            let b1 = ui.button("Transparency");

            if b1.contains_pointer() {
                println!("Button hovered!");
            }
            if b1.clicked() {
                println!("Button clicked!");
            }
        });

        let full_output = self.state.egui_ctx().end_pass();
        self.state
            .handle_platform_output(&window, full_output.platform_output);
        let tris = self.state.egui_ctx().tessellate(full_output.shapes, 1.0);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&device, &queue, *id, image_delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: window.inner_size().into(),
            pixels_per_point: window.scale_factor() as f32,
        };
        self.renderer
            .update_buffers(&device, &queue, encoder, &tris, &screen_descriptor);
        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_buffer,
                    resolve_target: Some(&view),
                    ops: egui_wgpu::wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                label: Some("egui main render pass"),
                occlusion_query_set: None,
            });

            self.renderer
                .render(&mut rpass.forget_lifetime(), &tris, &screen_descriptor);
            for x in &full_output.textures_delta.free {
                self.renderer.free_texture(x)
            }
        }
    }
}
