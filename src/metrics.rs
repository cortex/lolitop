use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    time::{Duration, Instant},
};

use sysinfo::System;
use wgpu::util::DeviceExt;

pub struct CPUSample {
    cpu_id: String,
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
    guest: u64,
    guest_nice: u64,
    time: Instant,
}

impl CPUSample {
    fn total(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
            + self.guest
            + self.guest_nice
    }
    fn idle(&self) -> u64 {
        self.idle + self.iowait
    }
    fn usage(&self, other: &Self) -> f32 {
        let total = self.total();
        let other_total = other.total();
        let idle = self.idle();
        let other_idle = other.idle();
        let total_delta = total - other_total;
        let idle_delta = idle - other_idle;
        1.0 - (idle_delta as f32 / total_delta as f32)
    }
}

fn parse_row(row: &str) -> Option<CPUSample> {
    let words: Vec<&str> = row.split_whitespace().collect();
    // The line we are interested in is the one that starts with cpu
    if !words[0].starts_with("cpu") || words[0] == "cpu" {
        return None;
    }

    Some(CPUSample {
        cpu_id: words[0].to_string(),
        user: words[1].parse().ok()?,
        nice: words[2].parse().ok()?,
        system: words[3].parse().ok()?,
        idle: words[4].parse().ok()?,
        iowait: words[5].parse().ok()?,
        irq: words[6].parse().ok()?,
        softirq: words[7].parse().ok()?,
        steal: words[8].parse().ok()?,
        guest: words[9].parse().ok()?,
        guest_nice: words[10].parse().ok()?,
        time: Instant::now(),
    })
}

pub struct CPUMetrics {
    samples: HashMap<String, Vec<CPUSample>>,
}

impl CPUMetrics {
    fn sample(&mut self) {
        // parse row for each line in /proc/stat
        let file = File::open("/proc/stat").unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let sample = parse_row(&line);
            if let Some(sample) = sample {
                self.samples
                    .entry(sample.cpu_id.clone())
                    .or_insert_with(Vec::new)
                    .push(sample);
            }
        }
    }
    fn current_usage(&self) -> Vec<f32> {
        // for each cpu, calculate the usage as the delta between the last two samples
        let mut usage = Vec::new();
        for (_, samples) in &self.samples {
            if samples.len() < 2 {
                continue;
            }
            let last = samples.last().unwrap();
            let prev = &samples[samples.len() - 2];
            usage.push(last.usage(&prev));
        }
        usage
    }
}

pub struct SysMetrics {
    last_sample_time: Instant,
    current: Vec<f32>,
    target: Vec<f32>,
    pub cpu_usage_buffer: wgpu::Buffer,
    pub cpu_metrics: CPUMetrics,
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
        let cpu_metrics = CPUMetrics {
            samples: HashMap::new(),
        };
        SysMetrics {
            last_sample_time,
            current,
            target,
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
        let now = Instant::now();
        if now - self.last_sample_time > Duration::from_millis(50) {
            self.cpu_metrics.sample();
            self.last_sample_time = now;
        }

        queue.write_buffer(
            &self.cpu_usage_buffer,
            0,
            bytemuck::cast_slice(&self.cpu_metrics.current_usage()),
        );
    }
}
