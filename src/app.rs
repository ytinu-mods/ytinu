use std::{
    collections::HashMap,
    fs::File,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
};

use alcro::dialog::{self, MessageBoxIcon, YesNo::*};
use anyhow::Context;
use app_dirs::AppDataType;
use rouille::{Request, Response};
use semver::Version;
use serde::de::DeserializeOwned;

use crate::{data::*, ErrorExt, APP_VERSION};

pub static METADATA_URL: &str =
    "https://raw.githubusercontent.com/ytinu-mods/meta/master/meta.json";
pub static GAME_MODS_URL_BASE: &str =
    "https://raw.githubusercontent.com/ytinu-mods/meta/master/games/";

#[cfg(unix)]
pub static BEP_IN_EX_DOWNLOAD_URL: &str =
    "https://github.com/BepInEx/BepInEx/releases/download/v5.4.4/BepInEx_unix_5.4.4.0.zip";
#[cfg(windows)]
pub static BEP_IN_EX_DOWNLOAD_URL: &str =
    "https://github.com/BepInEx/BepInEx/releases/download/v5.4.4/BepInEx_x64_5.4.4.0.zip";
pub static BEP_IN_EX_FILE_NAME: &str = "BepInEx_v5.4.4.0.zip";
pub static BEP_IN_EX_VERSION: Version = Version {
    major: 5,
    minor: 4,
    patch: 4,
    build: Vec::new(),
    pre: Vec::new(),
};

pub struct App {
    data_path: PathBuf,
    metadata: Option<Metadata>,
    state: State,
    config: Config,
}

impl App {
    pub fn start(ui_mode: Option<OpenUIConfig>) -> Arc<Mutex<Self>> {
        let data_path = crate::utils::app_dir(AppDataType::UserData)
            .unwrap_or_die("Startup error: Failed to get data directory")
            .join("data.json");
        log::info!("Using data file at: '{}'", data_path.to_string_lossy());

        let state: State = File::open(&data_path)
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

        let metadata = fetch_metadata();
        let mut config = Config::load();
        if let Some(ui_mode) = ui_mode {
            config.open_ui = ui_mode;
        }

        let mut app = App {
            data_path,
            metadata,
            state,
            config,
        };

        if app.config.check_for_updates {
            app.check_for_updates();
        }
        app.show_messages();
        app.try_ensure_game_selected();
        app.store_state();

        Arc::new(Mutex::new(app))
    }

    pub fn handle(&mut self, path: &str, request: &Request) -> Result<Response, String> {
        match path {
            "find_game_directory" => Ok(Response::json(&self.metadata.as_ref().map(|meta| {
                meta.games
                    .get("Desperados3")
                    .map(Game::find_installation_dir)
            }))),
            "browse_directory" => Ok(Response::json(&alcro::dialog::select_folder_dialog(
                "Browse directory",
                &request.get_param("path").unwrap_or_default(),
            ))),
            "update_install_path" => self
                .update_install_path(parse_request_body(request)?)
                .map(|()| Response::json(&true)),
            "add_game" => self
                .add_game(parse_request_body(request)?)
                .map(|()| Response::json(&true)),
            "state" => Ok(Response::json(&StateOut::new(&self.state))),
            "metadata" => Ok(Response::json(
                &self.metadata.as_ref().map(MetadataOut::new),
            )),
            "toggle_modloader_installed" => {
                self.state
                    .current_game_mut()
                    .map(SetupGame::toggle_modloader_installed);
                self.store_state();
                Ok(Response::empty_204())
            }
            "toggle_modloader_enabled" => {
                self.state
                    .current_game_mut()
                    .map(SetupGame::toggle_modloader_enabled);
                self.store_state();
                Ok(Response::empty_204())
            }
            "update" => {
                self.check_for_updates();
                Ok(Response::empty_204())
            }
            "shutdown" => {
                crate::server::stop();
                Ok(Response::empty_204())
            }
            "get_config" => Ok(Response::json(&self.config)),
            "set_config" => parse_request_body(request)
                .map(|config| {
                    self.config = config;
                    self.config.store();
                })
                .map(|()| Response::json(&true)),
            _ => {
                if let Some(dir) = path.strip_prefix("open/") {
                    self.state.open_dir(dir);
                    Ok(Response::empty_204())
                } else if let Some(mod_id) = path.strip_prefix("install_mod/") {
                    if let Some(m) = self.get_mod(mod_id) {
                        let m = m.clone();
                        if let Some(game) = self.state.current_game_mut() {
                            game.install_mod(m);
                            self.store_state();
                        } else {
                            crate::show_error("No game set up or selected");
                        }
                    } else {
                        crate::show_error(&format!("No mod with id '{}' found", mod_id));
                    }
                    Ok(Response::empty_204())
                } else if let Some(mod_id) = path.strip_prefix("remove_mod/") {
                    if let Some(game) = self.state.current_game_mut() {
                        game.remove_mod(mod_id);
                        self.store_state();
                    } else {
                        crate::show_error("No game set up or selected");
                    }
                    Ok(Response::empty_204())
                } else if let Some(mod_id) = path.strip_prefix("update_mod/") {
                    if let Some(game) = self.state.current_game_mut() {
                        game.update_mod(mod_id);
                        self.store_state();
                    } else {
                        crate::show_error("No game set up or selected");
                    }
                    Ok(Response::empty_204())
                } else {
                    Err(format!("Invalid API endpoint: {}", path))
                }
            }
        }
    }

    fn check_for_updates(&self) {
        if let Some(meta) = self.metadata.as_ref() {
            if meta.version > crate::APP_VERSION {
                log::info!(
                    "Update available. Installed version: {}. Latest version: {}",
                    APP_VERSION,
                    meta.version
                );
                let choice = dialog::message_box_yes_no(
                    "Update available",
                    &format!(
                        "A new version of ytinu is available:\n\n\
                         Installed version: {}\n\
                         Latest version:    {}\n\
                         \n\
                         Do you want to update?",
                        APP_VERSION, meta.version
                    ),
                    MessageBoxIcon::Question,
                    No,
                );

                if choice == Yes {
                    match meta.downloads.get(std::env::consts::OS) {
                        Some(url) => {
                            if let Err(error) = self.install_update(url) {
                                crate::show_error(&format!(
                                    "Failed to install update: {:#}",
                                    error
                                ));
                            }
                        }
                        None => crate::show_error(&format!(
                            "No update url for OS: '{}'",
                            std::env::consts::OS
                        )),
                    }
                }
                return;
            }
        }

        self.remove_old_version();
    }

    fn install_update(&self, url: &str) -> anyhow::Result<()> {
        let exe_path =
            std::env::current_exe().context("Failed to get location of ytinu installation")?;
        let exe_dir = exe_path
            .parent()
            .context("Failed to get directory of ytinu installation")?;

        let tmp_path_new = exe_dir.join("ytinu_new");
        crate::utils::download(url, &tmp_path_new)?;

        let tmp_path_old = exe_dir.join("ytinu_old");
        std::fs::rename(&exe_path, &tmp_path_old).context("Failed to remove current version")?;

        if let Err(error) = std::fs::rename(&tmp_path_new, &exe_path) {
            log::error!(
                "Failed to move new version into place: {}. Trying to restore current version.",
                error
            );
            dialog::message_box_ok(
                "Error during update",
                &format!(
                    "Failed to move new version into place: {}.\n\nTrying to restore current version.",
                    error
                ),
                MessageBoxIcon::Error
            );
            if let Err(error) = std::fs::rename(&tmp_path_old, &exe_path) {
                log::error!(
                    "Failed to restore current version: {}. The backuped version is located at: {}",
                    error,
                    &tmp_path_old.to_string_lossy()
                );
                dialog::message_box_ok(
                    "Error during restore",
                    &format!(
                        "Failed to resture current version: {}\n\n\
                         The backuped version is located at: {}\n\n\
                         You can try to manually replace it with the downloaded latest version at {}.",
                        error,
                        tmp_path_old.to_string_lossy(),
                        tmp_path_new.to_string_lossy()
                    ),
                    MessageBoxIcon::Error,
                );
                std::process::exit(-1);
            }
            if let Err(error) = std::fs::remove_file(&tmp_path_new) {
                log::error!(
                    "Failed to remove downloaded new version at '{}': {}",
                    tmp_path_new.to_string_lossy(),
                    error
                );
            }
        } else {
            #[cfg(not(windows))]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Err(error) = std::fs::metadata(&exe_path).and_then(|meta| {
                    let mut perms = meta.permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&exe_path, perms)
                }) {
                    log::warn!("Sucessfully downloaded and replaced binary but failed to make it executable: {}", error);
                    dialog::message_box_ok("Update partially sucessful", 
                    "Sucessfully downloaded and replaced ytinu but failed to make the new version executable. Please adjust the permissions manually and restart ytinu.", MessageBoxIcon::Info);
                    std::process::exit(-1);
                }
            }
            log::info!("Successfully updated and replaced executable. Restarting...");
            if let Err(error) = Command::new(exe_path).args(std::env::args()).spawn() {
                log::warn!("Failed to start new process: {}", error);
                dialog::message_box_ok("Update sucessful but failed to restart", "The update was sucessfully installed but ytinu was not able to restart itself automatically. Please start it again manually.", MessageBoxIcon::Info);
            }
            std::process::exit(0);
        }

        Ok(())
    }

    pub fn remove_old_version(&self) {
        let run = || -> anyhow::Result<()> {
            let path = std::env::current_exe()
                .context("Failed to get location of ytinu installation")?
                .parent()
                .context("Failed to get directory of ytinu installation")?
                .join("ytinu_old");

            if path.is_file() {
                log::info!(
                    "Found old leftover ytinu executable at '{}'. Removing...",
                    path.to_string_lossy()
                );
                std::fs::remove_file(path).context("Failed to remove file")?;
            }
            Ok(())
        };
        log::info!("Checking for old leftover ytinu executables");
        if let Err(error) = run() {
            log::warn!("Failed to check or remove old ytinu exexcutable: {}", error);
        }
    }

    fn get_mod(&self, id: &str) -> Option<&Mod> {
        if let Some(m) = self.metadata.as_ref()?.mods.get(id) {
            return Some(m);
        }
        self.metadata
            .as_ref()?
            .game_mods
            .get(self.state.selected_game.as_ref()?)
            .and_then(|mods| mods.get(id))
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
            let mut new_game = SetupGame {
                game: game.clone(),
                install_path,
                mods: HashMap::new(),
                bep_in_ex: None,
            };
            new_game.update_modloader_status();
            self.state.games.insert("Desperados3".into(), new_game);
            self.select_game("Desperados3".to_string());
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

    fn try_ensure_game_selected(&mut self) {
        let selected_game = self
            .state
            .selected_game
            .as_ref()
            .and_then(|game_id| self.state.games.get(game_id));

        if let Some(selected_game) = selected_game {
            let id = selected_game.game.id.clone();
            self.select_game(id);
        } else if let Some(game_id) = self.state.games.keys().next().cloned() {
            self.select_game(game_id);
        }
    }

    fn select_game(&mut self, id: String) {
        if let Some(game) = self.state.games.get_mut(&id) {
            self.state.selected_game = Some(id);
            game.update_modloader_status();
            if let Some(meta) = &self.metadata {
                game.update_mods_meta(&meta.mods);
            }
            self.fetch_game_metadata();
        } else {
            self.state.selected_game = None;
        }
    }

    fn fetch_game_metadata(&mut self) {
        log::info!("Fetching game metadata");
        let mut fetch = || {
            let game = self.state.current_game_mut()?;
            let meta = self.metadata.as_mut()?;
            if !meta.game_mods.contains_key(&game.game.id) {
                let game_mods = game.game.fetch_mods()?;
                game.update_mods_meta(&game_mods);
                meta.game_mods.insert(game.game.id.clone(), game_mods);
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
                        No,
                    );
                    if choice == No {
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

    pub fn config(&self) -> &Config {
        &self.config
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.store_state();
        self.config.store();
    }
}

fn parse_request_body<T: DeserializeOwned>(request: &Request) -> Result<T, String> {
    let body = request
        .data()
        .ok_or_else(|| "Missing Request body".to_string())?;
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
