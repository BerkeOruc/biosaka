mod connectome;
mod simulation;
mod worm;
mod tui;
mod generated;

use clap::Parser;
use connectome::Connectome;
use generated::Sex;
use simulation::Simulation;
use worm::Worm;
use tui::App;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "biosaka", version, about = "C. elegans neural simulation TUI")]
struct Cli {
    /// Sex: hermaphrodite (default) or male
    #[arg(short = 's', long = "sex", default_value = "hermaphrodite")]
    sex: String,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let sex = match cli.sex.as_str() {
        "male" | "m" => Sex::Male,
        _ => Sex::Hermaphrodite,
    };

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let connectome = Connectome::load(sex);
    let mut simulation = Simulation::new(connectome);
    let mut worm = Worm::new();
    worm.set_sex(&simulation.connectome.sex_label());
    let mut app = App::new(&simulation.connectome);

    let n_neurons = simulation.connectome.num_neurons();
    let n_edges = simulation.connectome.total_connections();
    app.sex_label = simulation.connectome.sex_label().to_string();
    app.herm_label = format!("{}n | {}e", n_neurons, n_edges);

    let tick_rate = Duration::from_millis(33);
    let mut last_tick = Instant::now();

    simulation.stimulate_sensory_neurons(0.15);

    while app.running {
        app.handle_input(&mut simulation, &mut worm)?;

        for name in app.pending_stimuli.drain(..) {
            simulation.stimulate_by_name(&name, 0.5);
        }

        if !app.paused {
            for _ in 0..app.speed_multiplier {
                simulation.step();
            }
            if app.is_recording {
                app.record_buffer.push(
                    simulation.neurons.iter().map(|n| if n.firing { 1u8 } else { 0u8 }).collect()
                );
                if app.record_buffer.len() > 60000 {
                    app.record_buffer.remove(0);
                }
            }
            worm.update(&simulation);

            if app.auto_stim_enabled && simulation.time as u64 % 100 == 0 {
                simulation.stimulate_sensory_neurons(0.08);
            }
        }

        terminal.draw(|frame| {
            app.draw(frame, &simulation, &worm);
        })?;

        let now = Instant::now();
        if now - last_tick < tick_rate {
            std::thread::sleep(tick_rate - (now - last_tick));
        }
        last_tick = now;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
