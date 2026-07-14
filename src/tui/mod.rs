use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::connectome::Connectome;
use crate::simulation::Simulation;
use crate::worm::Worm;

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
    " postsynaptic potential increases by weight * 0.15.",
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
    " - Neurotransmitter diversity (GABA/glut/acet)",
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
    " --- BIOsaka v0.1 ---",
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
    pub connectome_edges: Vec<(u16, u16, u16)>,
}

impl App {
    pub fn new(connectome: &Connectome) -> Self {
        let mut chemical_edges = connectome.get_chemical_edges().to_vec();
        chemical_edges.extend(connectome.get_gap_junction_edges().iter().map(|&(a, b, w)| (a, b, w)));
        App {
            running: true,
            paused: false,
            selected_tab: 0,
            zoom_level: 1.0,
            graph_offset_x: 0.0,
            graph_offset_y: 0.0,
            scroll_offset: 0,
            connectome_edges: chemical_edges,
        }
    }

    pub fn handle_input(&mut self) -> std::io::Result<()> {
        if !event::poll(std::time::Duration::from_millis(16))? {
            return Ok(());
        }
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => { self.running = false; }
                    KeyCode::Char(' ') => { self.paused = !self.paused; }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        self.zoom_level = (self.zoom_level * 1.2).min(5.0);
                    }
                    KeyCode::Char('-') => { self.zoom_level = (self.zoom_level / 1.2).max(0.2); }
                    KeyCode::Left => { self.graph_offset_x -= 5.0 / self.zoom_level; }
                    KeyCode::Right => { self.graph_offset_x += 5.0 / self.zoom_level; }
                    KeyCode::Up => {
                        if self.selected_tab == 4 {
                            self.scroll_offset = self.scroll_offset.saturating_sub(1);
                        } else {
                            self.graph_offset_y -= 5.0 / self.zoom_level;
                        }
                    }
                    KeyCode::Down => {
                        if self.selected_tab == 4 {
                            let max = TECH_DOC.len().saturating_sub(10);
                            self.scroll_offset = self.scroll_offset.saturating_add(1).min(max);
                        } else {
                            self.graph_offset_y += 5.0 / self.zoom_level;
                        }
                    }
                    KeyCode::Tab => { self.selected_tab = (self.selected_tab + 1) % 5; self.scroll_offset = 0; }
                    KeyCode::Char('1') => { self.selected_tab = 0; self.scroll_offset = 0; }
                    KeyCode::Char('2') => { self.selected_tab = 1; self.scroll_offset = 0; }
                    KeyCode::Char('3') => { self.selected_tab = 2; self.scroll_offset = 0; }
                    KeyCode::Char('c') | KeyCode::Char('C') => { self.selected_tab = 3; self.scroll_offset = 0; }
                    KeyCode::Char('i') | KeyCode::Char('I') => { self.selected_tab = 4; self.scroll_offset = 0; }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame, sim: &Simulation, worm: &Worm) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(frame.area());
        self.draw_header(frame, chunks[0], sim);
        self.draw_main(frame, chunks[1], sim, worm);
        self.draw_footer(frame, chunks[2]);
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
            " BioSaka v0.1 - C. elegans ",
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
        ]))
        .block(Block::default().borders(Borders::ALL));

        let ctrl = Paragraph::new(Line::from(vec![
            if self.paused {
                Span::styled(" PAUSED ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            } else {
                Span::raw(" [SPC]pause ")
            },
            Span::raw("[q]quit [c]redits [i]nfo "),
        ]))
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(title, chunks[0]);
        frame.render_widget(stats, chunks[1]);
        frame.render_widget(ctrl, chunks[2]);
    }

    fn draw_main(&self, frame: &mut Frame, area: Rect, sim: &Simulation, worm: &Worm) {
        match self.selected_tab {
            0 => self.draw_graph(frame, area, sim),
            1 => self.draw_worm(frame, area, worm, sim),
            2 => self.draw_stats(frame, area, sim),
            3 => self.draw_credits(frame, area),
            4 => self.draw_info(frame, area),
            _ => self.draw_graph(frame, area, sim),
        }
    }

    fn draw_graph(&self, frame: &mut Frame, area: Rect, sim: &Simulation) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Neural Network ({:.1}x) | {} neurons | {} edges ",
                self.zoom_level, 307usize, self.connectome_edges.len()));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let buf = frame.buffer_mut();
        let cw = inner.width.max(1) as f32;
        let ch = inner.height.max(1) as f32;
        let n = sim.neurons.len();
        let aspect = ch / cw;

        let positions: Vec<(u16, u16)> = (0..n)
            .map(|i| {
                let t = i as f32 / n.max(1) as f32;
                let angle = t * 6.2832;
                // Two-hemisphere brain shape using peanut/lobe parameterization
                // cos(2θ) bulges at sides (0°/180°) and pinches at top/bottom (90°/270°)
                let lobe = (angle * 2.0).cos();
                let organic = (angle * 3.0 + 1.0).sin() * 0.015;
                let r = 0.36 + lobe * 0.06 + organic;
                let sx = 1.08;
                let sy = 0.95;
                let x = 0.5 + angle.cos() * r * sx;
                let y = 0.5 + angle.sin() * r * sy * aspect;
                let xa = (x + self.graph_offset_x * 0.005) * self.zoom_level + (1.0 - self.zoom_level) * 0.5;
                let ya = (y + self.graph_offset_y * 0.005) * self.zoom_level + (1.0 - self.zoom_level) * 0.5;
                let px = (xa * cw) as u16 + inner.x;
                let py = (ya * ch) as u16 + inner.y;
                (px, py)
            })
            .collect();

        let step = (self.connectome_edges.len() / 1500).max(1);
        for idx in (0..self.connectome_edges.len()).step_by(step) {
            let (pre, post, _) = self.connectome_edges[idx];
            if let (Some(&(x1, y1)), Some(&(x2, y2))) = (positions.get(pre as usize), positions.get(post as usize)) {
                let mid_active = sim.neurons[pre as usize].firing as u8 + sim.neurons[post as usize].firing as u8;
                let c = if mid_active > 0 { Color::Gray } else { Color::DarkGray };
                draw_line(buf, x1, y1, x2, y2, c);
            }
        }

        for (i, &(px, py)) in positions.iter().enumerate() {
            if px >= inner.x + 1 && px < inner.x + inner.width - 1 && py >= inner.y + 1 && py < inner.y + inner.height - 1 {
                let rate = sim.neurons[i].firing_rate;
                let (color, bold) = if sim.neurons[i].firing {
                    (Color::Yellow, true)
                } else if rate > 0.08 {
                    (Color::LightGreen, true)
                } else if rate > 0.04 {
                    (Color::Green, false)
                } else if rate > 0.015 {
                    (Color::Cyan, false)
                } else if rate > 0.005 {
                    (Color::Blue, false)
                } else {
                    (Color::DarkGray, false)
                };
                let dot = if sim.neurons[i].firing { '\u{25C9}' } else { '\u{25CF}' };
                buf[(px, py)].set_char(dot);
                if bold {
                    buf[(px, py)].set_style(Style::default().fg(color).add_modifier(Modifier::BOLD));
                } else {
                    buf[(px, py)].set_fg(color);
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

        for (i, seg) in worm.segments.iter().enumerate() {
            let px = ((seg.x - cx + 0.5) * cw * 0.8 + cw * 0.1) as u16 + inner.x;
            let py = ((seg.y - cy + 0.5) * ch * 0.8 + ch * 0.1) as u16 + inner.y;
            if px < inner.x + inner.width && py < inner.y + inner.height && px > inner.x && py > inner.y {
                let frac = i as f32 / worm.segments.len().max(1) as f32;
                let (ch, color) = if i == 0 {
                    ('\u{2588}', Color::LightRed)
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

        let total_active = sim.neurons.iter().filter(|n| n.firing).count();
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

        // Classify neuron groups by name prefix
        let mut sensory_count = 0u32;
        let mut motor_count = 0u32;
        let mut inter_count = 0u32;
        let mut sensory_active = 0u32;
        let mut motor_active = 0u32;
        let mut inter_active = 0u32;
        let mut sensory_rate = 0.0f32;
        let mut motor_rate = 0.0f32;
        let mut inter_rate = 0.0f32;

        for n in &sim.neurons {
            let rate = n.firing_rate;
            let is_sensory = n.name.starts_with("AS")
                || n.name.starts_with("AD")
                || n.name.starts_with("FLP")
                || n.name.starts_with("CEP")
                || n.name.starts_with("IL")
                || n.name.starts_with("OL");
            let is_motor = n.name.starts_with("VA")
                || n.name.starts_with("DA")
                || n.name.starts_with("VB")
                || n.name.starts_with("DB")
                || n.name.starts_with("VC")
                || n.name.starts_with("VD")
                || n.name.starts_with("SM")
                || n.name == "M1" || n.name == "M2" || n.name == "M3" || n.name == "M4" || n.name == "M5";

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

        let info = Paragraph::new(vec![
            Line::from(Span::styled("C. elegans Connectome", Style::default().fg(Color::Cyan))),
            Line::from(Span::raw(format!("Chemical: {}", sim.connectome.total_chemical_synapses()))),
            Line::from(Span::raw(format!("Gap junct: {}", sim.connectome.total_gap_junctions()))),
            Line::from(Span::raw(format!("Active: {}/{}", total_active, sim.neurons.len()))),
            Line::from(Span::raw(format!("Total spikes: {}", sim.total_spikes))),
            Line::from(Span::raw(format!("Steps: {:.0}", sim.time))),
        ])
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

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let tabs = [
            " [1]Graph ",
            " [2]Worm ",
            " [3]Stats ",
            " [C]redit ",
            " [I]nfo ",
        ];
        let spans: Vec<Span> = tabs.iter().enumerate().map(|(i, name)| {
            if i == self.selected_tab {
                Span::styled(*name, Style::default().fg(Color::Black).bg(Color::Cyan))
            } else {
                Span::styled(*name, Style::default().fg(Color::White))
            }
        }).collect();
        frame.render_widget(Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL)), area);
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
