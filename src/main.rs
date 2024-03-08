mod app;
mod db;
mod ui;
use rev_buf_reader::RevBufReader;
// mod network; // not ready
//

use app::InputMode;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use db::DbClient;
use ratatui::{
    backend::Backend,
    prelude::{CrosstermBackend, Terminal},
};
use std::{
    alloc::System,
    collections::VecDeque,
    fs::{read, File},
    io::{self, BufRead, Write},
    process::{ChildStdout, Command},
    thread::sleep,
    time::Duration,
};
use std::{
    io::{BufReader, Error, ErrorKind},
    process::Stdio,
};

use sysinfo::Pid;

use tui_input::backend::crossterm::EventHandler;

// this will take over i/o from the terminal:
#[tokio::main]
async fn main() {
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

    let mut app = app::App::new(&args.cmd, &args.db);
    let formatted = format!("running cmd: {:?}...", args.cmd);
    app.update_logs(&formatted);
    app.update_logs("\n");
    app.update_logs("stdout: ");

    // connect to the db:
    let db = db::DbClient::new(&args.db).await;

    // match db.get_locks().await {
    //     Ok(result) => {
    //         let mut res = Vec::new();
    //         result.into_iter().for_each(|r| {
    //             // let thing = r.get(0);
    //             res.push([r.get(0)].to_vec());
    //         });
    //         app.update_db_logs(&res);
    //     }
    //     Err(e) => {
    //         app.update_db_logs(&e.to_string());
    //     }
    // };

    let _res = run_app(&mut terminal, &mut app, &db).await;

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

fn read_from_tail(file: &File) -> Vec<String> {
    let buf = RevBufReader::new(file);
    buf.lines()
        .take(1)
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

// Run a child process and collect logs from STDOUT
// 1. Get the command from the CLI args
// 2. Create a new child process using std::process::Command
// 3. Pipe the output from STDOUT
// 4. Return the process so that logs can be displayed in the app
fn run_task(cmd: &str) -> (Result<BufReader<ChildStdout>, Error>, Result<u32, Error>) {
    // split the cmd up as a string:
    let mut run_cmd = cmd.split(" ").collect::<VecDeque<&str>>();

    // script entry, e.g. "npm"
    let entry = match run_cmd.pop_front() {
        Some(c) => c,
        None => "--version",
    };

    // create a JS file that runs the app as a worker, and dumps ELU to a file:
    let js = format!(
        r#"
        const {{ Worker }} = require('worker_threads');
        const fs = require('node:fs');
        const worker = new Worker('{}');

        setInterval(() => {{
          // Check the worker's usage directly and immediately. The call is thread-safe
          // so it doesn't need to wait for the worker's event loop to become free.
          const elu = worker.performance.eventLoopUtilization();
          const log = `${{elu.utilization}}\n`
          fs.appendFile('elu.log', log, err => {{
              if (err) {{
                console.error(err);
              }} else {{
              // done!
              }}
           }});
        }}, 250);
        "#,
        entry
    );

    let mut worker_file = File::create("worker.js").expect("could not create worker file");
    worker_file
        .write_all(&js.as_bytes())
        .expect("could not write to file");

    let child = Command::new("node")
        .args([
            "--cpu-prof",
            // "--trace-sync-io",
            "--redirect-warnings=./warn.log",
            "worker.js",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn failed, womp womp");
    let id = child.id();
    let stdout = child
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "couldnt capture stdout"));
    let reader = match stdout {
        Ok(s) => BufReader::new(s),
        Err(_) => panic!("stuff"),
    };

    (Ok(reader), Ok(id))
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
    --cmd      STRING           the command to spawn
    --db       STRING           the database connection string

    ARGS:
    <INPUT>
    ";

// todo- convert to async and call cpu_usage ~100-500ms apart!
async fn fetch_ps_info(pid: u32, s: &sysinfo::System) -> (String, f64) {
    // // a call to npm will spawn another node process:
    // todo - just list all processes that are node-based
    // let re = Regex::new(r"node").unwrap();
    // this might be dangerous on some OS's where the pid isn't 32 bit
    if let Some(process) = s.process(Pid::from(pid as usize)) {
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

        let mut buf = String::new();
        buf.push_str(&format!("PID: {}\n", pid.to_string()));
        buf.push_str(&format!(
            "Memory consumption: {} MB",
            process.memory() / (1024 * 1024) // bytes -> kb -> mb)
        ));
        buf.push_str("\n");

        buf.push_str(&format!("CPUs: {}", f32::from(process.cpu_usage()) / 100.0));
        return (buf, f64::from(process.cpu_usage() / 100.0));
    } else {
        return (String::from("still fetching..."), f64::from(0.0));
    }
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut app::App,
    db: &DbClient,
) -> io::Result<bool> {
    // start by running the command in a child process --
    // this returns a BufReader to get output from
    let (reader_res, child_id_res) = run_task(&app.cmd);
    let mut reader = match reader_res {
        Ok(r) => r,
        Err(_) => panic!("cant run command"),
    };

    let child_id = match child_id_res {
        Ok(id) => id,
        Err(_) => panic!("process did not return a valid pid"),
    };

    // Initial monitoring info:
    let mut sys = sysinfo::System::new_all();

    // to-do: this should debounce:
    let (info, cpu) = fetch_ps_info(child_id, &sys).await;
    sys.refresh_all();
    app.update_process(&info);

    let log_file = File::open("elu.log").expect("could not open ELU log");
    let elu = read_from_tail(&log_file);
    elu.into_iter().for_each(|l| {
        let elu = match l.parse::<f64>() {
            Ok(elu) => elu,
            Err(_) => 0.0,
        };
        app.update_elu(elu, cpu)
    });
    // start async event loop:
    app.start();

    loop {
        terminal.draw(|f| ui::paint(f, app))?;

        if let Some(evt) = app.next().await {
            match evt {
                ui::Event::Key(k) => {
                    if k.code == KeyCode::Char('q') {
                        app.should_exit = true;
                    }
                }
                ui::Event::Tick => {
                    match db.get_locks().await {
                        Ok(result) => {
                            let mut res = Vec::new();
                            result.into_iter().for_each(|r| {
                                let wait_event: String = r.get(0);
                                let state: String = r.get(1);
                                let query: String = r.get(2);
                                res.push([wait_event, state, query].to_vec());
                            });
                            app.update_db_logs(res);
                        }
                        Err(e) => {
                            app.update_db_logs(
                                [[
                                    "error".to_string(),
                                    "error".to_string(),
                                    "error".to_string(),
                                ]
                                .to_vec()]
                                .to_vec(),
                            );
                        }
                    };

                    // to-do: this should debounce:
                    sys.refresh_all();
                    let (info, cpu) = fetch_ps_info(child_id, &sys).await;
                    app.update_process(&info);

                    let elu = read_from_tail(&log_file);
                    elu.into_iter().for_each(|l| {
                        let elu = match l.parse::<f64>() {
                            Ok(elu) => elu,
                            Err(_) => 0.5,
                        };
                        app.update_elu(elu, cpu)
                    });
                }
            }
        }

        let mut buf = String::new();
        reader
            .read_line(&mut buf)
            .ok()
            .expect("couldnt read from stdout");

        let logline = format!("{}", buf);
        app.update_logs(&logline);

        // if !logline.is_empty() {
        //     app.update_logs(&logline)
        // }
        //

        if app.should_exit {
            if let Some(process) = sys.process(Pid::from(child_id as usize)) {
                process.kill();
            }
            break;
        }

        // exit criteria -- todo update to ctrl+c:
        // if event::poll(std::time::Duration::from_millis(10)).expect("could not poll") {
        //     // dispatch events:
        //     if let event::Event::Key(key) = event::read().expect("could not read event") {
        //         match app.input_mode {
        //             InputMode::Normal => match key.code {
        //                 KeyCode::Char('q') => {
        //                     break;
        //                 }
        //                 KeyCode::Char('i') => {
        //                     app.input_mode = InputMode::Editing;
        //                 }
        //                 _ => {}
        //             },
        //             InputMode::Editing => match key.code {
        //                 KeyCode::Esc => {
        //                     app.input_mode = InputMode::Normal;
        //                 }
        //                 _ => {
        //                     app.input.handle_event(&Event::Key(key));
        //                 }
        //             },
        //         }
        //     }
        // }
    }
    Ok(true)
}
