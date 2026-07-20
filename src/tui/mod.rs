use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph};
use ratatui::Frame;
use std::collections::VecDeque;
use std::f32::consts::TAU;
use std::time::Instant;

use crate::connectome::Connectome;
use crate::simulation::{Neurotransmitter, SimParams, Simulation};
use crate::worm::{Worm, VULVA_SEGMENT};

const CREDIT_LINES: &[&str] = &[
    "",
    "",
    "",
    "",
    "                       ╔══════════════════════════════════╗",
    "                       ║           C R E D I T S          ║",
    "                       ║                                  ║",
    "                       ║       Berke Oruc                 ║",
    "                       ║       2026                       ║",
    "                       ║                                  ║",
    "                       ║    All rights reserved.          ║",
    "                       ║    Learning project.             ║",
    "                       ║                                  ║",
    "                       ║    ~ handwritten ~               ║",
    "                       ║    Debugged by OpenCode          ║",
    "                       ║                                  ║",
    "                       ║    \"Not a brain, not a worm,     ║",
    "                       ║     just synapses\"               ║",
    "                       ╚══════════════════════════════════╝",
    "",
    "                    Special thanks to:",
    "                    White et al. 1986",
    "                    Cook et al. 2019",
    "                    OpenWorm Project",
    "                    Rust Community",
    "",
];

const TECH_DOC: &[&str] = &[
    " BIOsaka — Technical Document",
    " =============================",
    "",
    " 1. Project Philosophy",
    " ---------------------",
    " BIOsaka simulates a living creature's brain inside a",
    " computer. First target: C. elegans (307 neurons,",
    " ~2800 synapses). Long term: Drosophila, hybrid",
    " creatures, mutation experiments.",
    "",
    " The name comes from 'Bio' (biology) + 'Saka' (a type",
    " of songbird). Small but original. A system that sings.",
    "",
    " 2. Connectome Data",
    " ------------------",
    " Uses White et al. 1986 (The Mind of a Worm) EM",
    " reconstruction data. 307 neurons total, 2386 chemical",
    " synapses, 575 gap junctions.",
    "",
    " Data format: pre, post, type, weight",
    " - type=0: chemical synapse (directed)",
    " - type=1: gap junction (undirected)",
    " Each connection records the actual synapse count",
    " observed in EM scans.",
    "",
    " CSV gets compiled into a Rust static edge list at",
    " build time. Zero file I/O at runtime.",
    "",
    " 3. Neuron Model",
    " ---------------",
    " Every neuron is a LIF (Leaky Integrate-and-Fire) unit:",
    "",
    "   V(t+1) = V(t) * leak + I_syn + I_noise",
    "",
    " - leak constant: 0.95 (potential decays each step)",
    " - threshold: 1.0 (firing threshold)",
    " - reset: 0.0 (reset after firing)",
    " - noise: Gaussian ~ N(0, 0.02)",
    "",
    " Chemical synapse: when presynaptic neuron fires,",
    " postsynaptic potential changes based on the",
    " neurotransmitter released by the presynaptic neuron:",
    "",
    "   GABA (inhibitory)    → potential DECREASES by w * 0.20",
    "   Glutamate (excit.)   → potential INCREASES by w * 0.15",
    "   Acetylcholine (excit.)→ potential INCREASES by w * 0.15",
    "   Dopamine/Serotonin   → weak excitation (w * 0.075)",
    "",
    " Neurotransmitter type is inferred from each neuron's",
    " name using WormAtlas conventions. 26 GABAergic neurons",
    " provide real inhibition — network dynamics shift",
    " dramatically when they fire.",
    "",
    " Gap junction: direct electrical link between two",
    " neurons. Current flows proportional to potential diff.",
    " Coupling strength: weight * 0.05.",
    "",
    " 4. Body Model",
    " -------------",
    " Worm has 20 segments. Head (segment 0) shown as red @,",
    " body segments as white o.",
    "",
    " Movement: sinusoidal wave.",
    " Frequency depends on motor neuron activity.",
    " Amplitude depends on left-right motor asymmetry.",
    "",
    " Motor neuron groups:",
    " - VB/DB right: right muscle contraction",
    " - VB/DB left: left muscle contraction",
    " - VA/DA: forward motion",
    "",
    " Motor asymmetry makes the worm turn. Sensory neurons",
    " (AS*, AD*) get periodic stimulation to keep the",
    " network alive.",
    "",
    " 5. TUI Architecture",
    " -------------------",
    " ratatui + crossterm terminal UI with 5 tabs:",
    "",
    "  [1] Neural Graph - 307 neurons in circular layout,",
    "      color-coded by firing state",
    "      (yellow=firing, green=active,",
    "      cyan=low, gray=inactive).",
    "",
    "  [2] Worm View - animated worm body,",
    "      motor statistics panel.",
    "",
    "  [3] Statistics - top 15 firing neurons,",
    "      activity gauges, network stats.",
    "",
    "  [C] Credits - project info.",
    "",
    "  [I] Info - this technical document.",
    "",
    " Each tab updates at 33ms (~30 FPS).",
    "",
    " 6. Build Process",
    " ----------------",
    " build.rs: data/connectome.csv ->",
    " OUT_DIR/connectome_data.rs",
    "",
    " The builder generates a Rust const array from the",
    " edge list. Binary size ~500KB.",
    "",
    " Dependencies:",
    " - ratatui: terminal UI framework",
    " - crossterm: terminal control",
    " - rand: gaussian noise",
    "",
    " 7. Future Plans",
    " ---------------",
    " - Cook 2019 full dataset integration",
    " ✓ Neurotransmitter diversity (GABA/glut/acet)",
    " - Muscle physics, real body simulation",
    " - Keyboard-driven sensory input",
    " - Force-directed graph layout",
    " - Neuron label display",
    " - Record/playback",
    " - Cross-species hybrid connectomes",
    " - Drosophila connectivity data",
    " - Mutation simulation",
    "",
    " 8. References",
    " -------------",
    " [1] White et al. 1986 - The Mind of a Worm",
    " [2] Cook et al. 2019 - Nature 571, 63-71",
    " [3] OpenWorm Project - openworm.org",
    " [4] Varshney et al. 2011 - PLoS Comput Biol",
    "",
    "",
    " --- BIOsaka v0.1.4 ---",
    " The worm meets bare metal.",
    " Berke Oruc, 2026",
    "",
];

pub struct App {
    pub running: bool,
    pub paused: bool,
    pub selected_tab: usize,
    pub zoom_level: f32,
    pub graph_offset_x: f32,
    pub graph_offset_y: f32,
    pub scroll_offset: usize,
    pub show_help: bool,
    pub connectome_edges: Vec<(u16, u16, u16, u8)>,  // (pre, post, weight, type) type=0 chem, 1 gap
    pub use_force_layout: bool,
    cached_positions: Vec<(u16, u16)>,
    unit_positions: Vec<(f32, f32)>,
    last_pos_area: (u16, u16),
    pub pending_stimuli: Vec<String>,
    pub stim_message: Option<String>,
    pub stim_message_ticks: u8,
    pub auto_stim_enabled: bool,
    pub speed_multiplier: u32,
    pub hubness: Vec<u32>,
    pub neuron_groups: Vec<u8>,
    pub search_query: String,
    pub search_results: Vec<bool>,
    pub search_match_count: usize,
    pub search_active: bool,
    pub param_panel_active: bool,
    pub param_selected: usize,
    pub record_buffer: VecDeque<Vec<u8>>,
    pub is_recording: bool,
    pub playback_frame: Option<usize>,
    pub sex_label: String,
    pub herm_label: String,
    pub is_male_specific: Vec<bool>,
    pub show_labels: bool,
    pub frame_count: u64,
    pub fps: u32,
    last_fps_tick: Instant,
}

fn is_male_neuron(name: &str) -> bool {
    if name.starts_with("CP") || name.starts_with("HOB") || name.starts_with("CEM")
        || name.starts_with("SPCH") || name.starts_with("SPD") || name.starts_with("SPV")
        || name.starts_with("PCA") || name.starts_with("PCB")
        || name.starts_with("PDE") || name.starts_with("PDP")
    {
        return true;
    }
    if name.starts_with("R") && name.len() >= 2 {
        if name[1..].chars().next().map_or(false, |c| c.is_ascii_digit()) {
            return true;
        }
    }
    if name.starts_with("SA") && !name.starts_with("SAA") && !name.starts_with("SAB") && !name.starts_with("SAD") && !name.starts_with("SAI") && !name.starts_with("SAS") {
        return true;
    }
    if (name.starts_with("VA") || name.starts_with("VB") || name.starts_with("DA")) && name.len() > 2 {
        if let Ok(n) = name[2..].parse::<u8>() {
            if n >= 10 { return true; }
        }
    }
    if name.starts_with("VC") && name.len() > 2 {
        if matches!(name[2..].as_ref(), "6" | "7") { return true; }
    }
    false
}

impl App {
    pub fn new(connectome: &Connectome) -> Self {
        let mut edges: Vec<(u16, u16, u16, u8)> = connectome.get_chemical_edges().iter()
            .map(|&(a, b, w)| (a, b, w, 0)).collect();
        edges.extend(connectome.get_gap_junction_edges().iter().map(|&(a, b, w)| (a, b, w, 1)));

        // Precompute peanut unit positions (without aspect, applied per-frame)
        let n = connectome.num_neurons() as usize;
        let unit_positions: Vec<(f32, f32)> = (0..n)
            .map(|i| {
                let t = i as f32 / n.max(1) as f32;
                let angle = t * TAU;
                let lobe = (angle * 2.0).cos();
                let organic = (angle * 3.0 + 1.0).sin() * 0.015;
                let r = 0.36 + lobe * 0.06 + organic;
                let x = 0.5 + angle.cos() * r * 1.08;
                let y = 0.5 + angle.sin() * r * 0.95;
                (x, y)
            })
            .collect();

        let mut hubness = vec![0u32; n];
        for &(pre, post, _, _) in &edges {
            if (pre as usize) < n { hubness[pre as usize] += 1; }
            if (post as usize) < n { hubness[post as usize] += 1; }
        }

        let neuron_groups: Vec<u8> = (0..n).map(|i| {
            let name = connectome.neuron_name(i as u16);
            if name.starts_with("AS") || name.starts_with("AD") || name.starts_with("FLP")
                || name.starts_with("CEP") || name.starts_with("IL") || name.starts_with("OL")
                || name.starts_with("CEM") || (name.starts_with("SA") && !name.starts_with("SAA"))
            {
                0
            } else if name.starts_with("VA") || name.starts_with("DA") || name.starts_with("VB")
                || name.starts_with("DB") || name.starts_with("VC") || name.starts_with("VD")
                || name.starts_with("SM") || (name.len() == 2 && name.starts_with("M"))
                || name.starts_with("CP") || name.starts_with("HOB")
                || name.starts_with("SPCH") || name.starts_with("SPD") || name.starts_with("SPV")
            {
                1
            } else {
                2
            }
        }).collect();

        let is_male_specific: Vec<bool> = (0..n)
            .map(|i| is_male_neuron(connectome.neuron_name(i as u16)))
            .collect();

        App {
            running: true,
            paused: false,
            selected_tab: 0,
            zoom_level: 1.0,
            graph_offset_x: 0.0,
            graph_offset_y: 0.0,
            scroll_offset: 0,
            show_help: false,
            connectome_edges: edges,
            use_force_layout: false,
            cached_positions: Vec::new(),
            unit_positions,
            last_pos_area: (0, 0),
            pending_stimuli: Vec::new(),
            stim_message: None,
            stim_message_ticks: 0,
            auto_stim_enabled: true,
            speed_multiplier: 3,
            hubness,
            neuron_groups,
            search_query: String::new(),
            search_results: Vec::new(),
            search_match_count: 0,
            search_active: false,
            param_panel_active: false,
            param_selected: 0,
            record_buffer: VecDeque::new(),
            is_recording: false,
            playback_frame: None,
            sex_label: "Hermaphrodite".to_string(),
            herm_label: "307n | 2847e".to_string(),
            is_male_specific,
            show_labels: false,
            frame_count: 0,
            fps: 0,
            last_fps_tick: Instant::now(),
        }
    }

    fn compute_force_layout(&mut self, n: usize) {
        if n == 0 { return; }
        // Initialise to a circle with lobe perturbation (better than random for force layout)
        let mut pos: Vec<(f32, f32)> = (0..n)
            .map(|i| {
                let t = i as f32 / n.max(1) as f32;
                let angle = t * TAU;
                let lobe = (angle * 2.0).cos();
                let organic = (angle * 3.0 + 1.0).sin() * 0.015;
                let r = 0.32 + lobe * 0.04 + organic;
                (0.5 + angle.cos() * r * 1.08, 0.5 + angle.sin() * r * 0.95)
            })
            .collect();
        let area = 0.7 * 0.7;
        let k = (area / n as f32).sqrt();
        let mut temp = 0.5;
        for _ in 0..80 {
            let mut disp = vec![(0.0f32, 0.0f32); n];
            for i in 0..n {
                for j in (i + 1)..n {
                    let dx = pos[j].0 - pos[i].0;
                    let dy = pos[j].1 - pos[i].1;
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    let f = k * k / d;
                    let fx = f * dx / d;
                    let fy = f * dy / d;
                    disp[i].0 -= fx; disp[i].1 -= fy;
                    disp[j].0 += fx; disp[j].1 += fy;
                }
            }
            for &(pre, post, _, _) in &self.connectome_edges {
                let i = pre as usize;
                let j = post as usize;
                if i >= n || j >= n { continue; }
                let dx = pos[j].0 - pos[i].0;
                let dy = pos[j].1 - pos[i].1;
                let d = (dx * dx + dy * dy).sqrt().max(0.001);
                let f = d * d / (k * 4.0);
                let fx = f * dx / d;
                let fy = f * dy / d;
                disp[i].0 += fx; disp[i].1 += fy;
                disp[j].0 -= fx; disp[j].1 -= fy;
            }
            for i in 0..n {
                let len = (disp[i].0 * disp[i].0 + disp[i].1 * disp[i].1).sqrt().max(0.0001);
                let scale = if len > temp { temp / len } else { 1.0 };
                pos[i].0 = (pos[i].0 + disp[i].0 * scale).clamp(0.0, 1.0);
                pos[i].1 = (pos[i].1 + disp[i].1 * scale).clamp(0.0, 1.0);
            }
            temp *= 0.95;
            if temp < 0.005 { break; }
        }
        self.unit_positions = pos;
        self.use_force_layout = true;
        self.cached_positions.clear();
    }

    fn search_update(&mut self, connectome: &Connectome) {
        let n = connectome.num_neurons() as usize;
        self.search_results.clear();
        self.search_results.resize(n, false);
        self.search_match_count = 0;
        if self.search_query.is_empty() { return; }
        let q = self.search_query.to_uppercase();
        for i in 0..n {
            let name = connectome.neuron_name(i as u16).to_uppercase();
            if name.contains(&q) {
                self.search_results[i] = true;
                self.search_match_count += 1;
            }
        }
    }

    pub fn handle_input(&mut self, sim: &mut Simulation, worm: &mut Worm) -> std::io::Result<()> {
        if !event::poll(std::time::Duration::from_millis(16))? {
            return Ok(());
        }
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if self.search_active {
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter => {
                            if key.code == KeyCode::Esc {
                                self.search_query.clear();
                                self.search_results.clear();
                                self.search_match_count = 0;
                            }
                            self.search_active = false;
                        }
                        KeyCode::Backspace => {
                            self.search_query.pop();
                            self.search_update(&sim.connectome);
                        }
                        KeyCode::Char(c) if c.is_alphanumeric() || c == '_' || c == '/' || c == '-' => {
                            self.search_query.push(c);
                            self.search_update(&sim.connectome);
                        }
                        _ => {}
                    }
                } else if self.param_panel_active {
                    let nparams = SimParams::count();
                    match key.code {
                        KeyCode::Up | KeyCode::Char('w') => {
                            self.param_selected = self.param_selected.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('s') => {
                            self.param_selected = (self.param_selected + 1).min(nparams - 1);
                        }
                        KeyCode::Left | KeyCode::Char('a') => {
                            let (min, _, step) = sim.params.range(self.param_selected);
                            let cur = sim.params.get(self.param_selected);
                            sim.params.set(self.param_selected, cur - step);
                            if sim.params.get(self.param_selected) < min {
                                sim.params.set(self.param_selected, min);
                            }
                        }
                        KeyCode::Right | KeyCode::Char('d') => {
                            let (_, max, step) = sim.params.range(self.param_selected);
                            let cur = sim.params.get(self.param_selected);
                            sim.params.set(self.param_selected, cur + step);
                            if sim.params.get(self.param_selected) > max {
                                sim.params.set(self.param_selected, max);
                            }
                        }
                        KeyCode::Enter | KeyCode::Esc | KeyCode::Char('t') => {
                            self.param_panel_active = false;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => { self.running = false; }
                        KeyCode::Char(' ') => {
                            self.paused = !self.paused;
                            if !self.paused {
                                self.playback_frame = None;
                            }
                        }
                        KeyCode::Char('t') => {
                            self.param_panel_active = !self.param_panel_active;
                            self.param_selected = 0;
                        }
                        KeyCode::Char('/') => {
                            self.search_active = true;
                            self.search_query.clear();
                            self.search_results.clear();
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            self.zoom_level = (self.zoom_level * 1.2).min(5.0);
                            self.cached_positions.clear();
                        }
                        KeyCode::Char('-') => { self.zoom_level = (self.zoom_level / 1.2).max(0.2);
                            self.cached_positions.clear(); }
                        KeyCode::Left => { self.graph_offset_x -= 5.0 / self.zoom_level;
                            self.cached_positions.clear(); }
                        KeyCode::Right => { self.graph_offset_x += 5.0 / self.zoom_level;
                            self.cached_positions.clear(); }
                        KeyCode::Up => {
                            if self.selected_tab == 4 {
                                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                            } else {
                                self.graph_offset_y -= 5.0 / self.zoom_level;
                                self.cached_positions.clear();
                            }
                        }
                        KeyCode::Down => {
                            if self.selected_tab == 4 {
                                let max = TECH_DOC.len().saturating_sub(10);
                                self.scroll_offset = self.scroll_offset.saturating_add(1).min(max);
                            } else {
                                self.graph_offset_y += 5.0 / self.zoom_level;
                                self.cached_positions.clear();
                            }
                        }
                        KeyCode::Tab => { self.selected_tab = (self.selected_tab + 1) % 5; self.scroll_offset = 0; }
                        KeyCode::Char('1') => { self.selected_tab = 0; self.scroll_offset = 0; }
                        KeyCode::Char('2') => { self.selected_tab = 1; self.scroll_offset = 0; }
                        KeyCode::Char('3') => { self.selected_tab = 2; self.scroll_offset = 0; }
                        KeyCode::Char('c') | KeyCode::Char('C') => { self.selected_tab = 3; self.scroll_offset = 0; }
                        KeyCode::Char('i') | KeyCode::Char('I') => { self.selected_tab = 4; self.scroll_offset = 0; }
                        KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') => { self.show_help = !self.show_help; }
                        KeyCode::Char('f') | KeyCode::Char('F') => {
                            if self.use_force_layout {
                                self.use_force_layout = false;
                                let n = self.unit_positions.len();
                                self.unit_positions = (0..n)
                                    .map(|i| {
                                        let t = i as f32 / n.max(1) as f32;
                                        let angle = t * TAU;
                                        let lobe = (angle * 2.0).cos();
                                        let organic = (angle * 3.0 + 1.0).sin() * 0.015;
                                        let r = 0.36 + lobe * 0.06 + organic;
                                        (0.5 + angle.cos() * r * 1.08, 0.5 + angle.sin() * r * 0.95)
                                    })
                                    .collect();
                                self.cached_positions.clear();
                            } else {
                                self.compute_force_layout(self.unit_positions.len());
                            }
                        }
                        KeyCode::Char('l') | KeyCode::Char('L') => {
                            self.show_labels = !self.show_labels;
                            self.stim_message = if self.show_labels {
                                Some("Labels ON ".to_string())
                            } else {
                                Some("Labels OFF".to_string())
                            };
                            self.stim_message_ticks = 15;
                        }
                        KeyCode::Char('j') => {
                            self.pending_stimuli.push("ASEL".to_string());
                            self.stim_message = Some("Stim: ASEL ".to_string());
                            self.stim_message_ticks = 15;
                        }
                        KeyCode::Char('k') => {
                            self.pending_stimuli.push("ASER".to_string());
                            self.stim_message = Some("Stim: ASER ".to_string());
                            self.stim_message_ticks = 15;
                        }
                        KeyCode::Char('u') => {
                            self.pending_stimuli.push("AWAL".to_string());
                            self.stim_message = Some("Stim: AWAL ".to_string());
                            self.stim_message_ticks = 15;
                        }
                        KeyCode::Char('o') => {
                            self.pending_stimuli.push("AWAR".to_string());
                            self.stim_message = Some("Stim: AWAR ".to_string());
                            self.stim_message_ticks = 15;
                        }
                        KeyCode::Char('p') => {
                            self.auto_stim_enabled = !self.auto_stim_enabled;
                            self.stim_message = if self.auto_stim_enabled {
                                Some("Auto-stim ON ".to_string())
                            } else {
                                Some("Auto-stim OFF".to_string())
                            };
                            self.stim_message_ticks = 30;
                        }
                        KeyCode::Char('g') => {
                            let count = sim.stimulate_by_prefix("CEM", 0.5);
                            self.stim_message = if count > 0 {
                                Some(format!("Stim: CEM* {}n ", count))
                            } else {
                                Some("No CEM neurons ".to_string())
                            };
                            self.stim_message_ticks = 15;
                        }
                        KeyCode::Char('v') => {
                            let count = sim.stimulate_by_prefix("SA", 0.4);
                            self.stim_message = Some(format!("Stim: SA* {}n ", count));
                            self.stim_message_ticks = 15;
                        }
                        KeyCode::Char('x') => {
                            let count = sim.stimulate_by_prefix("SP", 0.5);
                            self.stim_message = if count > 0 {
                                Some(format!("Stim: SP* {}n ", count))
                            } else {
                                Some("No SP neurons ".to_string())
                            };
                            self.stim_message_ticks = 15;
                        }
                        KeyCode::Char('n') => {
                            let count = sim.stimulate_by_prefix("R", 0.5);
                            self.stim_message = if count > 0 {
                                Some(format!("Stim: R* {}n ", count))
                            } else {
                                Some("No R* neurons ".to_string())
                            };
                            self.stim_message_ticks = 15;
                        }
                        KeyCode::Char('e') => {
                            worm.add_random_obstacle();
                            self.stim_message = Some(format!("Obstacle {} ", worm.obstacle_count));
                            self.stim_message_ticks = 30;
                        }
                        KeyCode::Char('[') => {
                            let speeds = [1, 3, 10, 30, 100];
                            let idx = speeds.iter().position(|&s| s == self.speed_multiplier).unwrap_or(2);
                            self.speed_multiplier = if idx > 0 { speeds[idx - 1] } else { speeds[0] };
                            self.stim_message = Some(format!("Speed {}x ", self.speed_multiplier));
                            self.stim_message_ticks = 15;
                        }
                        KeyCode::Char(']') => {
                            let speeds = [1, 3, 10, 30, 100];
                            let idx = speeds.iter().position(|&s| s == self.speed_multiplier).unwrap_or(2);
                            self.speed_multiplier = if idx < speeds.len() - 1 { speeds[idx + 1] } else { speeds[speeds.len() - 1] };
                            self.stim_message = Some(format!("Speed {}x ", self.speed_multiplier));
                            self.stim_message_ticks = 15;
                        }
                        KeyCode::Char('r') => {
                            if self.is_recording {
                                self.is_recording = false;
                                let len = self.record_buffer.len();
                                self.stim_message = Some(format!("Record stop ({} frames) ", len));
                            } else {
                                self.record_buffer.clear();
                                self.playback_frame = None;
                                self.is_recording = true;
                                self.stim_message = Some("Record start ".to_string());
                            }
                            self.stim_message_ticks = 30;
                        }
                        KeyCode::Char(',') => {
                            if !self.record_buffer.is_empty() {
                                self.paused = true;
                                let idx = self.playback_frame.unwrap_or(self.record_buffer.len().saturating_sub(1));
                                let new_idx = idx.saturating_sub(1);
                                self.playback_frame = Some(new_idx);
                                self.stim_message = Some(format!("Frame {}/{} ",
                                    new_idx,
                                    self.record_buffer.len().saturating_sub(1)));
                                self.stim_message_ticks = 30;
                            }
                        }
                        KeyCode::Char('.') => {
                            if !self.record_buffer.is_empty() {
                                self.paused = true;
                                let idx = self.playback_frame.unwrap_or(0);
                                let next = (idx + 1).min(self.record_buffer.len().saturating_sub(1));
                                self.playback_frame = Some(next);
                                self.stim_message = Some(format!("Frame {}/{} ",
                                    next,
                                    self.record_buffer.len().saturating_sub(1)));
                                self.stim_message_ticks = 30;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame, sim: &Simulation, worm: &Worm) {
        self.frame_count += 1;
        let elapsed = self.last_fps_tick.elapsed();
        if elapsed >= std::time::Duration::from_secs(1) {
            self.fps = (self.frame_count as f64 / elapsed.as_secs_f64()) as u32;
            self.frame_count = 0;
            self.last_fps_tick = Instant::now();
        }
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(frame.area());
        if self.stim_message_ticks > 0 {
            self.stim_message_ticks -= 1;
            if self.stim_message_ticks == 0 {
                self.stim_message = None;
            }
        }
        self.draw_header(frame, chunks[0], sim);
        self.draw_main(frame, chunks[1], sim, worm);
        self.draw_footer(frame, chunks[2], sim);
        if self.show_help {
            self.draw_help_overlay(frame, frame.area());
        }
        if self.param_panel_active {
            self.draw_params_panel(frame, frame.area(), sim);
        }
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect, sim: &Simulation) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(34), Constraint::Min(0), Constraint::Length(46)])
            .split(area);

        let time_secs = sim.time as u64 / 50;
        let h = time_secs / 3600;
        let m = (time_secs % 3600) / 60;
        let s = time_secs % 60;

        let title = Paragraph::new(Line::from(Span::styled(
            " BioSaka v0.2.0 - C. elegans ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )))
        .block(Block::default().borders(Borders::ALL));

        let stats = Paragraph::new(Line::from(vec![
            Span::raw(format!(" T{:02}:{:02}:{:02} ", h, m, s)),
            Span::styled(
                format!("|Act:{:.0}% ", sim.network_activity * 100.0),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("|Spikes:{} ", sim.total_spikes),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("|{}x ", self.speed_multiplier),
                Style::default().fg(Color::Magenta),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL));

        let ctrl = Paragraph::new(Line::from(vec![
            if self.search_active {
                Span::styled(
                    format!(" Search: {}█ ", self.search_query),
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                )
            } else if self.paused {
                Span::styled(" PAUSED ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            } else {
                Span::raw(" [SPC]pause ")
            },
            Span::raw("[q]quit [c]redits [i]nfo "),
            if self.search_match_count > 0 {
                Span::styled(
                    format!("[{}] ", self.search_match_count),
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                )
            } else if let Some(ref msg) = self.stim_message {
                Span::styled(msg.as_str(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("")
            },
        ]))
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(title, chunks[0]);
        frame.render_widget(stats, chunks[1]);
        frame.render_widget(ctrl, chunks[2]);
    }

    fn draw_main(&mut self, frame: &mut Frame, area: Rect, sim: &Simulation, worm: &Worm) {
        match self.selected_tab {
            0 => self.draw_graph(frame, area, sim),
            1 => self.draw_worm(frame, area, worm, sim),
            2 => self.draw_stats(frame, area, sim),
            3 => self.draw_credits(frame, area),
            4 => self.draw_info(frame, area),
            _ => self.draw_graph(frame, area, sim),
        }
    }

    fn draw_graph(&mut self, frame: &mut Frame, area: Rect, sim: &Simulation) {
        let recording_tag = if self.is_recording {
            " REC"
        } else if self.playback_frame.is_some() {
            " PLAY"
        } else {
            ""
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Neural Network ({:.1}x) | {} {}e{}{}{}",
                self.zoom_level, self.sex_label, self.connectome_edges.len(),
                if self.use_force_layout { " force" } else { "" },
                if !self.search_results.is_empty() {
                    format!(" | search: {}", self.search_results.len())
                } else { String::new() },
                recording_tag));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let buf = frame.buffer_mut();
        let cw = inner.width.max(1) as f32;
        let ch = inner.height.max(1) as f32;
        let n = sim.neurons.len();
        let aspect = ch / cw;

        // Invalidate cached positions on terminal resize
        if self.last_pos_area != (cw as u16, ch as u16) {
            self.cached_positions.clear();
            self.last_pos_area = (cw as u16, ch as u16);
        }

        if self.cached_positions.len() != n {
            if self.use_force_layout && self.unit_positions.len() == n {
                self.cached_positions = (0..n).map(|i| {
                    let (ux, uy) = self.unit_positions[i];
                    let uy_adj = 0.5 + (uy - 0.5) * aspect;
                    let xa = (ux + self.graph_offset_x * 0.005) * self.zoom_level + (1.0 - self.zoom_level) * 0.5;
                    let ya = (uy_adj + self.graph_offset_y * 0.005) * self.zoom_level + (1.0 - self.zoom_level) * 0.5;
                    let px = (xa * cw) as u16 + inner.x;
                    let py = (ya * ch) as u16 + inner.y;
                    (px, py)
                }).collect()
            } else {
                self.cached_positions = (0..n).map(|i| {
                    let (ux, uy) = self.unit_positions[i];
                    let y_with_aspect = 0.5 + (uy - 0.5) * aspect;
                    let xa = (ux + self.graph_offset_x * 0.005) * self.zoom_level + (1.0 - self.zoom_level) * 0.5;
                    let ya = (y_with_aspect + self.graph_offset_y * 0.005) * self.zoom_level + (1.0 - self.zoom_level) * 0.5;
                    let px = (xa * cw) as u16 + inner.x;
                    let py = (ya * ch) as u16 + inner.y;
                    (px, py)
                }).collect()
            }
        }
        let positions = &self.cached_positions;

        let firing_at = |i: usize| -> bool {
            if let Some(f) = self.playback_frame {
                self.record_buffer.get(f).and_then(|b| b.get(i).copied()).unwrap_or(0) != 0
            } else {
                sim.neurons.get(i).map_or(false, |n| n.firing)
            }
        };
        let target_edges = (2000.0 * self.zoom_level) as usize;
        let step = (self.connectome_edges.len() / target_edges.max(1)).max(1);
        for idx in (0..self.connectome_edges.len()).step_by(step) {
            let (pre, post, _, etype) = self.connectome_edges[idx];
            if let (Some(&(x1, y1)), Some(&(x2, y2))) = (positions.get(pre as usize), positions.get(post as usize)) {
                let mid_active = firing_at(pre as usize) as u8 + firing_at(post as usize) as u8;
                let base = if etype == 1 { Color::Cyan } else { Color::Yellow };
                let c = if mid_active > 0 { Color::White } else { base };
                draw_line(buf, x1, y1, x2, y2, c);
            }
        }

        
        let has_search = self.search_active && !self.search_query.is_empty();
        let stim_glow_n = sim.stim_glow.len();
        for (i, &(px, py)) in positions.iter().enumerate() {
            if px >= inner.x + 1 && px < inner.x + inner.width - 1 && py >= inner.y + 1 && py < inner.y + inner.height - 1 {
                if has_search && !self.search_results[i] { continue; }
                let is_hub = self.hubness.get(i).copied().unwrap_or(0) > 20;
                let (firing, rate) = if let Some(f) = self.playback_frame {
                    let fb = self.record_buffer.get(f).and_then(|b| b.get(i).copied()).unwrap_or(0) != 0;
                    (fb, if fb { 0.1 } else { 0.0 })
                } else {
                    (sim.neurons[i].firing, sim.neurons[i].firing_rate)
                };

                let stimulated = !firing && i < stim_glow_n && sim.stim_glow[i] > 0;

                let (color, bold) = if has_search {
                    (Color::Magenta, true)
                } else if firing {
                    (Color::Yellow, true)
                } else if stimulated {
                    (Color::LightMagenta, true)
                } else if rate > 0.08 {
                    (Color::LightGreen, true)
                } else if rate > 0.04 {
                    (Color::Green, false)
                } else if rate > 0.015 {
                    (Color::Cyan, false)
                } else if rate > 0.005 {
                    (Color::Blue, false)
                } else if !firing && self.sex_label == "Male" && i < self.is_male_specific.len() && self.is_male_specific[i] {
                    match self.neuron_groups.get(i).copied().unwrap_or(2) {
                        0 => (Color::Cyan, false),
                        1 => (Color::Magenta, false),
                        _ => (Color::Yellow, false),
                    }
                } else if self.use_force_layout {
                    match self.neuron_groups.get(i).copied().unwrap_or(2) {
                        0 => (Color::DarkGray, false),
                        1 => (Color::Red, false),
                        _ => (Color::DarkGray, false),
                    }
                } else {
                    (Color::DarkGray, false)
                };
                let dot = if firing { '\u{25C9}' } else { '\u{25CF}' };
                buf[(px, py)].set_char(dot);
                let final_bold = bold || is_hub;
                if final_bold {
                    buf[(px, py)].set_style(Style::default().fg(color).add_modifier(Modifier::BOLD));
                } else {
                    buf[(px, py)].set_fg(color);
                }

                if self.show_labels {
                    let name = &sim.neurons[i].name;
                    let label_x = px.saturating_add(1);
                    let label_y = py;
                    if label_x < inner.x + inner.width - 1 {
                        for (j, ch) in name.chars().enumerate().take(6) {
                            let cx = label_x + j as u16;
                            if cx < inner.x + inner.width - 1 {
                                buf[(cx, label_y)].set_char(ch);
                                buf[(cx, label_y)].set_style(Style::default().fg(color));
                            }
                        }
                    }
                }
            }
        }

        let mut active: Vec<(usize, f32)> = Vec::with_capacity(9);
        let playback_rate = self.playback_frame.map(|f| {
            let tmp: Vec<bool> = (0..positions.len())
                .map(|i| self.record_buffer.get(f).and_then(|b| b.get(i).copied()).unwrap_or(0) != 0)
                .collect();
            tmp
        });
        for (i, _pos) in positions.iter().enumerate() {
            let r = if let Some(ref fb) = playback_rate {
                if fb[i] { 0.1 } else { 0.0 }
            } else {
                sim.neurons[i].firing_rate
            };
            if r < 0.02 { continue; }
            let insert_at = active.iter().position(|&(_, rr)| r > rr);
            if let Some(pos) = insert_at {
                active.insert(pos, (i, r));
                if active.len() > 8 {
                    active.pop();
                }
            } else if active.len() < 8 {
                active.push((i, r));
            }
        }
        for &(i, rate) in &active {
            if rate < 0.02 { break; }
            let (px, py) = positions[i];
            if py >= inner.y + 1 && py < inner.y + inner.height - 1 {
                let name = sim.connectome.neuron_name(i as u16);
                let nt_label = sim.neurons[i].neurotransmitter.label();
                let nt_color = match sim.neurons[i].neurotransmitter.color_idx() {
                    0 => Color::Red,
                    1 => Color::Green,
                    2 => Color::Cyan,
                    3 => Color::Yellow,
                    4 => Color::Magenta,
                    _ => Color::DarkGray,
                };
                let full_label = format!("{} [{}]", name, nt_label);
                let x0 = px + 2;
                let label_len = full_label.len() as u16;
                if x0 + label_len <= inner.x + inner.width {
                    for (ci, c) in full_label.chars().enumerate() {
                        let cell = &mut buf[(x0 + ci as u16, py)];
                        cell.set_char(c);
                        if ci < name.len() || c == '[' || c == ']' {
                            cell.set_fg(Color::DarkGray);
                        } else {
                            cell.set_fg(nt_color);
                        }
                    }
                }
            }
        }
    }

    fn draw_worm(&self, frame: &mut Frame, area: Rect, worm: &Worm, sim: &Simulation) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(26)])
            .split(area);

        let block = Block::default().borders(Borders::ALL).title(" Worm Body ");
        let inner = block.inner(chunks[0]);
        frame.render_widget(block, chunks[0]);
        let buf = frame.buffer_mut();
        let cw = inner.width.max(1) as f32;
        let ch = inner.height.max(1) as f32;
        let cx = worm.body_center_x();
        let cy = worm.body_center_y();

        let is_male = self.sex_label == "Male";
        let tail_start = worm.segments.len().saturating_sub(3);

        let seg_px: Vec<(u16, u16)> = worm.segments.iter().map(|seg| {
            let px = ((seg.x - cx + 0.5) * cw * 0.8 + cw * 0.1) as u16 + inner.x;
            let py = ((seg.y - cy + 0.5) * ch * 0.8 + ch * 0.1) as u16 + inner.y;
            (px, py)
        }).collect();

        for i in 1..worm.segments.len() {
            let (x1, y1) = seg_px[i - 1];
            let (x2, y2) = seg_px[i];
            let frac = i as f32 / worm.segments.len().max(1) as f32;
            let line_color = if i == 1 { Color::LightRed }
                else if is_male && i >= tail_start { Color::Green }
                else if frac < 0.15 { Color::LightRed }
                else if frac < 0.40 { Color::Red }
                else if frac < 0.70 { Color::White }
                else { Color::DarkGray };
            draw_body_line(buf, x1, y1, x2, y2, line_color);
        }

        for (i, _seg) in worm.segments.iter().enumerate() {
            let (px, py) = seg_px[i];
            if px < inner.x + inner.width && py < inner.y + inner.height && px > inner.x && py > inner.y {
                let frac = i as f32 / worm.segments.len().max(1) as f32;
                let (ch, color) = if i == 0 {
                    ('\u{2588}', Color::LightRed)
                } else if is_male && i >= tail_start {
                    ('\u{25C9}', Color::Green)
                } else if frac < 0.15 {
                    ('\u{25CF}', Color::Red)
                } else if frac < 0.40 {
                    ('\u{25CF}', Color::LightRed)
                } else if frac < 0.70 {
                    ('\u{25D0}', Color::White)
                } else {
                    ('\u{25D1}', Color::DarkGray)
                };
                buf[(px, py)].set_char(ch);
                buf[(px, py)].set_fg(color);
            }
        }
        if !is_male {
                        let vulva_idx = VULVA_SEGMENT.min(worm.segments.len().saturating_sub(1));
            if let Some(seg) = worm.segments.get(vulva_idx) {
                let px = ((seg.x - cx + 0.5) * cw * 0.8 + cw * 0.1) as u16 + inner.x;
                let py = ((seg.y - cy + 0.5) * ch * 0.8 + ch * 0.1) as u16 + inner.y;
                if px < inner.x + inner.width && py < inner.y + inner.height && px > inner.x && py > inner.y {
                    buf[(px, py)].set_char('\u{25C6}');
                    buf[(px, py)].set_fg(Color::LightRed);
                }
            }
        }

        // Obstacles
        for ob in &worm.obstacles {
            let x1 = ((ob.x - cx + 0.5) * cw * 0.8 + cw * 0.1) as u16 + inner.x;
            let y1 = ((ob.y - cy + 0.5) * ch * 0.8 + ch * 0.1) as u16 + inner.y;
            let x2 = ((ob.x + ob.w - cx + 0.5) * cw * 0.8 + cw * 0.1) as u16 + inner.x;
            let y2 = ((ob.y + ob.h - cy + 0.5) * ch * 0.8 + ch * 0.1) as u16 + inner.y;
            for px in x1..=x2.min(inner.x + inner.width - 1) {
                for py in y1..=y2.min(inner.y + inner.height - 1) {
                    if px >= inner.x && py >= inner.y {
                        let cell = &mut buf[(px, py)];
                        if cell.symbol() == " " {
                            cell.set_char('\u{2588}');
                            cell.set_fg(Color::Red);
                        }
                    }
                }
            }
        }

        let mut lm = 0.0f32;
        let mut rm = 0.0f32;
        let mut lc = 0u32;
        let mut rc = 0u32;
        for n in &sim.neurons {
            if n.name.starts_with("VB") || n.name.starts_with("DB") {
                if n.name.ends_with('L') { lm += n.firing_rate; lc += 1; }
                if n.name.ends_with('R') { rm += n.firing_rate; rc += 1; }
            }
        }
        let lm_avg = if lc > 0 { lm / lc as f32 } else { 0.0 };
        let rm_avg = if rc > 0 { rm / rc as f32 } else { 0.0 };

        let bars = [
            format!("L-motor {:>5.0}% {}", lm_avg * 100.0,
                "\u{2588}".repeat((lm_avg * 20.0) as usize)),
            format!("R-motor {:>5.0}% {}", rm_avg * 100.0,
                "\u{2588}".repeat((rm_avg * 20.0) as usize)),
        ];

        let mut list_items: Vec<ListItem> = bars.iter().map(|b| {
            ListItem::new(Line::from(Span::styled(b, Style::default().fg(Color::Green))))
        }).collect();

        list_items.push(ListItem::new(Line::from(Span::raw(""))));
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled("Speed ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{:.4}", worm.speed)),
        ])));
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled("Segments ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", worm.segments.len())),
        ])));
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled("Phase ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{:.2}", worm.body_wave_phase)),
        ])));
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled("Obstacles ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}  [e] add", worm.obstacle_count)),
        ])));
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled("Sex ", Style::default().fg(Color::Cyan)),
            Span::styled(self.sex_label.as_str(), Style::default().fg(Color::LightGreen)),
        ])));

        let legend = List::new(list_items).block(Block::default().borders(Borders::ALL).title(" Motor "));
        frame.render_widget(legend, chunks[1]);
    }

    fn draw_stats(&self, frame: &mut Frame, area: Rect, sim: &Simulation) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        let top = sim.get_top_firing(15);
        let items: Vec<ListItem> = top.iter().enumerate().map(|(i, (id, rate))| {
            let name = &sim.neurons[*id as usize].name;
            let bar = "\u{2588}".repeat((rate * 30.0) as usize);
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>2}. ", i + 1), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:<6}", name), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {:>5.1}%", rate * 100.0), Style::default().fg(Color::Yellow)),
                Span::styled(format!(" {}", bar), Style::default().fg(Color::Green)),
            ]))
        }).collect();

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Top Firing "));
        frame.render_widget(list, chunks[0]);

        let rc = Layout::default().direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(5), Constraint::Min(0)])
            .split(chunks[1]);

        let ag = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Activity "))
            .gauge_style(Style::default().fg(Color::Yellow))
            .ratio(sim.network_activity as f64);
        frame.render_widget(ag, rc[0]);

        let sr = (sim.total_spikes as f64 / (sim.time + 1.0).max(1.0)) / 50.0;
        let sg = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Spike Rate "))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(sr.min(1.0));
        frame.render_widget(sg, rc[1]);

        let mut sensory_count = 0u32;
        let mut motor_count = 0u32;
        let mut inter_count = 0u32;
        let mut sensory_active = 0u32;
        let mut motor_active = 0u32;
        let mut inter_active = 0u32;
        let mut sensory_rate = 0.0f32;
        let mut motor_rate = 0.0f32;
        let mut inter_rate = 0.0f32;
        let mut low = 0u32; let mut med = 0u32; let mut high = 0u32;
        let mut gaba_count = 0u32;
        let mut glu_count = 0u32;
        let mut ach_count = 0u32;
        let mut hubs: Vec<(u16, u32)> = Vec::with_capacity(6);

        for (i, n) in sim.neurons.iter().enumerate() {
            let rate = n.firing_rate;

            if rate <= 0.02 { low += 1; }
            else if rate <= 0.08 { med += 1; }
            else { high += 1; }

            match n.neurotransmitter {
                Neurotransmitter::GABA => gaba_count += 1,
                Neurotransmitter::Glutamate => glu_count += 1,
                Neurotransmitter::Acetylcholine => ach_count += 1,
                _ => {}
            }

            if let Some(&h) = self.hubness.get(i) {
                let insert_at = hubs.iter().position(|&(_, hh)| h > hh);
                if let Some(pos) = insert_at {
                    hubs.insert(pos, (n.id, h));
                    if hubs.len() > 5 { hubs.pop(); }
                } else if hubs.len() < 5 {
                    hubs.push((n.id, h));
                }
            }

            let is_sensory = n.name.starts_with("AS")
                || n.name.starts_with("AD")
                || n.name.starts_with("FLP")
                || n.name.starts_with("CEP")
                || n.name.starts_with("IL")
                || n.name.starts_with("OL")
                || n.name.starts_with("CEM")
                || (n.name.starts_with("SA") && !n.name.starts_with("SAA"));
            let is_motor = n.name.starts_with("VA")
                || n.name.starts_with("DA")
                || n.name.starts_with("VB")
                || n.name.starts_with("DB")
                || n.name.starts_with("VC")
                || n.name.starts_with("VD")
                || n.name.starts_with("SM")
                || n.name == "M1" || n.name == "M2" || n.name == "M3" || n.name == "M4" || n.name == "M5"
                || n.name.starts_with("CP") || n.name.starts_with("HOB")
                || n.name.starts_with("SPCH") || n.name.starts_with("SPD") || n.name.starts_with("SPV");

            if is_sensory {
                sensory_count += 1;
                if n.firing { sensory_active += 1; }
                sensory_rate += rate;
            } else if is_motor {
                motor_count += 1;
                if n.firing { motor_active += 1; }
                motor_rate += rate;
            } else {
                inter_count += 1;
                if n.firing { inter_active += 1; }
                inter_rate += rate;
            }
        }
        let sensory_avg = sensory_rate / sensory_count.max(1) as f32;
        let motor_avg = motor_rate / motor_count.max(1) as f32;
        let inter_avg = inter_rate / inter_count.max(1) as f32;

        fn bar(rate: f32, color: Color) -> Span<'static> {
            let filled = (rate * 10.0).min(10.0).round() as usize;
            Span::styled(
                "\u{2588}".repeat(filled) + &" ".repeat(10 - filled),
                Style::default().fg(color),
            )
        }

        let group_info = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(" SEN ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:>2}/{:<2}", sensory_active, sensory_count), Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:>5.1}% ", sensory_avg * 100.0), Style::default().fg(Color::Green)),
                bar(sensory_avg, Color::Cyan),
            ]),
            Line::from(vec![
                Span::styled(" MOT ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:>2}/{:<2}", motor_active, motor_count), Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:>5.1}% ", motor_avg * 100.0), Style::default().fg(Color::Green)),
                bar(motor_avg, Color::LightRed),
            ]),
            Line::from(vec![
                Span::styled(" INT ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:>2}/{:<2}", inter_active, inter_count), Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:>5.1}% ", inter_avg * 100.0), Style::default().fg(Color::Green)),
                bar(inter_avg, Color::White),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Groups "));
        frame.render_widget(group_info, rc[2]);

        let total_active = sensory_active + motor_active + inter_active;
        let sync_idx = (total_active as f32 / sim.neurons.len() as f32).powi(2);
        let mut hub_lines: Vec<Line> = hubs.iter().take(5).map(|&(id, cnt)| {
            Line::from(vec![
                Span::styled("  ", Style::default().fg(Color::DarkGray)),
                Span::styled(&sim.neurons[id as usize].name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" ({})", cnt), Style::default().fg(Color::DarkGray)),
            ])
        }).collect();
        if hub_lines.is_empty() {
            hub_lines.push(Line::from(Span::raw("")));
        }

        let mut info_lines = vec![
            Line::from(Span::styled("C. elegans Connectome", Style::default().fg(Color::Cyan))),
            Line::from(Span::raw(format!("Chemical: {}", sim.connectome.total_chemical_synapses()))),
            Line::from(Span::raw(format!("Gap junct: {}", sim.connectome.total_gap_junctions()))),
            Line::from(Span::raw(format!("Active: {}/{}", total_active, sim.neurons.len()))),
            Line::from(Span::raw(format!("Total spikes: {}", sim.total_spikes))),
            Line::from(Span::styled(format!("Sex: {:15}  {}", self.sex_label, self.herm_label), Style::default().fg(Color::LightGreen))),
            Line::from(Span::raw(format!("Steps: {:.0}", sim.time))),
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled("Sync idx ", Style::default().fg(Color::Cyan)),
                Span::styled(format!("{:.3}", sync_idx), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Rate dist ", Style::default().fg(Color::Cyan)),
                Span::styled(format!("L:{} ", low), Style::default().fg(Color::Blue)),
                Span::styled(format!("M:{} ", med), Style::default().fg(Color::Green)),
                Span::styled(format!("H:{}", high), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("NT ", Style::default().fg(Color::Cyan)),
                Span::styled(format!("GABA:{} ", gaba_count), Style::default().fg(Color::Red)),
                Span::styled(format!("Glu:{} ", glu_count), Style::default().fg(Color::Green)),
                Span::styled(format!("ACh:{}", ach_count), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::styled("Hubs:", Style::default().fg(Color::Cyan))),
        ];
        info_lines.extend(hub_lines);
        let info = Paragraph::new(info_lines)
            .block(Block::default().borders(Borders::ALL).title(" Stats "));
        frame.render_widget(info, rc[3]);
    }

    fn draw_credits(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title(" Credits ");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let lines: Vec<Line> = CREDIT_LINES.iter().map(|l| {
            if l.contains("C R E D I T S") {
                Line::from(Span::styled(*l, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
            } else if l.contains("Berke") {
                Line::from(Span::styled(*l, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
            } else if l.contains("2026") || l.contains("rights") {
                Line::from(Span::styled(*l, Style::default().fg(Color::Yellow)))
            } else if l.contains("OpenCode") {
                Line::from(Span::styled(*l, Style::default().fg(Color::Magenta)))
            } else if l.contains("synapses") {
                Line::from(Span::styled(*l, Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)))
            } else if l.contains("thanks") || l.contains("Thanks") {
                Line::from(Span::styled(*l, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)))
            } else {
                Line::from(Span::raw(*l))
            }
        }).collect();

        let para = Paragraph::new(lines)
            .block(Block::default());
        frame.render_widget(para, inner);
    }

    fn draw_info(&self, frame: &mut Frame, area: Rect) {
        let current_max = TECH_DOC.len().saturating_sub(1);
        let scroll = self.scroll_offset.min(current_max);

        let block = Block::default().borders(Borders::ALL)
            .title(format!(" Technical Info ({}/{}) ", scroll + 1, current_max + 1));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let take_n = inner.height.max(5) as usize;
        let visible_lines: Vec<Line> = TECH_DOC.iter()
            .skip(scroll)
            .take(take_n)
            .map(|l| {
                if l.starts_with(" BIOsaka") {
                    Line::from(Span::styled(*l, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
                } else if l.starts_with(" ===") {
                    Line::from(Span::styled(*l, Style::default().fg(Color::Cyan)))
                } else if l.len() < 30 && l.contains("--") && l.starts_with(" ") {
                    Line::from(Span::styled(*l, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
                } else if l.starts_with(" [") {
                    Line::from(Span::styled(*l, Style::default().fg(Color::Green)))
                } else if l.starts_with(" ---") {
                    Line::from(Span::styled(*l, Style::default().fg(Color::Cyan)))
                } else if l.starts_with(" ") && l.contains(':') && !l.contains("--") {
                    Line::from(Span::styled(*l, Style::default().fg(Color::White)))
                } else {
                    Line::from(Span::raw(*l))
                }
            })
            .collect();

        let para = Paragraph::new(visible_lines);
        frame.render_widget(para, inner);
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect, sim: &Simulation) {
        let stim_label = if self.auto_stim_enabled {
            " \u{25B6}Auto"
        } else {
            " \u{23F8}Auto"
        };
        let stim_color = if self.auto_stim_enabled { Color::Green } else { Color::DarkGray };
        let tick_label = format!(" {:>6.0}t ", sim.time);
        let tabs = [
            " [1]Graph ",
            " [2]Worm ",
            " [3]Stats ",
            " [C]redit ",
            " [I]nfo ",
            " [H]elp ",
            " [j]stim ",
        ];
        let mut spans: Vec<Span> = tabs.iter().enumerate().map(|(i, name)| {
            if i == self.selected_tab {
                Span::styled(*name, Style::default().fg(Color::Black).bg(Color::Cyan))
            } else {
                Span::styled(*name, Style::default().fg(Color::White))
            }
        }).collect();
        spans.push(Span::styled(stim_label, Style::default().fg(stim_color)));
        let fps_label = format!(" {:>3}fps ", self.fps);
        spans.push(Span::styled(fps_label, Style::default().fg(Color::DarkGray)));
        spans.push(Span::raw(tick_label));
        frame.render_widget(Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL)), area);
    }

    fn draw_params_panel(&self, frame: &mut Frame, area: Rect, sim: &Simulation) {
        let panel_w = 38.min(area.width.saturating_sub(4));
        let panel_h = 10.min(area.height.saturating_sub(4));
        let x = (area.width - panel_w) / 2;
        let y = (area.height - panel_h) / 2;
        let overlay = Rect { x, y, width: panel_w, height: panel_h };
        frame.render_widget(Clear, overlay);

        let param_colors = [Color::Yellow, Color::Magenta, Color::Cyan, Color::Green];
        let labels = SimParams::labels();

        let mut lines: Vec<Line> = Vec::new();
        for i in 0..SimParams::count() {
            let val = sim.params.get(i);
            let (min, max, _step) = sim.params.range(i);
            let selected = i == self.param_selected;
            let marker = if selected { "\u{25B6} " } else { "  " };
            let bar_len = ((val - min) / (max - min).max(0.001) * 10.0).round() as usize;
            let bar = "\u{2588}".repeat(bar_len) + &"\u{2591}".repeat(10 - bar_len);
            let style = if selected {
                Style::default().fg(param_colors[i]).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(param_colors[i])
            };
            let val_style = Style::default().fg(Color::White);
            lines.push(Line::from(vec![
                Span::styled(format!("{}{:<8}", marker, labels[i]), style),
                Span::styled(format!(" {:.3} ", val), val_style),
                Span::styled(bar, Style::default().fg(param_colors[i])),
            ]));
        }
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            " \u{2191}\u{2193} select  \u{2190}\u{2192} adjust  [t] close",
            Style::default().fg(Color::DarkGray),
        )));

        let para = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Parameters "))
            .style(Style::default().fg(Color::White).bg(Color::Black));
        frame.render_widget(para, overlay);
    }

    fn draw_help_overlay(&self, frame: &mut Frame, area: Rect) {
        let help_w = 44.min(area.width.saturating_sub(4));
        let help_h = 25.min(area.height.saturating_sub(4));
        let x = (area.width - help_w) / 2;
        let y = (area.height - help_h) / 2;
        let overlay = Rect { x, y, width: help_w, height: help_h };
        frame.render_widget(Clear, overlay);
        let sex_display = format!("C. elegans {}", self.sex_label.to_lowercase());
        let help_data: [(&str, Color, bool); 34] = [
            (" Controls ", Color::Cyan, true),
            ("", Color::White, false),
            (" [1] [2] [3]  switch tabs", Color::White, false),
            (" [c]          credits tab", Color::White, false),
            (" [i]          info tab", Color::White, false),
            (" [h] [?]      this help", Color::White, false),
            (" [Space]      pause/resume", Color::White, false),
            (" [+]/[-]      zoom in/out", Color::White, false),
            (" [Arrows]     pan (graph) / scroll (info)", Color::White, false),
             (" [f]          toggle force layout", Color::White, false),
             (" [t]          parameter tuning panel", Color::White, false),
             (" []/[]        speed up/down", Color::White, false),
            (" [/]          search neuron", Color::White, false),
            (" [l]          toggle labels", Color::White, false),
            (" Stimulation ", Color::Green, true),
             (" [j]          poke ASEL (head left)", Color::White, false),
             (" [k]          poke ASER (head right)", Color::White, false),
             (" [u]          poke AWAL (olfaction L)", Color::White, false),
             (" [o]          poke AWAR (olfaction R)", Color::White, false),
              (" [g]          poke CEM* (male head)", Color::White, false),
              (" [v]          poke SA* (sensory assoc)", Color::White, false),
              (" [n]          poke R* (male rays)", Color::White, false),
              (" [x]          poke SP* (male spicules)", Color::White, false),
              (" [p]          toggle auto-stimulation", Color::White, false),
              (" [e]          add obstacle", Color::White, false),
              ("", Color::White, false),
             (" Record ", Color::Green, true),
             (" [r]          record/stop", Color::White, false),
             (" [,] [.]      prev/next frame", Color::White, false),
            ("", Color::White, false),
            (" [q]          quit", Color::White, false),
            ("", Color::White, false),
            (self.herm_label.as_str(), Color::DarkGray, false),
            (&sex_display, Color::DarkGray, false),
        ];
        let lines: Vec<Line> = help_data.iter().map(|(text, color, bold)| {
            let mut style = Style::default().fg(*color);
            if *bold { style = style.add_modifier(Modifier::BOLD); }
            Line::from(Span::styled(*text, style))
        }).collect();
        let para = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Help "))
            .style(Style::default().fg(Color::White).bg(Color::Black));
        frame.render_widget(para, overlay);
    }
}

fn draw_body_line(buf: &mut ratatui::buffer::Buffer, x1: u16, y1: u16, x2: u16, y2: u16, color: Color) {
    let bw = buf.area.width as i32;
    let bh = buf.area.height as i32;
    let mut x = x1 as i32;
    let mut y = y1 as i32;
    let dx = (x2 as i32 - x1 as i32).abs();
    let dy = -(y2 as i32 - y1 as i32).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x >= 0 && y >= 0 && x < bw && y < bh {
            buf[(x as u16, y as u16)].set_char('\u{2588}');
            buf[(x as u16, y as u16)].set_fg(color);
        }
        if x == x2 as i32 && y == y2 as i32 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

fn draw_line(buf: &mut ratatui::buffer::Buffer, x1: u16, y1: u16, x2: u16, y2: u16, color: Color) {
    let bw = buf.area.width as i32;
    let bh = buf.area.height as i32;
    let mut x = x1 as i32;
    let mut y = y1 as i32;
    let dx = (x2 as i32 - x1 as i32).abs();
    let dy = -(y2 as i32 - y1 as i32).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x >= 0 && y >= 0 && x < bw && y < bh {
            let cell = &mut buf[(x as u16, y as u16)];
            if cell.symbol() == " " || cell.symbol() == "\u{00B7}" {
                cell.set_char('\u{00B7}');
                cell.set_fg(color);
            }
        }
        if x == x2 as i32 && y == y2 as i32 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}
