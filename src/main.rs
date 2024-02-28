use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    execute,
    terminal::{self, disable_raw_mode, LeaveAlternateScreen},
    ExecutableCommand,
};
use tokio::time::{sleep, Duration};
extern crate systemstat;
use std::{error::Error, io};
use systemstat::{Platform, System};
use termion::raw::IntoRawMode;
use tui::{backend::Backend, layout::Rect, widgets::Sparkline, Frame};
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
    // if let Ok((total, free)) = sys.memory() {
    //     format!("RAM ⚙ {}%", ((total-free).as_u64()) * 100 / total as u64)
    // } else {
    //      "⚙ _".to_string()
    // }
    if let Ok(mem) = sys.memory() {
        format!(
            "RAM ⚙ {:.2}%",
            (mem.total.as_u64() - mem.free.as_u64()) as f64 * 100.0 / mem.total.as_u64() as f64
        )
    } else {
        "⚙ RAM _".to_string()
    }
}

// fn ram_load(sys: &System) -> Result<u64, Box<dyn std::error::Error>> {
//     match sys.memory() {
//         Ok(mem) => {
//             let used = mem.total - mem.free;
//             Ok((used * 100 / mem.total) as u64) // Convert to percentage and return
//          }
//         Err(e) => Err(Box::new(e)), // Wrap the error in a Box for returning it as Result
//     }
// }

async fn ram_load(sys: &System) -> Result<u64, Box<dyn std::error::Error>> {
    match sys.memory() {
        Ok(mem) => {
            let used = mem.total.as_u64() - mem.free.as_u64(); // Correctly calculate used memory
            let total = mem.total.as_u64();

            let percentage = (used as u64 / total as u64) * 100 as u64;
            println!(
                "Used: {}, Total: {}, Percentage: {}",
                used, total, percentage
            );
            Ok(percentage) // Calculate and return the percentage of used memory
        }
        Err(e) => Err(Box::new(e)), // Wrap the error in a Box for returning it as Result
    }
}

fn draw_ram_usage_sparkline<B: Backend>(f: &mut Frame<B>, area: Rect, ram_usage_history: Vec<u64>) {
    let sparkline = Sparkline::default()
        .block(Block::default().title("RAM Load").borders(Borders::ALL))
        .data(&ram_usage_history)
        .style(Style::default().fg(Color::Green));

    if ram_usage_history.is_empty() {
        let message = vec![Spans::from(Span::raw("No RAM usage data available"))];
        let paragraph = Paragraph::new(message).block(Block::default());
        f.render_widget(paragraph, area);
    } else {
        f.render_widget(sparkline, area);
    }
}

async fn fetch_cpu_load(sys: &System) -> Result<f32, Box<dyn Error>> {
    let cpu_load = sys.cpu_load_aggregate().unwrap();
    // We wait for 1 second to get the CPU load measurement.
    tokio::time::sleep(Duration::from_secs(1)).await;
    let cpu_load = cpu_load.done().unwrap();

    // Here, we return the total CPU load as a percentage.
    // You can adjust this to return user, system, or idle load specifically if preferred.
    Ok(cpu_load.user * 100.0)
}

fn draw_cpu_usage_sparkline<B: Backend>(f: &mut Frame<B>, area: Rect, cpu_usage_history: Vec<u64>) {
    let sparkline = Sparkline::default()
        .block(Block::default().title("CPU Load").borders(Borders::ALL))
        .data(&cpu_usage_history)
        .style(Style::default().fg(Color::Green));
    // If there is no data, show a message
    if cpu_usage_history.is_empty() {
        let message = vec![Spans::from(Span::raw("No CPU usage data available"))];
        let paragraph = Paragraph::new(message).block(Block::default());
        f.render_widget(paragraph, area);
        return;
    } else {
        f.render_widget(sparkline, area);
    }
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
    // let mut last_cpu_load: Option = None;
    let mut cpu_usage_history: Vec<u64> = vec![];
    let mut ram_usage_history: Vec<u64> = vec![];

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
        let cpu_load = fetch_cpu_load(&sys).await?;
        cpu_usage_history.push(cpu_load as u64);
        if cpu_usage_history.len() > 20 {
            cpu_usage_history.remove(0);
        }
        let ram_load = ram_load(&sys).await?; // This will return an error if fetch_cpu_load is unsuccessful
        if (ram_load as u64) < 100 {
            ram_usage_history.push(ram_load);
            println!("RAM Load: {}", ram_load);
        } else {
            eprintln!("Could not get memory stats"); // Handle error if it occurs while fetching RAM load
        }

        // Redraw UI
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical) // The boxes are vertically stacked one on top of each other
                .margin(1)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                ]) // Add a third percentage constraint
                .split(size);

            let current_status = status(&sys);
            let cpu_paragraph = Paragraph::new(current_status).block(
                Block::default()
                    .title("System Status")
                    .borders(Borders::ALL),
            );
            f.render_widget(cpu_paragraph, chunks[0]);
            if ram_usage_history.len() > 1 {
                draw_ram_usage_sparkline(f, chunks[1], ram_usage_history.clone());
            // Render RAM usage sparkline if available data exists
            } else {
                let message = vec![Spans::from(Span::raw("No RAM usage data available"))];
                let paragraph = Paragraph::new(message).block(Block::default());
                f.render_widget(paragraph, chunks[1]); // Render a message if no data exists
            }
            draw_cpu_usage_sparkline(f, chunks[2], cpu_usage_history.clone());
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
