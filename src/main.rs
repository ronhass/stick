use std::env;
use std::io::{stdout, Error, ErrorKind, Result};
use std::process::Command;

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::Flex,
    prelude::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Paragraph},
    Terminal,
};

use clap::Parser;

fn run_command(command: &str) -> Result<()> {
    let shell = match env::var("SHELL") {
        Ok(res) => res,
        _ => {
            return Err(Error::new(
                ErrorKind::Other,
                "Please set $SHELL environemnt variable",
            ))
        }
    };
    let _ = Command::new(shell).arg("-c").arg(command).status();
    Ok(())
}

fn display_choice(command: &str) -> Result<bool> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let first_line = Line::from(vec![
        Span::raw("Command: "),
        Span::styled(command, Style::default().fg(Color::Green)),
    ]);
    let second_line = Line::from(vec![
        Span::raw("<"),
        Span::styled("ENTER", Style::default().fg(Color::Red)),
        Span::raw("> to run, <"),
        Span::styled("q", Style::default().fg(Color::Red)),
        Span::raw("> to quit"),
    ]);

    let text = Text::from(vec![first_line, second_line]);
    let par = Paragraph::new(text).centered();

    let run;
    loop {
        terminal.draw(|frame| {
            frame.render_widget(
                Block::bordered().border_type(BorderType::Rounded),
                frame.size(),
            );
            let [area] = Layout::vertical([Constraint::Length(2)])
                .flex(Flex::Center)
                .areas(frame.size());
            frame.render_widget(par.clone(), area);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    run = false;
                    break;
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                    run = true;
                    break;
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(run)
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short = 'k', long, action)]
    hold: bool,

    command: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.hold {
        run_command(&args.command)?;
    }

    loop {
        if !display_choice(&args.command)? {
            break;
        }
        run_command(&args.command)?;
    }
    Ok(())
}
