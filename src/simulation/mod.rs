#![allow(dead_code)]
use crate::connectome::Connectome;
use rand::Rng;

const RESET_POTENTIAL: f32 = 0.0;
const GABA_WEIGHT: f32 = -0.20;
const GAP_JUNCTION_STRENGTH: f32 = 0.05;

/// Tunable simulation parameters — can be adjusted live via the parameter panel.
#[derive(Debug, Clone, Copy)]
pub struct SimParams {
    pub leak_constant: f32,      // 0.90 – 0.99  (default 0.95)
    pub threshold: f32,          // 0.5  – 2.0   (default 1.0)
    pub noise_strength: f32,     // 0.0  – 0.1   (default 0.02)
    pub synaptic_multiplier: f32, // 0.5  – 3.0   (default 1.0)
}

impl Default for SimParams {
    fn default() -> Self {
        SimParams {
            leak_constant: 0.95,
            threshold: 1.0,
            noise_strength: 0.02,
            synaptic_multiplier: 1.0,
        }
    }
}

impl SimParams {
    pub fn labels() -> &'static [&'static str] {
        &["Leak", "Thresh", "Noise", "SynMul"]
    }

    pub fn count() -> usize { 4 }

    pub fn get(&self, idx: usize) -> f32 {
        match idx {
            0 => self.leak_constant,
            1 => self.threshold,
            2 => self.noise_strength,
            3 => self.synaptic_multiplier,
            _ => 0.0,
        }
    }

    pub fn set(&mut self, idx: usize, val: f32) {
        match idx {
            0 => self.leak_constant = val.clamp(0.90, 0.99),
            1 => self.threshold = val.clamp(0.5, 2.0),
            2 => self.noise_strength = val.clamp(0.0, 0.1),
            3 => self.synaptic_multiplier = val.clamp(0.5, 3.0),
            _ => {}
        }
    }

    pub fn range(&self, idx: usize) -> (f32, f32, f32) {
        match idx {
            0 => (0.90, 0.99, 0.01),
            1 => (0.5,  2.0,  0.1),
            2 => (0.0,  0.1,  0.005),
            3 => (0.5,  3.0,  0.1),
            _ => (0.0, 0.0, 0.0),
        }
    }
}

/// Neurotransmitter type for a neuron.
/// Determines how its chemical synapses affect postsynaptic neurons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Neurotransmitter {
    GABA,        // inhibitory — lowers postsynaptic potential
    Glutamate,   // excitatory — raises postsynaptic potential
    Acetylcholine, // excitatory — standard motor/inter-neuron transmitter
    Dopamine,    // modulatory (treated as weak excitatory for now)
    Serotonin,   // modulatory (treated as weak excitatory for now)
    Other,       // unknown / default (mild excitatory)
}

/// Infer neurotransmitter type from a C. elegans neuron name.
/// Based on published connectome literature (White 1986, Cook 2019, WormAtlas).
pub fn infer_neurotransmitter(name: &str) -> Neurotransmitter {
    // GABAergic neurons — well-characterised set from WormAtlas
    if name.starts_with("VD") || name.starts_with("DD") || name.starts_with("RME")
        || name == "AVL" || name == "DVB" || name == "RIS"
    {
        return Neurotransmitter::GABA;
    }

    // Dopaminergic
    if name.starts_with("CEP") || name == "ADE" || name.starts_with("ADE")
        || name == "PDE" || name.starts_with("PDE")
        || name == "PDP"
        || name.starts_with("CEM")
    {
        return Neurotransmitter::Dopamine;
    }

    // Serotonergic
    if name.starts_with("NSM") || name.starts_with("HSN") || name.starts_with("ADF")
    {
        return Neurotransmitter::Serotonin;
    }

    // Glutamatergic — most sensory neurons + many interneurons
    if name.starts_with("AS")   // ASE, ASH, ASI, ASJ, ASK, ASG, etc.
        || name.starts_with("AD") || name.starts_with("ADL")
        || name.starts_with("AW") || name.starts_with("AWA")
        || name.starts_with("AWB") || name.starts_with("AWC")
        || name.starts_with("FLP") || name.starts_with("IL")
        || name.starts_with("OL") || name.starts_with("CEP")
        || name.starts_with("AIY") || name.starts_with("AIZ")
        || name.starts_with("RIA") || name.starts_with("RIB")
        || name.starts_with("RIC") || name.starts_with("RIG")
        || name.starts_with("RIH")
        || name.starts_with("SA") || name.starts_with("SAA")
        || name.starts_with("SMB") || name.starts_with("SMD")
        || name.starts_with("RMD") || name.starts_with("RMG")
        || name.starts_with("SDQ")
        || name == "URY" || name.starts_with("URY")
    {
        return Neurotransmitter::Glutamate;
    }

    // Ray sensory neurons (male-specific)
    if name.starts_with("R1") || name.starts_with("R2")
        || name.starts_with("R3") || name.starts_with("R4")
        || name.starts_with("R5") || name.starts_with("R6")
        || name.starts_with("R7") || name.starts_with("R8")
        || name.starts_with("R9")
    {
        return Neurotransmitter::Glutamate;
    }

    // Cholinergic — most motor neurons + remainder of interneurons
    if name.starts_with("VA") || name.starts_with("VB")
        || name.starts_with("VC") || name.starts_with("DA")
        || name.starts_with("DB")
        || name.starts_with("AV")
        || name.starts_with("AI") && !name.starts_with("AIY") && !name.starts_with("AIZ")
        || name.starts_with("RIM") || name.starts_with("RIR")
        || name.starts_with("SIA") || name.starts_with("SIB")
        || name.starts_with("PV")
        || name.starts_with("RM")
        || name.starts_with("CA") || name.starts_with("SP")
        || name.starts_with("PCA") || name.starts_with("PCB")
    {
        return Neurotransmitter::Acetylcholine;
    }

    Neurotransmitter::Acetylcholine
}

pub struct NeuronState {
    pub id: u16,
    pub potential: f32,
    pub firing: bool,
    pub firing_rate: f32,
    pub spike_count: u64,
    pub name: String,
    pub neurotransmitter: Neurotransmitter,
}

pub struct Simulation {
    pub neurons: Vec<NeuronState>,
    pub time: f64,
    pub connectome: Connectome,
    pub total_spikes: u64,
    pub network_activity: f32,
    pub sensor_inputs: Vec<f32>,
    pub params: SimParams,
}

impl Neurotransmitter {
    pub fn label(&self) -> &'static str {
        match self {
            Neurotransmitter::GABA => "GABA",
            Neurotransmitter::Glutamate => "Glu",
            Neurotransmitter::Acetylcholine => "ACh",
            Neurotransmitter::Dopamine => "DA",
            Neurotransmitter::Serotonin => "5HT",
            Neurotransmitter::Other => "?",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Neurotransmitter::GABA => "inhibitory",
            Neurotransmitter::Glutamate => "excitatory",
            Neurotransmitter::Acetylcholine => "excitatory",
            Neurotransmitter::Dopamine => "modulatory",
            Neurotransmitter::Serotonin => "modulatory",
            Neurotransmitter::Other => "unknown",
        }
    }

    pub fn color_idx(&self) -> u8 {
        match self {
            Neurotransmitter::GABA => 0,         // red-ish
            Neurotransmitter::Glutamate => 1,     // green-ish
            Neurotransmitter::Acetylcholine => 2,  // cyan-ish
            Neurotransmitter::Dopamine => 3,       // yellow-ish
            Neurotransmitter::Serotonin => 4,      // magenta-ish
            Neurotransmitter::Other => 5,          // gray
        }
    }
}

impl Simulation {
    pub fn new(connectome: Connectome) -> Self {
        let mut rng = rand::thread_rng();
        let neurons: Vec<NeuronState> = (0..connectome.num_neurons())
            .map(|i| {
                let name = connectome.neuron_name(i).to_string();
                let nt = infer_neurotransmitter(&name);
                NeuronState {
                    id: i,
                    potential: rng.gen::<f32>() * 0.3,
                    firing: false,
                    firing_rate: 0.0,
                    spike_count: 0,
                    name,
                    neurotransmitter: nt,
                }
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
            params: SimParams::default(),
        }
    }

    pub fn step(&mut self) {
        let mut rng = rand::thread_rng();
        let n = self.neurons.len();
        let p = &self.params;

        let mut new_potentials = vec![0.0f32; n];

        for i in 0..n {
            let mut v = self.neurons[i].potential;

            v *= p.leak_constant;
            v += p.noise_strength * (rng.gen::<f32>() - 0.5) * 2.0;

            v += self.sensor_inputs[i];
            self.sensor_inputs[i] = 0.0;

            new_potentials[i] = v;
        }

        let syn_w = p.synaptic_multiplier * 0.15;
        for &(pre, post, weight) in self.connectome.get_chemical_edges() {
            let pre = pre as usize;
            let post = post as usize;
            if pre < n && post < n && self.neurons[pre].firing {
                let w = match self.neurons[pre].neurotransmitter {
                    Neurotransmitter::GABA => GABA_WEIGHT * weight as f32,
                    Neurotransmitter::Dopamine | Neurotransmitter::Serotonin => syn_w * 0.5 * weight as f32,
                    _ => syn_w * weight as f32,
                };
                new_potentials[post] += w;
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

        let threshold = p.threshold;
        let mut active_count = 0;
        for i in 0..n {
            if new_potentials[i] >= threshold {
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
            if name.starts_with("AS") || name.starts_with("AD")
                || name.starts_with("CEM")
                || name.starts_with("SA")
                || name.starts_with("R1") || name.starts_with("R2")
                || name.starts_with("R3") || name.starts_with("R4")
                || name.starts_with("R5") || name.starts_with("R6")
                || name.starts_with("R7") || name.starts_with("R8")
                || name.starts_with("R9")
            {
                self.sensor_inputs[i] += strength;
            }
        }
    }

    pub fn stimulate_by_name(&mut self, name: &str, strength: f32) -> bool {
        for i in 0..self.neurons.len() {
            if self.neurons[i].name == name {
                self.sensor_inputs[i] += strength;
                return true;
            }
        }
        false
    }

    pub fn stimulate_by_prefix(&mut self, prefix: &str, strength: f32) -> u32 {
        let mut count = 0;
        for i in 0..self.neurons.len() {
            if self.neurons[i].name.starts_with(prefix) {
                self.sensor_inputs[i] += strength;
                count += 1;
            }
        }
        count
    }
}
