use crate::cpu::CPUSampler;
use procfs::process::all_processes;
use std::collections::HashMap;
type Pid = u32;

struct ProcessSampler {
    processes: HashMap<Pid, ProcessStats>,
}

struct ProcessTree {
    pid: Pid,
    cpu: f32,
    comm: String,
    children: Vec<ProcessTree>,
}

struct ProcessSample {
    pid: Pid,
    utime: u64,
    stime: u64,
    comm: String,
}

struct ProcessStats {
    pid: u32,
    comm: String,
    cpu_percent_fast: f64,
    cpu_percent_slow: f64,
    prev_utime: u64,
    prev_stime: u64,
    prev_sample_time: u64,
}

impl ProcessStats {
    fn update(&mut self, sample: &ProcessSample, utime: u64, stime: u64) {
        let cpu_time = sample.utime + sample.stime;
        let prev_cpu_time = self.prev_utime + self.prev_stime;
        let cpu_percent = ((cpu_time - prev_cpu_time) as f64 / (utime + stime) as f64) * 100.0;
        self.cpu_percent_fast = cpu_percent;
        self.prev_utime = sample.utime;
        self.prev_stime = sample.stime;
    }
}

impl ProcessSampler {
    pub fn update(&mut self) {
        //Sample current global cpu ticks
        let mut cpu_sampler = CPUSampler::new();

        let stats_before = cpu_sampler.global();

        let processes: Vec<ProcessSample> = match all_processes() {
            Err(err) => {
                println!("Failed to read all processes: {}", err);
                return;
            }
            Ok(processes) => processes,
        }
        .filter_map(|v| {
            v.and_then(|p| {
                let stat = p.stat()?;
                let cmdline = p.cmdline().ok();
                Ok(ProcessSample {
                    pid: stat.pid.into()?,
                    utime: stat.utime,
                    stime: stat.stime,
                    comm: stat.comm,
                })
            })
            .ok()
        })
        .collect();
        let stats_after = cpu_sampler.global();

        let global_cpu_ticks_avg = stats_before + (stats_after - stats_before) / 2;

        for sample in &processes {
            self.processes
                .entry(sample.pid)
                .or_insert_with(|| ProcessStats {
                    pid: sample.pid,
                    comm: sample.comm.clone(),
                    cpu_percent_fast: 0.0,
                    cpu_percent_slow: 0.0,
                    prev_utime: 0,
                    prev_stime: 0,
                    prev_sample_time: 0,
                })
                .update(sample, sample.utime, sample.stime);
        }
    }
}
