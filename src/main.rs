mod app;
mod ui;

use crate::ui::ui;

use app::InputMode;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::Backend,
    prelude::{CrosstermBackend, Terminal},
};
use std::{
    io::{self, BufRead},
    process::{ChildStdout, Command},
};
use std::{
    io::{BufReader, Error, ErrorKind},
    process::Stdio,
};

use tui_input::backend::crossterm::EventHandler;


// this will take over i/o from the terminal:
fn main() {
    let mut stdout = io::stdout();
    enable_raw_mode().expect("could not use terminal");

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).expect("couldnt use crossterm");

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend).expect("could not use stdout");
    terminal.clear().expect("could not clear");

    // get CLI args first:
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    let mut app = app::App::new(&args.cmd);
    let _res = run_app(&mut terminal, &mut app);

    let formatted = format!("{:?}", args);
    app.update_logs(&formatted);

    disable_raw_mode().expect("raw mode not allowed");
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .expect("cant return to normal state");
    terminal.show_cursor().expect("cant show cursor");
}

// Parse options passed to HeyNode:
fn parse_args() -> Result<RunnerArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = RunnerArgs {
        cmd: pargs.value_from_str("--cmd")?,
        db: pargs.value_from_str("--db")?,
    };
    let stuff = pargs.finish();
    if !stuff.is_empty() {}
    Ok(args)
}

// Run a child process and collect logs from STDOUT
// 1. Get the command from the CLI args
// 2. Create a new child process using std::process::Command
// 3. Pipe the output from STDIN
// 4. Return the process so that logs can be displayed in the app
fn run_task(cmd: &str) -> Result<BufReader<ChildStdout>, Error> {
    let stdout = Command::new(cmd)
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "couldnt capture stdout"));
    let reader = match stdout {
        Ok(s) => BufReader::new(s),
        Err(_) => panic!("stuff"),
    };
    // reader.lines().filter_map(|l| l.ok());

    Ok(reader)
}

#[derive(Debug)]
pub struct RunnerArgs {
    cmd: String,
    db: String,
}

pub enum Database {
    Postgres,
    Mysql,
    Mssql,
}

// help text:
const HELP: &str = "\
    App

    USAGE:
    hey --cmd \"START_CMD_IN_QUOTES\"

    FLAGS:
    -h, --help            Prints help information

    OPTIONS:
    --cmd      PATH           the command to spawn
    --db       PATH           the type of database to inspect traffic on

    ARGS:
    <INPUT>
    ";

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut app::App) -> io::Result<bool> {
    // start by running the command in a child process --
    // this returns a BufReader to get output from
    let reader = match run_task(&app.cmd) {
        Ok(r) => r,
        Err(_) => panic!("cant run command"),
    };

    for line in reader.lines() {
        let log = match line {
            Ok(l) => l,
            Err(_) => String::from("err1"),
        };
        app.update_logs(&log)
    }
    loop {
        terminal.draw(|f| ui(f, app))?;

        // exit criteria -- todo update to ctrl+c:
        if event::poll(std::time::Duration::from_millis(10)).expect("could not poll") {
            // dispatch events:
            if let event::Event::Key(key) = event::read().expect("could not read event") {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Char('i') => {
                            app.input_mode = InputMode::Editing;
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {
                            app.input.handle_event(&Event::Key(key));
                        }
                    },
                }
            }
        }
    }
    Ok(true)
}
