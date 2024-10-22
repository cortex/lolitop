use core::f32;
use std::time::{Duration, Instant};

use cgmath::Rotation3;
use wgpu::util::DeviceExt;
use wgpu::BufferDescriptor;

use crate::cpu::CPUMetrics;

pub struct SysMetrics {
    last_sample_time: Instant,
    cpu_usage_buffer: wgpu::Buffer,
    pub cpu_metrics: CPUMetrics,
    pub cpu_core_instances: Vec<Instance>,
    pub sample_rate_hz: f32,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    instance_buffer: wgpu::Buffer,
}

use crate::model;

impl SysMetrics {
    pub fn new(device: &wgpu::Device) -> Self {
        let cpu_metrics = CPUMetrics::default();

        let last_sample_time = Instant::now();

        let cpu_usage_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("CPU usage"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: cpu_metrics.ncpus() as u64 * 4,
            mapped_at_creation: false,
        });

        let model = model::cube();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&model.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&model.vertex_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = model.vertex_indices.len() as u32;

        let ncpus = cpu_metrics.ncpus();
        let cpu_core_instances = SysMetrics::instances(ncpus as u64);

        let instance_data = cpu_core_instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        SysMetrics {
            last_sample_time,
            cpu_metrics,
            cpu_usage_buffer,
            cpu_core_instances: cpu_core_instances,
            sample_rate_hz: 2.0,
            vertex_buffer,
            index_buffer,
            num_indices,
            instance_buffer,
        }
    }
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<f32>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 9,
                format: wgpu::VertexFormat::Float32,
            }],
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        let sample_period_s = 1.0 / self.sample_rate_hz;
        let now = Instant::now();
        if now - self.last_sample_time > Duration::from_secs_f32(sample_period_s) {
            self.cpu_metrics.sample();
            self.last_sample_time = now;
        }
        queue.write_buffer(
            &self.cpu_usage_buffer,
            0,
            bytemuck::cast_slice(&self.cpu_metrics.interpolate_usage(
                now.duration_since(self.last_sample_time).as_secs_f32() / sample_period_s,
            )),
        );
    }

    fn instances(n_cpus: u64) -> Vec<Instance> {
        static SPACING: f32 = 2.0;

        let per_row = (n_cpus as f32).sqrt().ceil() as u64;
        let displacement: cgmath::Vector3<f32> =
            cgmath::Vector3::new((per_row - 1) as f32, 0., (per_row - 1) as f32) * SPACING / 2.;
        (0..n_cpus)
            .map(|i| {
                let x = i % per_row;
                let z = i / per_row;
                let position = cgmath::Vector3 {
                    x: SPACING * x as f32,
                    y: 0.0,
                    z: SPACING * z as f32,
                } - displacement;

                let rotation = cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                );
                Instance { position, rotation }
            })
            .take(n_cpus as usize)
            .collect()
    }

    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass,
        render_pipeline: &wgpu::RenderPipeline,
        light_bind_group: &wgpu::BindGroup,
        camera_controller: &crate::camera::CameraController,
    ) {
        render_pass.set_pipeline(&render_pipeline);
        render_pass.set_bind_group(0, &camera_controller.camera().bind_group, &[]);
        render_pass.set_bind_group(1, &light_bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(2, self.cpu_usage_buffer.slice(..));
        render_pass.draw_indexed(
            0..self.num_indices,
            0,
            0..self.cpu_core_instances.len() as _,
        );
    }
}

pub struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
