pub struct UIEventResponse {
    pub consumed: bool,
    pub repaint: bool,
    pub exit_requested: bool,
}

pub struct UIRenderOutput {
    pub transparency_toggled: bool,
    /// Set when the user started a primary-button drag on an empty part of the panel.
    pub start_window_drag: bool,
    pub sample_rate_hz: f32,
    pub command_buffers: Vec<wgpu::CommandBuffer>,
}

pub struct UI {
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    context: egui::Context,
    menu: bool,
}

impl UI {
    pub fn new(
        window: &winit::window::Window,
        surface_format: wgpu::TextureFormat,
        device: &wgpu::Device,
    ) -> Self {
        let context = egui::Context::default();
        let viewport_id = context.viewport_id();

        let state = egui_winit::State::new(
            context.clone(),
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            window.theme(),
            Some(2 * 1024),
        );

        let renderer = egui_wgpu::Renderer::new(
            device,
            surface_format,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                ..Default::default()
            },
        );

        Self {
            state,
            renderer,
            context,
            menu: true,
        }
    }

    pub fn handle(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
        is_fullscreen: &mut bool,
        is_transparent: &mut bool,
        sample_rate_hz: &mut f32,
    ) -> UIEventResponse {
        let response = self.state.on_window_event(window, event);
        let mut exit_requested = false;
        let keyboard_handled = if response.consumed {
            false
        } else {
            match event {
                winit::event::WindowEvent::KeyboardInput { event, .. }
                    if event.state == winit::event::ElementState::Pressed =>
                {
                    match &event.logical_key {
                        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) => {
                            exit_requested = true;
                            true
                        }
                        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space) => {
                            dbg!("space pressed");
                            true
                        }
                        winit::keyboard::Key::Character(character) => {
                            match character.to_lowercase().as_str() {
                                "f" => {
                                    if *is_fullscreen {
                                        window.set_fullscreen(None);
                                    } else {
                                        window.set_fullscreen(Some(
                                            winit::window::Fullscreen::Borderless(None),
                                        ));
                                    }
                                    *is_fullscreen = !*is_fullscreen;
                                    true
                                }
                                "t" => {
                                    self.toggle_transparent(is_transparent);
                                    true
                                }
                                "r" => {
                                    let sample_rates = [0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0];
                                    *sample_rate_hz = sample_rates
                                        .iter()
                                        .find(|&&rate| rate > *sample_rate_hz)
                                        .copied()
                                        .unwrap_or(sample_rates[0]);
                                    true
                                }
                                "m" => {
                                    self.toggle_menu();
                                    true
                                }
                                _ => false,
                            }
                        }
                        _ => false,
                    }
                }
                _ => false,
            }
        };

        UIEventResponse {
            consumed: response.consumed,
            repaint: response.repaint || keyboard_handled,
            exit_requested,
        }
    }

    pub fn toggle_menu(&mut self) {
        self.menu = !self.menu;
    }

    pub fn toggle_transparent(&mut self, is_transparent: &mut bool) {
        *is_transparent = !*is_transparent;
    }

    /// Tell egui the primary button was released.
    ///
    /// After `Window::drag_window()` the compositor grabs the pointer and winit never
    /// delivers the matching `MouseInput::Released`, so egui would otherwise think the
    /// button is still held and misinterpret the next press (e.g. on a slider).
    pub fn release_primary_button(&mut self) {
        let pos = self
            .context
            .input(|i| i.pointer.latest_pos())
            .unwrap_or_default();
        let modifiers = self.context.input(|i| i.modifiers);
        let input = self.state.egui_input_mut();
        input.events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers,
        });
        input.events.push(egui::Event::PointerGone);
    }

    pub fn render(
        &mut self,
        window: &winit::window::Window,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        fps: f64,
        sample_rate_hz: f32,
        is_transparent: bool,
        cpu_usages: &[f32],
    ) -> UIRenderOutput {
        let raw_input = self.state.take_egui_input(window);

        let mut out_sample_rate_hz = sample_rate_hz;
        let mut transparency_toggled = false;
        let mut start_window_drag = false;
        let menu_visible = self.menu;

        let full_output = self.context.run_ui(raw_input, |ui| {
            if menu_visible {
                egui::Panel::bottom("panel").show(ui, |ui| {
                    let drag_bg = ui.interact(
                        ui.max_rect(),
                        ui.scope_id().with("window_drag_bg"),
                        egui::Sense::drag(),
                    );
                    if drag_bg.drag_started_by(egui::PointerButton::Primary) {
                        start_window_drag = true;
                    }
                    ui.style_mut().interaction.selectable_labels = false;

                    ui.horizontal(|ui| {
                        ui.set_width(ui.available_width());
                        ui.label("lolitop");
                        ui.separator();
                        ui.label("v0.1");
                        ui.separator();
                        ui.label(format!("FPS: {:.0}", fps));
                        ui.separator();
                        ui.label(format!("m to toggle menu"));
                        ui.separator();
                        ui.label(if is_transparent {
                            "transparent"
                        } else {
                            "opaque"
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Sample rate:");
                        ui.add(
                            egui::Slider::new(&mut out_sample_rate_hz, 0.5..=60.0)
                                .suffix(" Hz")
                                .logarithmic(true),
                        );

                        if ui
                            .button(if is_transparent {
                                "Set opaque"
                            } else {
                                "Set transparent"
                            })
                            .clicked()
                        {
                            transparency_toggled = true;
                        }
                    });

                    egui::CollapsingHeader::new("CPU")
                        .default_open(false)
                        .show(ui, |ui| {
                            for (i, usage) in cpu_usages.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("CPU {i:>2}"));
                                    ui.add(
                                        egui::ProgressBar::new(usage.clamp(0.0, 1.0))
                                            .desired_width(120.0)
                                            .show_percentage(),
                                    );
                                });
                            }
                        });
                });
            }
        });

        let egui::FullOutput {
            platform_output,
            mut textures_delta,
            shapes,
            ..
        } = full_output;

        self.state.handle_platform_output(window, platform_output);

        let paint_jobs = self
            .context
            .tessellate(shapes, window.scale_factor() as f32);

        for (id, image_deltas) in &textures_delta.set {
            for image_delta in image_deltas {
                self.renderer
                    .update_texture(device, queue, *id, image_delta);
            }
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: window.inner_size().into(),
            pixels_per_point: window.scale_factor() as f32,
        };

        let command_buffers =
            self.renderer
                .update_buffers(device, queue, encoder, &paint_jobs, &screen_descriptor);

        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.renderer.render(
                &mut rpass.forget_lifetime(),
                &paint_jobs,
                &screen_descriptor,
            );
        }

        for x in &textures_delta.free {
            self.renderer.free_texture(x)
        }

        textures_delta.clear();

        UIRenderOutput {
            transparency_toggled,
            start_window_drag,
            sample_rate_hz: out_sample_rate_hz,
            command_buffers,
        }
    }
}
