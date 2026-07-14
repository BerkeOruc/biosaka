mod connectome;
mod simulation;
mod worm;
mod tui;
mod generated;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::time::{Duration, Instant};

use connectome::Connectome;
use simulation::Simulation;
use worm::Worm;
use tui::App;

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let connectome = Connectome::load();
    let mut simulation = Simulation::new(connectome);
    let mut worm = Worm::new();
    let mut app = App::new(&simulation.connectome);

    let tick_rate = Duration::from_millis(33);
    let mut last_tick = Instant::now();

    simulation.stimulate_sensory_neurons(0.15);

    while app.running {
        app.handle_input()?;

        if !app.paused {
            for _ in 0..3 {
                simulation.step();
            }
            worm.update(&simulation);

            if simulation.time as u64 % 100 == 0 {
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
