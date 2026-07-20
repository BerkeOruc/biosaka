use crate::connectome::Connectome;
use rand::Rng;

const RESET_POTENTIAL: f32 = 0.0;
const GABA_WEIGHT: f32 = -0.20;
const GAP_JUNCTION_STRENGTH: f32 = 0.05;

/// Box-Muller transform: generate a standard normal N(0,1) sample from two uniform(0,1)
fn gaussian_sample(rng: &mut impl Rng) -> f32 {
    let u: f32 = rng.gen_range(1e-8..1.0);
    let v: f32 = rng.gen_range(1e-8..1.0);
    (-2.0 * u.ln()).sqrt() * (std::f32::consts::TAU * v).cos()
}

/// Tunable simulation parameters — can be adjusted live via the parameter panel.
#[derive(Debug, Clone, Copy)]
pub struct SimParams {
    pub leak_constant: f32,       // 0.90 – 0.99  (default 0.95)
    pub threshold: f32,           // 0.5  – 2.0   (default 1.0)
    pub noise_strength: f32,      // 0.0  – 0.1   (default 0.02)
    pub synaptic_multiplier: f32, // 0.5  – 3.0   (default 1.0)
    pub refractory_period: f32,   // 0  – 20 ticks  (default 2.0)
}

impl Default for SimParams {
    fn default() -> Self {
        SimParams {
            leak_constant: 0.95,
            threshold: 1.0,
            noise_strength: 0.02,
            synaptic_multiplier: 1.0,
            refractory_period: 2.0,
        }
    }
}

impl SimParams {
    pub const COUNT: usize = 5;

    pub fn labels() -> &'static [&'static str] {
        &["Leak", "Thresh", "Noise", "SynMul", "Refrct"]
    }

    pub fn count() -> usize { Self::COUNT }

    pub fn get(&self, idx: usize) -> f32 {
        match idx {
            0 => self.leak_constant,
            1 => self.threshold,
            2 => self.noise_strength,
            3 => self.synaptic_multiplier,
            4 => self.refractory_period,
            _ => 0.0,
        }
    }

    pub fn set(&mut self, idx: usize, val: f32) {
        match idx {
            0 => self.leak_constant = val.clamp(0.90, 0.99),
            1 => self.threshold = val.clamp(0.5, 2.0),
            2 => self.noise_strength = val.clamp(0.0, 0.1),
            3 => self.synaptic_multiplier = val.clamp(0.5, 3.0),
            4 => self.refractory_period = val.clamp(0.0, 20.0).round(),
            _ => {}
        }
    }

    pub fn range(&self, idx: usize) -> (f32, f32, f32) {
        match idx {
            0 => (0.90, 0.99, 0.01),
            1 => (0.5,  2.0,  0.1),
            2 => (0.0,  0.1,  0.005),
            3 => (0.5,  3.0,  0.1),
            4 => (0.0, 20.0, 1.0),
            _ => (0.0, 0.0, 0.0),
        }
    }

    pub fn refractory_ticks(&self) -> u32 {
        self.refractory_period as u32
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
    if name.starts_with("CEP") || name.starts_with("ADE")
        || name.starts_with("PDE") || name.starts_with("PDP")
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
        || name.starts_with("AD")
        || name.starts_with("AW")   // covers AWA, AWB, AWC
        || name.starts_with("FLP") || name.starts_with("IL")
        || name.starts_with("OL") || name.starts_with("CEP")
        || name.starts_with("AIY") || name.starts_with("AIZ")
        || name.starts_with("RIA") || name.starts_with("RIB")
        || name.starts_with("RIC") || name.starts_with("RIG")
        || name.starts_with("RIH")
        || name.starts_with("SA")   // covers SAA, SAB, SAD, SAI, SAS etc.
        || name.starts_with("SMB") || name.starts_with("SMD")
        || name.starts_with("RMD") || name.starts_with("RMG")
        || name.starts_with("SDQ")
        || name.starts_with("URY")
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
    pub refractory_remaining: u32,
    pub motor_role: u8,  // 0=none, 1=VB/AVB-L, 2=VB/AVB-R, 3=DB-L, 4=DB-R, 5=VA/DA/VC, 6=CP, 7=HOB
}

pub struct Simulation {
    pub neurons: Vec<NeuronState>,
    pub time: f64,
    pub connectome: Connectome,
    pub total_spikes: u64,
    pub network_activity: f32,
    pub sensor_inputs: Vec<f32>,
    pub stim_glow: Vec<u8>,
    pub params: SimParams,
    rng: rand::rngs::ThreadRng,
    potentials_buf: Vec<f32>,
}

impl Neurotransmitter {
    pub fn label(&self) -> &'static str {
        match self {
            Neurotransmitter::GABA => "GABA",
            Neurotransmitter::Glutamate => "Glu",
            Neurotransmitter::Acetylcholine => "ACh",
            Neurotransmitter::Dopamine => "DA",
            Neurotransmitter::Serotonin => "5HT",
        }
    }

    pub fn color_idx(&self) -> u8 {
        match self {
            Neurotransmitter::GABA => 0,         // red-ish
            Neurotransmitter::Glutamate => 1,     // green-ish
            Neurotransmitter::Acetylcholine => 2,  // cyan-ish
            Neurotransmitter::Dopamine => 3,       // yellow-ish
            Neurotransmitter::Serotonin => 4,      // magenta-ish
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
                let motor_role = if (name.starts_with("VB") || name == "AVBL" || name == "AVBR")
                    && (name.ends_with('L') || name.ends_with('R'))
                {
                    if name.ends_with('L') { 1 } else { 2 }
                } else if name.starts_with("DB") && (name.ends_with('L') || name.ends_with('R')) {
                    if name.ends_with('L') { 3 } else { 4 }
                } else if name.starts_with("VA") || name.starts_with("DA") || name.starts_with("VC") {
                    5
                } else if name.starts_with("CP") {
                    6
                } else if name == "HOB" {
                    7
                } else {
                    0
                };
                NeuronState {
                    id: i,
                    potential: rng.gen::<f32>() * 0.3,
                    firing: false,
                    firing_rate: 0.0,
                    spike_count: 0,
                    name,
                    neurotransmitter: nt,
                    refractory_remaining: 0,
                    motor_role,
                }
            })
            .collect();

        let n = connectome.num_neurons() as usize;
        let sensor_inputs = vec![0.0; n];

        Simulation {
            neurons,
            time: 0.0,
            connectome,
            total_spikes: 0,
            network_activity: 0.0,
            sensor_inputs,
            stim_glow: vec![0; n],
            params: SimParams::default(),
            rng,
            potentials_buf: vec![0.0; n],
        }
    }

    pub fn step(&mut self) {
        let n = self.neurons.len();
        let p = &self.params;
        let buf = &mut self.potentials_buf;

        for i in 0..n {
            let mut v = self.neurons[i].potential;

            if self.neurons[i].refractory_remaining > 0 {
                self.neurons[i].refractory_remaining -= 1;
                v = 0.0;
            } else {
                v *= p.leak_constant;
                v += p.noise_strength * gaussian_sample(&mut self.rng);
                v += self.sensor_inputs[i];
            }
            self.sensor_inputs[i] = 0.0;
            buf[i] = v;
        }

        // Phase 2: chemical synapses (pre→post)
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
                buf[post] += w;
            }
        }

        // Phase 3: gap junctions (use pre-update potentials for consistency)
        for &(a, b, weight) in self.connectome.get_gap_junction_edges() {
            let a = a as usize;
            let b = b as usize;
            if a < n && b < n {
                let diff = self.neurons[a].potential - self.neurons[b].potential;
                let coupling = GAP_JUNCTION_STRENGTH * weight as f32 * diff;
                buf[a] -= coupling;
                buf[b] += coupling;
            }
        }

        // Decay stimulation glow
        for g in &mut self.stim_glow {
            *g = g.saturating_sub(1);
        }

        // Phase 4: threshold, firing, rate update
        let threshold = p.threshold;
        let mut active_count = 0;
        for i in 0..n {
            if buf[i] >= threshold {
                self.neurons[i].potential = RESET_POTENTIAL;
                self.neurons[i].firing = true;
                self.neurons[i].spike_count += 1;
                self.total_spikes += 1;
                self.neurons[i].refractory_remaining = p.refractory_ticks();
                active_count += 1;
            } else {
                self.neurons[i].potential = buf[i];
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
        let mut top: Vec<(u16, f32)> = Vec::with_capacity(count + 1);
        for n in &self.neurons {
            let rate = n.firing_rate;
            let insert_at = top.iter().position(|&(_, r)| rate > r);
            if let Some(pos) = insert_at {
                top.insert(pos, (n.id, rate));
                if top.len() > count {
                    top.pop();
                }
            } else if top.len() < count {
                top.push((n.id, rate));
            }
        }
        top
    }

    #[allow(dead_code)]
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
                self.stim_glow[i] = 8;
            }
        }
    }

    pub fn stimulate_by_name(&mut self, name: &str, strength: f32) -> bool {
        if let Some(id) = self.connectome.id_of(name) {
            self.sensor_inputs[id as usize] += strength;
            self.stim_glow[id as usize] = 10;
            true
        } else {
            false
        }
    }

    pub fn stimulate_by_prefix(&mut self, prefix: &str, strength: f32) -> u32 {
        let ids = self.connectome.find_by_prefix(prefix);
        for &id in &ids {
            self.sensor_inputs[id as usize] += strength;
            self.stim_glow[id as usize] = 10;
        }
        ids.len() as u32
    }
}
