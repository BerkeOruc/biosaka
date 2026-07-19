use crate::simulation::Simulation;
use rand::Rng;
use std::f32::consts::{PI, TAU};

const NUM_SEGMENTS: usize = 20;
const SPRING_K: f32 = 600.0;
const DAMPING: f32 = 12.0;
const REST_LENGTH: f32 = 0.028;
const DT: f32 = 0.006;
const SEGMENT_RADIUS: f32 = 0.014;
const MOTOR_GAIN: f32 = 0.0025;
const TURN_GAIN: f32 = 0.0003;
const FRICTION_ALONG: f32 = 1.5;
const FRICTION_PERP: f32 = 10.0;
const MAX_SPEED: f32 = 0.025;
const BOUNDARY: f32 = 0.025;
#[allow(dead_code)]
pub const VULVA_SEGMENT: usize = 10;

pub struct BodySegment {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub angle: f32,
    mass: f32,
}

pub struct Obstacle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub struct Worm {
    pub segments: Vec<BodySegment>,
    pub head_x: f32,
    pub head_y: f32,
    pub direction: f32,
    pub speed: f32,
    pub body_wave_phase: f32,
    pub obstacles: Vec<Obstacle>,
    pub obstacle_count: u32,
    pub sex: String,
}

impl Worm {
    pub fn new() -> Self {
        let head_x = 0.55;
        let head_y = 0.5;
        let mut segments: Vec<BodySegment> = (0..NUM_SEGMENTS)
            .map(|i| {
                let t = i as f32;
                BodySegment {
                    x: head_x - t * REST_LENGTH,
                    y: head_y,
                    vx: 0.0,
                    vy: 0.0,
                    angle: PI,
                    mass: 1.0,
                }
            })
            .collect();
        for i in 0..NUM_SEGMENTS.saturating_sub(1) {
            let dx = segments[i + 1].x - segments[i].x;
            let dy = segments[i + 1].y - segments[i].y;
            segments[i].angle = dy.atan2(dx);
        }
        if NUM_SEGMENTS > 1 {
            segments[NUM_SEGMENTS - 1].angle = segments[NUM_SEGMENTS - 2].angle;
        }

        Worm {
            segments,
            head_x,
            head_y,
            direction: PI,
            speed: 0.0,
            body_wave_phase: 0.0,
            obstacles: Vec::new(),
            obstacle_count: 0,
            sex: String::from("Hermaphrodite"),
        }
    }

    pub fn add_random_obstacle(&mut self) {
        let mut rng = rand::thread_rng();
        let w = 0.04 + rng.gen_range(0..8) as f32 * 0.01;
        let h = 0.04 + rng.gen_range(0..8) as f32 * 0.01;
        let cx = self.body_center_x();
        let cy = self.body_center_y();
        let angle = rng.gen::<f32>() * TAU;
        let dist = 0.08 + rng.gen::<f32>() * 0.12;
        let x = (cx + angle.cos() * dist - w * 0.5).clamp(0.02, 0.98 - w);
        let y = (cy + angle.sin() * dist - h * 0.5).clamp(0.02, 0.98 - h);
        self.obstacles.push(Obstacle { x, y, w, h });
        self.obstacle_count = self.obstacles.len() as u32;
    }

    pub fn update(&mut self, sim: &Simulation) {
        let (left_motor, right_motor) = self.get_motor_activity(sim);
        let motor_sum = left_motor + right_motor;
        let motor_asym = left_motor - right_motor;

        let wave_freq = 0.08 + motor_sum * 0.025;
        self.body_wave_phase += wave_freq;
        if self.body_wave_phase > 628.3 {
            self.body_wave_phase -= 628.3;
        }

        let mut fx = [0.0f32; NUM_SEGMENTS];
        let mut fy = [0.0f32; NUM_SEGMENTS];

        // Spring forces between adjacent segments
        for i in 0..NUM_SEGMENTS.saturating_sub(1) {
            let dx = self.segments[i + 1].x - self.segments[i].x;
            let dy = self.segments[i + 1].y - self.segments[i].y;
            let dist = (dx * dx + dy * dy).sqrt().max(1e-8);
            let stretch = dist - REST_LENGTH;
            let f = SPRING_K * stretch;
            let nx = dx / dist;
            let ny = dy / dist;
            let mi = self.segments[i].mass;
            let mj = self.segments[i + 1].mass;
            fx[i] += f * nx / mi;
            fy[i] += f * ny / mi;
            fx[i + 1] -= f * nx / mj;
            fy[i + 1] -= f * ny / mj;
        }

        // Velocity damping
        for i in 0..NUM_SEGMENTS {
            fx[i] -= DAMPING * self.segments[i].vx;
            fy[i] -= DAMPING * self.segments[i].vy;
        }

        // Motor-driven undulatory wave
        let amplitude = motor_sum * MOTOR_GAIN;
        let turn_bias = motor_asym * TURN_GAIN;
        for i in 0..NUM_SEGMENTS {
            let t = i as f32 / (NUM_SEGMENTS.saturating_sub(1).max(1)) as f32;
            let wave = (self.body_wave_phase - t * 4.5).sin();
            let lateral_force = wave * amplitude + turn_bias;
            let angle = self.segments[i].angle;
            let perp_x = -angle.sin();
            let perp_y = angle.cos();
            fx[i] += lateral_force * perp_x / self.segments[i].mass;
            fy[i] += lateral_force * perp_y / self.segments[i].mass;
        }

        // Anisotropic ground friction — high perpendicular drag converts lateral waves into thrust
        for i in 0..NUM_SEGMENTS {
            let angle = self.segments[i].angle;
            let along_x = angle.cos();
            let along_y = angle.sin();
            let perp_x = -angle.sin();
            let perp_y = angle.cos();
            let vx = self.segments[i].vx;
            let vy = self.segments[i].vy;
            let v_along = vx * along_x + vy * along_y;
            let v_perp = vx * perp_x + vy * perp_y;
            let fric_x = FRICTION_ALONG * v_along * along_x
                       + FRICTION_PERP * v_perp * perp_x;
            let fric_y = FRICTION_ALONG * v_along * along_y
                       + FRICTION_PERP * v_perp * perp_y;
            fx[i] -= fric_x / self.segments[i].mass;
            fy[i] -= fric_y / self.segments[i].mass;
        }

        // Semi-implicit Euler integration
        for i in 0..NUM_SEGMENTS {
            self.segments[i].vx += fx[i] * DT;
            self.segments[i].vy += fy[i] * DT;
            let spd = (self.segments[i].vx * self.segments[i].vx
                     + self.segments[i].vy * self.segments[i].vy)
                     .sqrt();
            if spd > MAX_SPEED {
                let scale = MAX_SPEED / spd;
                self.segments[i].vx *= scale;
                self.segments[i].vy *= scale;
            }
            self.segments[i].x += self.segments[i].vx * DT;
            self.segments[i].y += self.segments[i].vy * DT;
        }

        // Recompute angles from segment positions
        for i in 0..NUM_SEGMENTS.saturating_sub(1) {
            let dx = self.segments[i + 1].x - self.segments[i].x;
            let dy = self.segments[i + 1].y - self.segments[i].y;
            self.segments[i].angle = dy.atan2(dx);
        }
        if NUM_SEGMENTS > 1 {
            self.segments[NUM_SEGMENTS - 1].angle = self.segments[NUM_SEGMENTS - 2].angle;
        }

        // Obstacle collision — push segments out of rectangles
        for ob in &self.obstacles {
            for seg in &mut self.segments {
                let cx = seg.x.clamp(ob.x, ob.x + ob.w);
                let cy = seg.y.clamp(ob.y, ob.y + ob.h);
                let dx = seg.x - cx;
                let dy = seg.y - cy;
                let dist_sq = dx * dx + dy * dy;
                let radius = SEGMENT_RADIUS;
                if dist_sq < radius * radius {
                    let dist = dist_sq.sqrt().max(1e-8);
                    let overlap = radius - dist;
                    let nx = dx / dist;
                    let ny = dy / dist;
                    seg.x += overlap * nx;
                    seg.y += overlap * ny;
                    let v_dot_n = seg.vx * nx + seg.vy * ny;
                    if v_dot_n < 0.0 {
                        seg.vx -= 1.5 * v_dot_n * nx;
                        seg.vy -= 1.5 * v_dot_n * ny;
                    }
                }
            }
        }

        // Boundary clamping
        for seg in &mut self.segments {
            if seg.x < BOUNDARY {
                seg.x = BOUNDARY + (BOUNDARY - seg.x).max(0.0) * 0.5;
                seg.vx = seg.vx.abs().max(0.001);
            } else if seg.x > 1.0 - BOUNDARY {
                seg.x = 1.0 - BOUNDARY - (seg.x - (1.0 - BOUNDARY)).max(0.0) * 0.5;
                seg.vx = -seg.vx.abs().min(-0.001);
            }
            if seg.y < BOUNDARY {
                seg.y = BOUNDARY + (BOUNDARY - seg.y).max(0.0) * 0.5;
                seg.vy = seg.vy.abs().max(0.001);
            } else if seg.y > 1.0 - BOUNDARY {
                seg.y = 1.0 - BOUNDARY - (seg.y - (1.0 - BOUNDARY)).max(0.0) * 0.5;
                seg.vy = -seg.vy.abs().min(-0.001);
            }
        }

        self.head_x = self.segments[0].x;
        self.head_y = self.segments[0].y;
        let spd = (self.segments[0].vx * self.segments[0].vx
                 + self.segments[0].vy * self.segments[0].vy).sqrt();
        self.speed = spd;
        self.direction = self.segments[0].angle;
    }

    fn get_motor_activity(&self, sim: &Simulation) -> (f32, f32) {
        let mut left_act = 0.0f32;
        let mut right_act = 0.0f32;
        let mut left_cnt = 0u32;
        let mut right_cnt = 0u32;

        for neuron in &sim.neurons {
            let rate = neuron.firing_rate;
            match neuron.motor_role {
                1 => { left_act += rate; left_cnt += 1; }      // VB/AVB-L
                2 => { right_act += rate; right_cnt += 1; }     // VB/AVB-R
                3 => { left_act += rate * 0.5; left_cnt += 1; } // DB-L
                4 => { right_act += rate * 0.5; right_cnt += 1; } // DB-R
                5 => {                                            // VA/DA/VC bilateral
                    left_act += rate * 0.3;
                    right_act += rate * 0.3;
                }
                6 => {                                            // CP bilateral
                    left_act += rate * 0.5;
                    right_act += rate * 0.5;
                }
                7 => {                                            // HOB bilateral
                    left_act += rate * 0.2;
                    right_act += rate * 0.2;
                }
                _ => {}
            }
        }

        let left = if left_cnt > 0 { left_act / left_cnt as f32 } else { 0.0 };
        let right = if right_cnt > 0 { right_act / right_cnt as f32 } else { 0.0 };
        (left, right)
    }

    pub fn body_center_x(&self) -> f32 {
        self.segments.iter().map(|s| s.x).sum::<f32>() / self.segments.len() as f32
    }

    pub fn body_center_y(&self) -> f32 {
        self.segments.iter().map(|s| s.y).sum::<f32>() / self.segments.len() as f32
    }

    pub fn set_sex(&mut self, sex: &str) {
        self.sex = sex.to_string();
    }

    #[allow(dead_code)]
    pub fn has_tail_fan(&self) -> bool {
        self.sex == "Male"
    }
    #[allow(dead_code)]
    pub fn has_vulva(&self) -> bool {
        self.sex == "Hermaphrodite"
    }
}
