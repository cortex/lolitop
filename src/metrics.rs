use std::time::{Duration, Instant};

use sysinfo::System;
use wgpu::util::DeviceExt;

pub struct SysMetrics {
    sys: System,
    last_sample_time: Instant,
    current: Vec<f32>,
    target: Vec<f32>,
    pub cpu_usage_buffer: wgpu::Buffer,
}

impl SysMetrics {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut sys = System::new();
        sys.refresh_cpu(); // Refreshing CPU information.
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

        dbg!(sys.cpus().len());
        sys.refresh_cpu(); // Refreshing CPU information.
        let last_sample_time = Instant::now();
        let current: Vec<f32> = sys.cpus().into_iter().map(|f| 0.0).collect();
        let target: Vec<f32> = sys.cpus().into_iter().map(|f| f.cpu_usage()).collect();
        let cpu_usage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("CPU usage"),
            contents: bytemuck::cast_slice(&current),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        SysMetrics {
            sys,
            last_sample_time,
            current,
            target,
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
        let now = Instant::now();
        if now - self.last_sample_time > sysinfo::MINIMUM_CPU_UPDATE_INTERVAL {
            //if now - self.last_sample_time > Duration::from_millis(2000) {
            self.sys.refresh_cpu(); // Refreshing CPU information.
            self.target = self
                .sys
                .cpus()
                .into_iter()
                .map(|f| f.cpu_usage() / 100.0)
                .collect();
            self.last_sample_time = now;
        }
        // The length of the interpolation in milliseconds
        let interpolation_ns = Duration::from_millis(150);

        let progress = ((now - self.last_sample_time).as_nanos() as f32
            / interpolation_ns.as_nanos() as f32)
            .min(1.0);
        // dbg!(progress);
        self.current = self
            .current
            .iter()
            .zip(self.target.iter())
            .map(|(current, target)| current + (progress * (target - current)))
            .collect();

        //dbg!(&cpu_usage);
        queue.write_buffer(
            &self.cpu_usage_buffer,
            0,
            bytemuck::cast_slice(&self.current),
        )
    }
}
