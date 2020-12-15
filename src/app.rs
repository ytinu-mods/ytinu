use std::{
    collections::HashMap,
    fs::File,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use alcro::dialog::{self, MessageBoxIcon, YesNo};
use rouille::{RequestBody, Response};
use serde::de::DeserializeOwned;

use crate::data::*;

pub const METADATA_URL: &str = "https://raw.githubusercontent.com/ytinu-mods/meta/master/meta.json";
pub const GAME_MODS_URL_BASE: &str =
    "https://raw.githubusercontent.com/ytinu-mods/meta/master/games/";

pub struct App {
    data_path: PathBuf,
    metadata: Option<Metadata>,
    state: State,
}

impl App {
    pub fn start() -> Arc<Mutex<Self>> {
        let data_path = crate::data_root_unwrap().join("data.json");
        log::info!("Using data file at: '{}'", data_path.to_string_lossy());

        let mut state: State = File::open(&data_path)
            .map(|path| {
                serde_json::from_reader(path).unwrap_or_else(|e| {
                    log::error!("Failed to parse data.json: {}", e);
                    Default::default()
                })
            })
            .unwrap_or_else(|e| {
                if data_path.exists() {
                    crate::show_error(&format!("Failed to read data.json: {}", e));
                } else {
                    log::info!("No data.json found");
                }
                Default::default()
            });
        
        state.select_game();

        let metadata = fetch_metadata();
        let mut app = App {
            data_path,
            metadata,
            state,
        };

        app.show_messages();
        app.fetch_game_metadata();
        app.store_state();

        Arc::new(Mutex::new(app))
    }

    pub fn server_port(&self) -> u16 {
        if cfg!(debug_assertions) {
            5001
        } else {
            0
        }
    }

    pub fn handle(&mut self, path: &str, body: Option<RequestBody>) -> Result<Response, String> {
        match path {
            "find_game_directory" => Ok(Response::json(&self.metadata.as_ref().map(|meta| {
                meta.games
                    .get("Desperados3")
                    .map(Game::find_installation_dir)
            }))),
            "browse_directory" => Ok(Response::json(&alcro::dialog::select_folder_dialog(
                "Browse directory",
                "",
            ))),
            "update_install_path" => self
                .update_install_path(parse_request_body(body)?)
                .map(|()| Response::json(&true)),
            "add_game" => self
                .add_game(parse_request_body(body)?)
                .map(|()| Response::json(&true)),
            "state" => Ok(Response::json(&StateOut::new(&self.state))),
            "metadata" => Ok(Response::json(
                &self.metadata.as_ref().map(MetadataOut::new),
            )),
            _ => Err(format!("Invalid API endpoint: {}", path)),
        }
    }

    fn update_install_path(&mut self, install_path: String) -> Result<(), String> {
        let game = match self.state.games.get_mut("Desperados3") {
            Some(game) => game,
            None => return Err("Desperados III is not yet configured".into()),
        };

        let path = std::path::Path::new(&install_path);
        if path.exists() {
            game.install_path = install_path;
            self.store_state();
            Ok(())
        } else {
            Err("Path is invalid or doesn't exist".to_string())
        }
    }

    fn add_game(&mut self, install_path: String) -> Result<(), String> {
        if !self.state.games.is_empty() {
            return Err("Desperados III is already configured".into());
        }

        let path = std::path::Path::new(&install_path);
        if path.exists() {
            let meta = self
                .metadata
                .as_ref()
                .ok_or("No metadata loaded. Can not add new game.")?;
            let game = meta
                .games
                .get("Desperados3")
                .ok_or("No metadata for the game Desperados3 loaded.")?;
            let new_game = SetupGame {
                game: game.clone(),
                install_path,
                mods: HashMap::new(),
                bep_in_ex: None,
            };
            self.state.games.insert("Desperados3".into(), new_game);
            self.state.selected_game = Some("Desperados3".into());
            self.store_state();
            Ok(())
        } else {
            Err("Path is invalid or doesn't exist".to_string())
        }
    }

    fn show_messages(&mut self) {
        if let Some(meta) = &self.metadata {
            for msg in &meta.messages {
                if msg.version.matches(&crate::APP_VERSION)
                    && (self.state.shown_messages.insert(msg.id.clone()) || msg.show_always)
                {
                    alcro::dialog::message_box_ok(&msg.id, &msg.message, msg.icon.into());
                }
            }
        }
    }

    fn fetch_game_metadata(&mut self) {
        let mut fetch = || {
            let game = self.state.current_game()?;
            let meta = self.metadata.as_mut()?;
            if !meta.game_mods.contains_key(&game.game.id) {
                meta.game_mods
                    .insert(game.game.id.clone(), game.game.fetch_mods()?);
            }
            Some(())
        };
        let _ = fetch();
    }

    fn store_state(&self) {
        let backup_file = match self.backup_data() {
            Ok(file) => file,
            Err(()) => return,
        };

        let file = match File::create(&self.data_path) {
            Ok(path) => path,
            Err(error) => {
                if let Some(file) = backup_file {
                    crate::show_error(&format!(
                        "Failed to create data.json: {}. Trying to restore backup.",
                        error
                    ));
                    self.restore_backup(file);
                } else {
                    crate::show_error(&format!("Failed to create data.json: '{}'", error));
                }
                return;
            }
        };
        if let Err(error) = serde_json::to_writer(file, &self.state) {
            if let Some(file) = backup_file {
                crate::show_error(&format!(
                    "Failed to write to data.json: {}. Trying to restore backup.",
                    error
                ));
                self.restore_backup(file);
            } else {
                crate::show_error(&format!("Failed to write to data.json: '{}'", error));
            }
        }
    }

    fn restore_backup(&self, path: PathBuf) {
        if !path.exists() {
            crate::show_error("Backup not found");
            return;
        }
        if let Err(error) = std::fs::rename(&path, &self.data_path) {
            log::error!(
                "Failed to rename backup: {}. Trying to copy it instead.",
                error
            );
            if let Err(error) = std::fs::copy(&path, &self.data_path) {
                log::error!("Failed to copy backup: {}", error);
                dialog::message_box_ok(
                    "Error",
                    &format!("Failed to restore backup: {}", error),
                    MessageBoxIcon::Error,
                );
            }
        }
    }

    fn backup_data(&self) -> Result<Option<PathBuf>, ()> {
        if self.data_path.exists() {
            let target = self.data_path.with_file_name("data.json.bkp");
            if let Err(error) = std::fs::rename(&self.data_path, &target) {
                log::error!(
                    "Failed to backup data.json: {}. Trying to copy it instead.",
                    error
                );
                if let Err(error) = std::fs::copy(&self.data_path, &target) {
                    log::error!("Failed to copy data.json: {}", error);
                    let choice = dialog::message_box_yes_no(
                        "Failed to backup data.json",
                        "Failed to backup data.json. Do you want to try and overwrite it anyway?",
                        MessageBoxIcon::Error,
                        YesNo::No,
                    );
                    if choice == YesNo::No {
                        log::info!("User chose to NOT overwrite data.json");
                        return Err(());
                    }
                    log::info!("User chose to overwrite data.json");
                    return Ok(None);
                }
            }
            return Ok(Some(target));
        }
        Ok(None)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.store_state();
    }
}

fn parse_request_body<T: DeserializeOwned>(body: Option<RequestBody>) -> Result<T, String> {
    let body = body.ok_or_else(|| "Missing Request body".to_string())?;
    serde_json::from_reader(body).map_err(|e| format!("Failed to parse request body: {}", e))
}

fn fetch_metadata() -> Option<Metadata> {
    reqwest::blocking::get(METADATA_URL)
        .map_err(|e| log::error!("Failed to get metadata: {}", e))
        .ok()?
        .json::<MetadataIn>()
        .map_err(|e| log::error!("Failed to parse metadata: {}", e))
        .ok()
        .map(Into::into)
}
