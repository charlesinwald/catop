use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    execute,
    terminal::{self, disable_raw_mode, LeaveAlternateScreen},
    ExecutableCommand,
};
use tokio::time::{sleep, Duration};
extern crate systemstat;
use std::{error::Error, io, os::linux::net};
use sysinfo::System as Sys;
use systemstat::{Platform, System};
use termion::raw::IntoRawMode;
use tui::{
    backend::Backend,
    layout::Rect,
    style::Modifier,
    widgets::{Cell, Gauge, Row, Sparkline, Table},
    Frame,
};
use tui::{
    backend::CrosstermBackend, // Connects `tui` with `crossterm` for terminal backend operations.
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

fn cpu_load(sys: &System) -> String {
    if let Ok(load) = sys.load_average() {
        format!("CPU ⚙ {:.2}%", load.one)
    } else {
        "⚙ _".to_string()
    }
}

fn ram_load_string(sys: &System) -> String {
    if let Ok(mem) = sys.memory() {
        format!(
            "RAM ⚙ {:.2}%",
            (mem.total.as_u64() - mem.free.as_u64()) as f64 * 100.0 / mem.total.as_u64() as f64
        )
    } else {
        "⚙ RAM _".to_string()
    }
}

fn ram_load(sys: &System) -> Result<u64, Box<dyn std::error::Error>> {
    match sys.memory() {
        Ok(mem) => {
            let mut percentage =
                (mem.total.as_u64() - mem.free.as_u64()) as f64 * 100.0 / mem.total.as_u64() as f64;
            let hundred: f64 = 100.0;
            percentage = hundred - percentage; // Invert the percentage to show the used memory
            Ok(percentage as u64) // Cast the result back to u64 if needed
        }
        Err(e) => Err(Box::new(e)),
    }
}

fn draw_ram_usage_gauge<B: Backend>(f: &mut Frame<B>, area: Rect, ram_usage_percentage: u64) {
    let gauge = Gauge::default()
        .block(Block::default().title("RAM Usage").borders(Borders::ALL))
        .gauge_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::ITALIC),
        )
        .percent(ram_usage_percentage as u16);

    f.render_widget(gauge, area);
}

fn fetch_cpu_load(sys: &System) -> Result<f32, Box<dyn Error>> {
    let cpu_load_future = sys.cpu_load_aggregate()?;
    // We wait for 1 second to get the CPU load measurement.
    tokio::time::sleep(Duration::from_secs(1));
    let cpu_load = cpu_load_future.done()?;
    Ok(cpu_load.user * 100.0)
}

fn draw_cpu_usage_gauge<B: Backend>(f: &mut Frame<B>, area: Rect, cpu_usage_percentage: f32) {
    let gauge = Gauge::default()
        .block(Block::default().title("CPU Load").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(cpu_usage_percentage as u16 + 1);

    f.render_widget(gauge, area);
}

fn fetch_processes() -> Vec<(String, String, String, String)> {
    let mut sys = Sys::new_all();
    sys.refresh_all();

    sys.processes()
        .iter()
        .map(|(&pid, process)| {
            (
                pid.to_string(),
                process.name().to_string(),
                format!("{:.2}%", process.cpu_usage()),
                format!("{} KB", process.memory()),
            )
        })
        .collect()
}

fn separated(s: String) -> String {
    if s == "" {
        s
    } else {
        s + " ⸱ "
    }
}

fn status(sys: &System) -> String {
    // separated(plugged(sys)) + &separated(battery(sys)) + &separated(ram(sys)) +
    separated(cpu_load(sys)) + &separated(ram_load_string(sys))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use systemstat::System; // This line is needed to make the render method available in current scope.
    let sys = System::new();
    let mut stdout = io::stdout().into_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut stdout = io::stdout().into_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    terminal.backend_mut().execute(Hide)?;

    'mainloop: loop {
        // Check for keyboard events
        if event::poll(Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        // Quit the application
                        break 'mainloop;
                    }
                    _ => {} // Handle other keys here
                }
            }
        }
        let cpu_load = fetch_cpu_load(&sys);

        // Redraw UI
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical) // The boxes are vertically stacked one on top of each other
                .margin(1)
                .constraints([
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(70),
                ]) // Add a third percentage constraint
                .split(size);

            let current_status = status(&sys);
            // let system_stats_paragraph = Paragraph::new(format!("{}", current_status))
            //     .block(Block::default().title("System Stats").borders(Borders::ALL));
            // f.render_widget(system_stats_paragraph, chunks[0]);
            draw_cpu_usage_gauge(f, chunks[0], cpu_load.unwrap());
            let ram_load = ram_load(&sys); // This will return an error if fetch_cpu_load is unsuccessful
            let ram_load_value = 100 - ram_load.unwrap() as u64;
            if (ram_load_value > 0) {
                draw_ram_usage_gauge(f, chunks[1], ram_load_value);
            // Render RAM usage sparkline if available data exists
            } else {
                let message = vec![Spans::from(Span::raw("No RAM usage data available"))];
                let paragraph = Paragraph::new(message).block(Block::default());
                f.render_widget(paragraph, chunks[1]); // Render a message if no data exists
            }
            let processes_data = fetch_processes();

            let rows: Vec<Row> = processes_data
                .into_iter()
                .map(|(pid, name, cpu, mem)| {
                    // Create a row for each process
                    Row::new(vec![
                        Cell::from(pid),
                        Cell::from(name),
                        Cell::from(cpu),
                        Cell::from(mem),
                    ])
                })
                .collect();

            let table = Table::new(rows)
                .block(Block::default().title("Processes").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::LightGreen)) // Highlight style is optional
                .widths(&[
                    Constraint::Length(10), // PID
                    Constraint::Length(20), // Process Name
                    Constraint::Length(10), // CPU Usage
                    Constraint::Length(10), // Memory Usage
                ]);
            f.render_widget(table, chunks[3]);
        })?;

        sleep(Duration::from_millis(100)).await; // Sleep to throttle the loop
    }
    // Cleanup before exiting
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), Show, LeaveAlternateScreen)?;
    // Cleanup before exiting
    terminal::disable_raw_mode()?;
    terminal.backend_mut().execute(Show)?;
    terminal
        .backend_mut()
        .execute(terminal::LeaveAlternateScreen)?;
    Ok(())
}
