use std::iter;
use std::time::Instant;

use cgmath::Rotation3;
use wgpu::util::DeviceExt;
use winit::keyboard::NamedKey;
use winit::window::Window;
use winit::{event::*, keyboard::Key};

use crate::camera::CameraController;
use crate::metrics::InstanceRaw;
use crate::{camera::Camera, metrics::SysMetrics};
use crate::{model, text};

use crate::light::LightUniform;

use std::sync::Arc;

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    window: Arc<Window>,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    depth_buffer: wgpu::Texture,
    msaa_buffer: wgpu::TextureView,

    // Light
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,

    // Camera
    camera_controller: CameraController,
    sys_metrics: SysMetrics,

    // Text
    main_text: text::Text,

    last_frame: Instant,
    is_fullscreen: bool,
    is_transparent: bool,
}

impl<'a> State<'a> {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let window = Arc::new(window);
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("no suitable adapter found");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            desired_maximum_frame_latency: 2,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let sys_metrics = SysMetrics::new(&device);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let camera_controller = CameraController::new(Camera::new(
            &device,
            config.width as f32,
            config.height as f32,
        ));

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_uniform = LightUniform {
            position: [1.5, 1.5, 1.5],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_controller.camera().bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            cache: None,
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    model::Vertex::desc(),
                    InstanceRaw::desc(),
                    SysMetrics::desc(),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        // We'll want to update our lights position, so we use COPY_DST
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let depth_buffer = Self::depth_buffer(&device, &config);
        let msaa_buffer = Self::msaa_buffer(&device, &config, 4);

        let main_text = text::Text::init_text(&device, &queue, surface_format, size.width, size.height);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,

            camera_controller,
            sys_metrics,
            window,
            last_frame: Instant::now(),
            light_uniform,
            light_buffer,
            light_bind_group,
            depth_buffer,
            msaa_buffer,
            main_text,
            is_fullscreen: false,
            is_transparent: false,
        }
    }

    fn depth_buffer(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::Texture {
        let depth_buffer_size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size: depth_buffer_size,
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        device.create_texture(&desc)
    }

    fn msaa_buffer(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> wgpu::TextureView {
        let multisampled_texture_extent = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
            size: multisampled_texture_extent,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            //    format: config.view_formats[0],
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        };

        device
            .create_texture(multisampled_frame_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn window(&mut self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera_controller
                .camera_mut()
                .resize(new_size.width as f32, new_size.height as f32);

            self.depth_buffer = Self::depth_buffer(&self.device, &self.config);
            self.msaa_buffer = Self::msaa_buffer(&self.device, &self.config, 4);

            self.main_text
                .resize(&self.queue, new_size.width, new_size.height);
            self.window.request_redraw();
        }
    }

    fn toggle_fullscreen(&mut self) {
        if self.is_fullscreen {
            self.window.set_fullscreen(None);
        } else {
            self.window
                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }
        self.is_fullscreen = !self.is_fullscreen;
    }

    fn toggle_transparent(&mut self) {
        if self.is_transparent {
            self.window.set_transparent(false);
            self.config.alpha_mode = wgpu::CompositeAlphaMode::Opaque;
        } else {
            self.window.set_transparent(true);
            self.config.alpha_mode = wgpu::CompositeAlphaMode::PreMultiplied;
        }
        self.is_transparent = !self.is_transparent;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event);
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Space),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                dbg!("space pressed");
                true
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key: Key::Character(c),
                        ..
                    },
                ..
            } => match c.to_lowercase().as_str() {
                "f" => {
                    self.toggle_fullscreen();
                    true
                }
                "t" => {
                    self.toggle_transparent();
                    false
                }
                "r" => {
                    // cycle through the available sample rates
                    let sample_rates = [0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0];
                    let current_rate = self.sys_metrics.sample_rate_hz;
                    let new_rate = sample_rates
                        .iter()
                        .find(|&&r| r > current_rate)
                        .unwrap_or(&sample_rates[0]);
                    self.sys_metrics.sample_rate_hz = *new_rate;
                    true
                }
                _ => false,
            },

            _ => false,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_frame;
        self.last_frame = now;

        self.camera_controller.update(dt, &mut self.queue);
        self.sys_metrics.update(&self.queue);

        // Update the light
        let old_position: cgmath::Vector3<_> = self.light_uniform.position.into();
        self.light_uniform.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                * old_position)
                .into();

        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );

        self.main_text.set_text(
            &[
                "lolitop v0.1",
                format!("FPS: {:.2}", 1.0 / dt.as_secs_f64()).as_str(),
                format!("Sample rate: {}hz", self.sys_metrics.sample_rate_hz).as_str(),
            ]
            .join("\n"),
        );
        self.window.request_redraw();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa_buffer,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self
                        .depth_buffer
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.sys_metrics.render(
                &mut render_pass,
                &self.render_pipeline,
                &self.light_bind_group,
                &self.camera_controller,
            );
        }
        self.main_text
            .render(&self.device, &view, &mut encoder, &self.queue);

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
