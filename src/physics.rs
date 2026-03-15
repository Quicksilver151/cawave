use crate::config::Config;

pub struct WaveState {
    pub points: Vec<f32>,
    velocity: Vec<f32>,
    prev_frame: Vec<f32>,
    n: usize,
    driver_bars: usize,
    wave_speed: f32,
    damping: f32,
    wall_damping: f32,
    impulse_threshold: f32,
    impulse_strength: f32,
    impulse_spread: f32,
    min_rms: f32,
    compressibility: f32,
    reverse_frequencies: bool,
    mirror: bool,
}

impl WaveState {
    pub fn new(cfg: &Config) -> Self {
        Self {
            points: vec![0.0; cfg.physics.points],
            velocity: vec![0.0; cfg.physics.points],
            prev_frame: vec![0.0; cfg.input.driver_bars],
            n: cfg.physics.points,
            driver_bars: cfg.input.driver_bars,
            wave_speed: cfg.physics.wave_speed,
            damping: cfg.physics.damping,
            wall_damping: cfg.physics.wall_damping,
            impulse_threshold: cfg.physics.impulse_threshold,
            impulse_strength: cfg.physics.impulse_strength,
            impulse_spread: cfg.physics.impulse_spread,
            min_rms: cfg.physics.min_rms,
            compressibility: cfg.physics.compressibility,
            reverse_frequencies: cfg.input.reverse_frequencies,
            mirror: cfg.output.mirror,
        }
    }

    pub fn update(&mut self, frame: &[f32]) {
        let rms = (frame.iter().map(|x| x * x).sum::<f32>() / frame.len() as f32).sqrt();
        if rms >= self.min_rms {
            self.inject_impulses(frame);
        }
        self.propagate();
        self.apply_damping();
        self.restore_mean();
        self.prev_frame.copy_from_slice(frame);
    }

    fn inject_at(&mut self, center: usize, impulse: f32) {
        let n = self.n;
        let spread = self.impulse_spread;
        let radius = (3.0 * spread) as usize;
        let outer_radius = (5.0 * spread) as usize;

        let inner_lo = center.saturating_sub(radius);
        let inner_hi = (center + radius).min(n - 1);

        // Normalise inner gaussian
        let inner_sum: f32 = (inner_lo..=inner_hi)
            .map(|j| { let d = (j as f32 - center as f32) / spread; (-0.5*d*d).exp() })
            .sum();

        // Apply downward velocity to inner region
        for j in inner_lo..=inner_hi {
            let d = (j as f32 - center as f32) / spread;
            let w = (-0.5 * d * d).exp() / inner_sum.max(1e-6);
            self.velocity[j] -= impulse * w;
        }

        // Apply upward velocity to outer ring (annulus just outside inner region)
        let outer_lo = center.saturating_sub(outer_radius);
        let outer_hi = (center + outer_radius).min(n - 1);

        let mut outer_sum = 0.0f32;
        for j in outer_lo..=outer_hi {
            let d = (j as f32 - center as f32) / spread;
            let inner_w = (-0.5 * d * d).exp();
            // Outer ring = broad gaussian minus inner gaussian
            let outer_w = ((-0.5 * (d / 1.8) * (d / 1.8)).exp() - inner_w).max(0.0);
            outer_sum += outer_w;
        }

        for j in outer_lo..=outer_hi {
            let d = (j as f32 - center as f32) / spread;
            let inner_w = (-0.5 * d * d).exp();
            let outer_w = ((-0.5 * (d / 1.8) * (d / 1.8)).exp() - inner_w).max(0.0);
            if outer_sum > 1e-6 {
                self.velocity[j] += impulse * (outer_w / outer_sum);
            }
        }
    }

    fn inject_impulses(&mut self, frame: &[f32]) {
        let n = self.n;
        let frame_vec: Vec<f32> = if self.reverse_frequencies {
            frame.iter().cloned().rev().collect()
        } else {
            frame.to_vec()
        };

        // Collect impulses first to avoid borrow conflict with self.prev_frame
        let impulses: Vec<(usize, f32)> = frame_vec.iter()
            .zip(self.prev_frame.iter())
            .enumerate()
            .filter_map(|(i, (&current, &prev))| {
                let delta = current - prev;
                if delta > self.impulse_threshold {
                    Some((i, (delta * self.impulse_strength).min(1.0)))
                } else {
                    None
                }
            })
            .collect();

        for (i, impulse) in impulses {
            if self.mirror {
                let center_left = (i * (n / 2)) / self.driver_bars;
                let center_right = n - 1 - center_left;
                self.inject_at(center_left, impulse);
                self.inject_at(center_right, impulse);
            } else {
                let center = (i * n) / self.driver_bars;
                self.inject_at(center, impulse);
            }
        }
    }

    fn propagate(&mut self) {
        let k = self.wave_speed;
        let n = self.n;
        let p = self.points.clone();

        for i in 0..n {
            let left  = if i == 0     { -p[0]     * self.wall_damping } else { p[i - 1] };
            let right = if i == n - 1 { -p[n - 1] * self.wall_damping } else { p[i + 1] };
            self.velocity[i] += k * (left + right - 2.0 * p[i]);
        }
        for i in 0..n {
            self.points[i] = (self.points[i] + self.velocity[i]).clamp(-100.0, 100.0);
        }
    }

    fn apply_damping(&mut self) {
        for v in self.velocity.iter_mut() {
            *v *= self.damping;
        }
    }

    fn restore_mean(&mut self) {
        let mean = self.points.iter().sum::<f32>() / self.n as f32;
        for p in self.points.iter_mut() {
            *p -= mean * self.compressibility;
        }
    }
}
