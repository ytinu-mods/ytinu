#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs::File,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use alcro::{dialog, Content, UIBuilder};
use app_dirs::AppDataType;
use data::OpenUIConfig;
use server::ServerHandle;
use simplelog::{
    CombinedLogger, Config, LevelFilter, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};

mod app;
mod data;
mod server;
mod utils;

pub use app::App;
pub use utils::{show_error, ErrorExt};

static APP_VERSION: semver::Version = semver::Version {
    major: 0,
    minor: 1,
    patch: 0,
    build: Vec::new(),
    pre: Vec::new(),
};

fn setup_panic_hook() {
    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("Panic: {}", info);
        default_panic_hook(info);
        dialog::message_box_ok(
            "Unexpected Error",
            &format!("An unexpected error occured:\n\n{}", info),
            dialog::MessageBoxIcon::Error,
        );
    }));
}

fn setup_logging() {
    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
    )];
    let mut error = None;
    let log_file = match utils::app_dir(AppDataType::UserData) {
        Ok(file) => file.join("ytinu.log"),
        Err(err) => {
            error = Some(err);
            PathBuf::from("ytinu.log")
        }
    };
    match File::create(&log_file) {
        Ok(file) => loggers.push(WriteLogger::new(LevelFilter::Info, Config::default(), file)),
        Err(error) => eprintln!(
            "Failed to create log file at '{}': {}",
            log_file.to_string_lossy(),
            error
        ),
    }
    if let Err(error) = CombinedLogger::init(loggers) {
        eprintln!("Failed to setup logging: {}", error);
    } else if let Some(error) = error {
        log::error!("Failed to find canonical log file location: {}", error);
    }
}

fn launch_ui(app: Arc<Mutex<App>>, server_handle: ServerHandle, port: u16) {
    let ui_mode = app
        .lock()
        .unwrap_or_die("App::lock() failed")
        .config()
        .open_ui;

    match ui_mode {
        OpenUIConfig::Chromium => {
            let ui = UIBuilder::new()
                .content(Content::Url(&format!("http://127.0.0.1:{}/", port)))
                .size(1200, 720)
                .run()
                .unwrap_or_die("Startup Error on UIBuilder::run");

            ui.wait_finish();
            log::info!("UI closed. Waiting for server to stop...");
            server::stop();
        }
        OpenUIConfig::Browser => {
            if let Err(error) = webbrowser::open(&format!("http://127.0.0.1:{}/", port)) {
                crate::show_error(&format!("Failed to launch browser: {}", error));
            }
        }
        OpenUIConfig::None => (),
    }

    server_handle.join()
}

fn main() {
    setup_panic_hook();
    setup_logging();

    let ui_mode = parse_args();

    let app = App::start(ui_mode);
    let (server_handle, port) = server::start(Arc::clone(&app));

    log::info!("Started server on localhost:{}", port);

    launch_ui(Arc::clone(&app), server_handle, port);

    let app = app.lock();
    if let Ok(app) = app {
        app.remove_old_version();
    }
}

fn parse_args() -> Option<OpenUIConfig> {
    let mut args_iter = std::env::args();
    if args_iter.any(|a| a.as_str() == "--ui") {
        let mode = match args_iter.next().as_deref() {
            Some("chromium") => OpenUIConfig::Chromium,
            Some("browser") => OpenUIConfig::Browser,
            Some("none") => OpenUIConfig::None,
            _ => {
                crate::show_error("Invalid arguments. Usage: ytinu [--ui chromium|browser|none]");
                std::process::exit(-1);
            }
        };
        Some(mode)
    } else {
        None
    }
}
