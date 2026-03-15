mod config;
mod input;
mod physics;
mod renderer;

use std::path::PathBuf;
use std::time::{Duration, Instant};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use clap::Parser;

use config::Config;
use input::InputReader;
use physics::WaveState;
use renderer::Renderer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable debug mode (shows raw cava bars)
    #[arg(short, long)]
    debug: bool,

    /// Path to a custom config file (optional, confy handles the default)
    #[arg(short, long)]
    config: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let cfg = Config::load(args.config.map(PathBuf::from))?;

    let frame_duration = Duration::from_micros(1_000_000 / cfg.general.framerate);

    let input = InputReader::spawn(cfg.input.driver_bars)?;
    let mut wave = WaveState::new(&cfg);
    let renderer = Renderer::new();

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &input, &mut wave, &renderer, &cfg, frame_duration, args.debug);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_loop(
    terminal: &mut ratatui::Terminal<CrosstermBackend<std::io::Stdout>>,
    input: &InputReader,
    wave: &mut WaveState,
    renderer: &Renderer,
    cfg: &Config,
    frame_duration: Duration,
    debug: bool,
) -> Result<()> {
    let silence = vec![0.0f32; cfg.input.driver_bars];
    let mut latest_frame = silence.clone();

    loop {
        let frame_start = Instant::now();

        let mut had_input = false;
        while let Ok(frame) = input.rx.try_recv() {
            latest_frame = frame.clone();
            wave.update(&frame);
            had_input = true;
        }
        if !had_input {
            wave.update(&silence);
        }

        if debug {
            terminal.draw(|f| renderer.draw_debug(f, &latest_frame))?;
        } else {
            match cfg.output.mode {
                config::Mode::Bars => {
                    terminal.draw(|f| renderer.draw_bars(
                        f, 
                        &wave.points, 
                        &cfg.output.color_mode, 
                        cfg.color1_rgb(), 
                        cfg.color2_rgb(), 
                        cfg.output.visual_gain
                    ))?;
                }
            }
        }

        let elapsed = frame_start.elapsed();
        let poll_time = frame_duration.saturating_sub(elapsed);
        if event::poll(poll_time)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    Ok(())
}
