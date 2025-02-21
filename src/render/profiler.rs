use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Profiler {
    measurements: HashMap<String, Vec<Duration>>,
    start_times: HashMap<String, Instant>,
    frame_start: Instant,
    print_interval: Duration,
    last_print: Instant,
    enabled: bool,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
            start_times: HashMap::new(),
            frame_start: Instant::now(),
            print_interval: Duration::from_secs(1),
            last_print: Instant::now(),
            enabled: true
        }
    }
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
    pub fn start_frame(&mut self) {
        self.frame_start = Instant::now();
    }

    pub fn begin_scope(&mut self, name: &str) {
        self.start_times.insert(name.to_string(), Instant::now());
    }

    pub fn end_scope(&mut self, name: &str) {
        if let Some(start_time) = self.start_times.remove(name) {
            let duration = start_time.elapsed();
            self.measurements
                .entry(name.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }

    pub fn end_frame(&mut self) {
        let frame_time = self.frame_start.elapsed();
        self.measurements
            .entry("Total Frame".to_string())
            .or_insert_with(Vec::new)
            .push(frame_time);

        // Print stats every second
        if self.last_print.elapsed() >= self.print_interval {
            self.print_statistics();
            self.last_print = Instant::now();
            self.measurements.clear();
        }
    }

    fn print_statistics(&self) {
        if !self.enabled {return;}
        println!("\n=== Performance Profile ===");
        
        // Calculate and sort statistics
        let mut stats: Vec<(&String, f64, f64, f64)> = self.measurements
            .iter()
            .map(|(name, durations)| {
                let avg = durations.iter().sum::<Duration>().as_secs_f64() / durations.len() as f64;
                let max = durations.iter().max().unwrap().as_secs_f64();
                let min = durations.iter().min().unwrap().as_secs_f64();
                (name, avg, min, max)
            })
            .collect();

        // Sort by average time (descending)
        stats.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let t_stats = stats.clone();
        // Get total frame time for percentage calculation
        let total_time = t_stats.iter()
            .find(|(name, ..)| name.as_str() == "Total Frame")
            .map(|(_, avg, ..)| avg)
            .unwrap_or(&1.0);

        // Print statistics
        for (name, avg, min, max) in stats {
            let percentage = (avg / total_time) * 100.0;
            println!(
                "{:<20} Avg: {:.2}ms ({:.1}%) Min: {:.2}ms Max: {:.2}ms",
                name,
                avg * 1000.0,
                percentage,
                min * 1000.0,
                max * 1000.0
            );
        }
        println!("========================\n");
    }
}

pub struct ProfileScope<'a> {
    name: String,
    profiler: &'a mut Profiler,
}

impl<'a> ProfileScope<'a> {
    pub fn new(name: &str, profiler: &'a mut Profiler) -> Self {
        profiler.begin_scope(name);
        Self {
            name: name.to_string(),
            profiler,
        }
    }
}

impl<'a> Drop for ProfileScope<'a> {
    fn drop(&mut self) {
        self.profiler.end_scope(&self.name);
    }
}
