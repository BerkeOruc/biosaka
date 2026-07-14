#![allow(dead_code)]
use crate::simulation::Simulation;

const NUM_SEGMENTS: usize = 20;

pub struct BodySegment {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
}


pub struct Worm {
    pub segments: Vec<BodySegment>,
    pub head_x: f32,
    pub head_y: f32,
    pub direction: f32,
    pub speed: f32,
    pub body_wave_phase: f32,
}

impl Worm {
    pub fn new() -> Self {
        let segments: Vec<BodySegment> = (0..NUM_SEGMENTS)
            .map(|i| {
                let t = i as f32 / (NUM_SEGMENTS - 1) as f32;
                BodySegment {
                    x: 0.5 - t * 0.4,
                    y: 0.5,
                    angle: 0.0,
                }
            })
            .collect();

        Worm {
            segments,
            head_x: 0.5,
            head_y: 0.5,
            direction: 0.0,
            speed: 0.0,
            body_wave_phase: 0.0,
        }
    }

    pub fn update(&mut self, sim: &Simulation) {
        let motor_activity = self.get_motor_activity(sim);

        let left_motor = motor_activity.0;
        let right_motor = motor_activity.1;

        let wave_freq = 0.08 + (left_motor + right_motor) * 0.02;
        self.body_wave_phase += wave_freq;

        self.speed = 0.003 + (left_motor + right_motor) * 0.008;

        let wave_amplitude = 0.06 + (left_motor - right_motor).abs() * 0.03;

        let mut new_segments: Vec<BodySegment> = (0..NUM_SEGMENTS)
            .map(|i| {
                let t = i as f32 / (NUM_SEGMENTS - 1) as f32;
                let wave = (self.body_wave_phase - t * 4.0).sin() * wave_amplitude * t;
                let offset_x = t * 0.4;
                let perp_y = wave;
                BodySegment {
                    x: self.head_x - offset_x * self.direction.cos()
                        - perp_y * self.direction.sin(),
                    y: self.head_y - offset_x * self.direction.sin()
                        + perp_y * self.direction.cos(),
                    angle: wave,
                }
            })
            .collect();

        let dx = self.speed * self.direction.cos();
        let dy = self.speed * self.direction.sin();
        for seg in &mut new_segments {
            seg.x += dx;
            seg.y += dy;
        }

        self.head_x += dx;
        self.head_y += dy;

        self.segments = new_segments;
    }

    fn get_motor_activity(&self, sim: &Simulation) -> (f32, f32) {
        let mut left_activity = 0.0f32;
        let mut right_activity = 0.0f32;
        let mut left_count = 0u32;
        let mut right_count = 0u32;

        for neuron in &sim.neurons {
            let name = &neuron.name;
            let rate = neuron.firing_rate;

            if name.starts_with("VB") || name == "AVBL" || name == "AVBR" {
                if name.ends_with('L') {
                    left_activity += rate;
                    left_count += 1;
                } else if name.ends_with('R') {
                    right_activity += rate;
                    right_count += 1;
                }
            }

            if name.starts_with("DB") {
                if name.ends_with('L') {
                    left_activity += rate * 0.5;
                    left_count += 1;
                } else if name.ends_with('R') {
                    right_activity += rate * 0.5;
                    right_count += 1;
                }
            }

            if name.starts_with("VA") || name.starts_with("DA") {
                left_activity += rate * 0.3;
                right_activity += rate * 0.3;
            }
        }

        let left = if left_count > 0 {
            left_activity / left_count as f32
        } else {
            0.0
        };
        let right = if right_count > 0 {
            right_activity / right_count as f32
        } else {
            0.0
        };

        (left, right)
    }

    pub fn body_center_x(&self) -> f32 {
        self.segments.iter().map(|s| s.x).sum::<f32>() / self.segments.len() as f32
    }

    pub fn body_center_y(&self) -> f32 {
        self.segments.iter().map(|s| s.y).sum::<f32>() / self.segments.len() as f32
    }
}
