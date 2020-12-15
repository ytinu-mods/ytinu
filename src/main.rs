// #![windows_subsystem = "windows"]

use std::{fs::File, path::PathBuf};

use alcro::{dialog, Content, UIBuilder};
use simplelog::{
    CombinedLogger, Config, LevelFilter, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};

mod app;
mod data;
mod utils;
mod server;

pub use app::App;
pub use utils::{data_root, data_root_unwrap, show_error, ErrorExt};

static APP_VERSION: semver::Version = semver::Version {
    major: 1,
    minor: 0,
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
    let log_file = match data_root() {
        Ok(file) => file,
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

fn main() {
    setup_panic_hook();
    setup_logging();

    let app = App::start();
    let (server_handle, port) = server::start(app);

    // let ui = UIBuilder::new()
    //     .content(Content::Url(&format!("http://127.0.0.1:{}/", port)))
    //     .run()
    //     .unwrap_or_die("Startup Error on UIBuilder::run");

    log::info!("Started server on localhost:{}", port);

    // ui.wait_finish();

    log::info!("UI closed. Waiting for server to stop...");

    server_handle.join();
    // server_handle.stop();
}
