use std::{
    collections::{BTreeMap, VecDeque},
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
struct CPUSample {
    cpu_id: u32,
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
    // We want the per-core lines ("cpu0", "cpu1", ...), not the aggregate "cpu" line.
    let cpu_id = words.first()?.strip_prefix("cpu")?.parse().ok()?;

    Some(CPUSample {
        cpu_id,
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
    })
}

/// Interpolation needs the usage over the last two intervals, i.e. three samples.
const SAMPLES_KEPT: usize = 3;

pub struct CPUMetrics {
    /// Most recent samples per core, keyed by core number so iteration is in core order.
    samples: BTreeMap<u32, VecDeque<CPUSample>>,
}

impl CPUMetrics {
    pub fn ncpus(&self) -> usize {
        self.samples.len()
    }

    pub fn sample(&mut self) {
        // parse row for each line in /proc/stat
        let file = File::open("/proc/stat").unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            let sample = parse_row(&line);
            if let Some(sample) = sample {
                let history = self.samples.entry(sample.cpu_id).or_default();
                if history.len() == SAMPLES_KEPT {
                    history.pop_front();
                }
                history.push_back(sample);
            }
        }
    }

    pub fn interpolate_usage(&self, delta: f32) -> Vec<f32> {
        self.samples
            .values()
            .filter_map(|samples| {
                let (pprev, prev, last) = (samples.front()?, samples.get(1)?, samples.get(2)?);

                let last_usage = last.usage(prev);
                let prev_usage = prev.usage(pprev);
                Some(prev_usage + (last_usage - prev_usage) * delta)
            })
            .collect()
    }
}

impl Default for CPUMetrics {
    fn default() -> Self {
        let mut s = CPUMetrics {
            samples: BTreeMap::new(),
        };
        s.sample();
        s
    }
}
