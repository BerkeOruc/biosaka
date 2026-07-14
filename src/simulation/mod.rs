#![allow(dead_code)]
use crate::connectome::Connectome;
use rand::Rng;

const LEAK_CONSTANT: f32 = 0.95;
const THRESHOLD: f32 = 1.0;
const RESET_POTENTIAL: f32 = 0.0;
const SYNAPTIC_WEIGHT: f32 = 0.15;
const GAP_JUNCTION_STRENGTH: f32 = 0.05;
const NOISE_STRENGTH: f32 = 0.02;

pub struct NeuronState {
    pub id: u16,
    pub potential: f32,
    pub firing: bool,
    pub firing_rate: f32,
    pub spike_count: u64,
    pub name: String,
}

pub struct Simulation {
    pub neurons: Vec<NeuronState>,
    pub time: f64,
    pub connectome: Connectome,
    pub total_spikes: u64,
    pub network_activity: f32,
    pub sensor_inputs: Vec<f32>,
}

impl Simulation {
    pub fn new(connectome: Connectome) -> Self {
        let mut rng = rand::thread_rng();
        let neurons: Vec<NeuronState> = (0..connectome.num_neurons())
            .map(|i| NeuronState {
                id: i,
                potential: rng.gen::<f32>() * 0.3,
                firing: false,
                firing_rate: 0.0,
                spike_count: 0,
                name: connectome.neuron_name(i).to_string(),
            })
            .collect();

        let sensor_inputs = vec![0.0; connectome.num_neurons() as usize];

        Simulation {
            neurons,
            time: 0.0,
            connectome,
            total_spikes: 0,
            network_activity: 0.0,
            sensor_inputs,
        }
    }

    pub fn step(&mut self) {
        let mut rng = rand::thread_rng();
        let n = self.neurons.len();

        let mut new_potentials = vec![0.0f32; n];

        for i in 0..n {
            let mut v = self.neurons[i].potential;

            v *= LEAK_CONSTANT;
            v += NOISE_STRENGTH * (rng.gen::<f32>() - 0.5) * 2.0;

            v += self.sensor_inputs[i];
            self.sensor_inputs[i] = 0.0;

            new_potentials[i] = v;
        }

        for &(pre, post, weight) in self.connectome.get_chemical_edges() {
            let pre = pre as usize;
            let post = post as usize;
            if pre < n && post < n && self.neurons[pre].firing {
                new_potentials[post] += SYNAPTIC_WEIGHT * weight as f32;
            }
        }

        for &(a, b, weight) in self.connectome.get_gap_junction_edges() {
            let a = a as usize;
            let b = b as usize;
            if a < n && b < n {
                let diff = self.neurons[a].potential - self.neurons[b].potential;
                let coupling = GAP_JUNCTION_STRENGTH * weight as f32 * diff;
                new_potentials[a] -= coupling;
                new_potentials[b] += coupling;
            }
        }

        let mut active_count = 0;
        for i in 0..n {
            if new_potentials[i] >= THRESHOLD {
                self.neurons[i].potential = RESET_POTENTIAL;
                self.neurons[i].firing = true;
                self.neurons[i].spike_count += 1;
                self.total_spikes += 1;
                active_count += 1;
            } else {
                self.neurons[i].potential = new_potentials[i];
                self.neurons[i].firing = false;
            }

            self.neurons[i].firing_rate = self.neurons[i].firing_rate * 0.99
                + if self.neurons[i].firing { 0.01 } else { 0.0 };
        }

        self.network_activity = active_count as f32 / n as f32;
        self.time += 1.0;
    }

    #[allow(dead_code)]
    pub fn get_active_neurons(&self) -> Vec<u16> {
        self.neurons
            .iter()
            .filter(|n| n.firing)
            .map(|n| n.id)
            .collect()
    }

    pub fn get_top_firing(&self, count: usize) -> Vec<(u16, f32)> {
        let mut rates: Vec<(u16, f32)> = self
            .neurons
            .iter()
            .map(|n| (n.id, n.firing_rate))
            .collect();
        rates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        rates.truncate(count);
        rates
    }

    pub fn stimulate_neuron(&mut self, id: u16, strength: f32) {
        if (id as usize) < self.neurons.len() {
            self.sensor_inputs[id as usize] += strength;
        }
    }

    pub fn stimulate_sensory_neurons(&mut self, strength: f32) {
        for i in 0..self.neurons.len() {
            let name = &self.neurons[i].name;
            if name.starts_with("AS") || name.starts_with("AD") {
                self.sensor_inputs[i] += strength;
            }
        }
    }
}
