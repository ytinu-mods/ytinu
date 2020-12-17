use rouille::{Response, Server};
use rust_embed::RustEmbed;
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

use crate::{app::App, ErrorExt};

static STOP_SERVER: AtomicBool = AtomicBool::new(false);

#[derive(RustEmbed)]
#[folder = "svelte/public"]
struct Asset;

fn run(app: Arc<Mutex<App>>, port_tx: std::sync::mpsc::Sender<u16>) {
    let port = if cfg!(debug_assertions) {
        5001
    } else {
        app.lock().unwrap_or_die("App::lock() failed").config().port
    };
    let server = Server::new(("127.0.0.1", port), move |request| {
        let path = request.url();

        log::info!("Request: {}", path);

        let path = if path == "/" {
            "index.html"
        } else if let Some(path) = path.strip_prefix("/api/") {
            let mut app = app.lock().unwrap_or_die("App::lock() failed");
            match app.handle(path, request) {
                Ok(response) => return add_cors(response),
                Err(error) => return add_cors(Response::json(&APIErrorResponse::new(error))),
            }
        } else {
            &path[1..]
        };

        match Asset::get(path) {
            Some(content) => Response::from_data(
                mime_guess::from_path(path)
                    .first_or_octet_stream()
                    .to_string(),
                content,
            ),
            None => Response::html("<h1>Not found</h1>").with_status_code(404),
        }
    })
    .unwrap_or_die("Startup Error on Server::new()");

    let port = server.server_addr().port();
    port_tx
        .send(port)
        .unwrap_or_die("Startup Error on port::send()");

    while !STOP_SERVER.load(Ordering::SeqCst) {
        server.poll();
    }

    log::info!("Server stopped.");
}

#[cfg(debug_assertions)]
fn add_cors(response: Response) -> Response {
    response.with_additional_header("Access-Control-Allow-Origin", "http://localhost:5000")
}

#[cfg(not(debug_assertions))]
fn add_cors(response: Response) -> Response {
    response
}

pub struct ServerHandle(JoinHandle<()>);

pub fn start(app: Arc<Mutex<App>>) -> (ServerHandle, u16) {
    let (port_tx, port_rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || run(app, port_tx));
    let port = port_rx
        .recv()
        .unwrap_or_die("Startup Error on port::recv()");
    (ServerHandle(handle), port)
}

pub fn stop() {
    STOP_SERVER.store(true, Ordering::SeqCst);
}

impl ServerHandle {
    pub fn join(self: ServerHandle) {
        if let Err(error) = self.0.join() {
            log::error!("Error while joining server thread: {:?}", error);
        }
    }
}

#[derive(Serialize)]
struct APIErrorResponse {
    error: String,
}

impl APIErrorResponse {
    fn new(error: String) -> Self {
        Self { error }
    }
}
