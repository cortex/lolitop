use core::f32;
use std::time::{Duration, Instant};

use cgmath::Rotation3;
use wgpu::BufferDescriptor;

use crate::cpu::CPUMetrics;

pub struct SysMetrics {
    last_sample_time: Instant,
    pub cpu_usage_buffer: wgpu::Buffer,
    pub cpu_metrics: CPUMetrics,
}

impl SysMetrics {
    pub fn new(device: &wgpu::Device) -> Self {
        let last_sample_time = Instant::now();

        let cpu_usage_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("CPU usage"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: 16 * 4,
            mapped_at_creation: false,
        });

        let cpu_metrics = CPUMetrics::new();
        SysMetrics {
            last_sample_time,
            cpu_metrics,
            cpu_usage_buffer,
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
        let sample_rate: u64 = 200;
        let now = Instant::now();
        if now - self.last_sample_time > Duration::from_millis(sample_rate) {
            self.cpu_metrics.sample();
            self.last_sample_time = now;
        }
        queue.write_buffer(
            &self.cpu_usage_buffer,
            0,
            bytemuck::cast_slice(&self.cpu_metrics.interpolate_usage(
                now.duration_since(self.last_sample_time).as_millis() as f32 / (sample_rate as f32),
            )),
        );
    }
}

pub struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

const NUM_INSTANCES_PER_ROW: u32 = 4;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

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
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
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

pub fn instances() -> Vec<Instance> {
    (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 {
                    x: 2.0 * x as f32,
                    y: 0.0,
                    z: 2.0 * z as f32,
                } - INSTANCE_DISPLACEMENT;

                let rotation = cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                );
                Instance { position, rotation }
            })
        })
        .collect::<Vec<_>>()
}
