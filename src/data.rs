use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use alcro::dialog::{self, MessageBoxIcon, YesNo::*};
use serde::{Deserialize, Serialize};

use crate::{utils::checksum, ErrorExt};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct State {
    pub selected_game: Option<String>,
    pub games: HashMap<String, SetupGame>,
    pub shown_messages: HashSet<String>,
}

impl State {
    pub fn current_game(&self) -> Option<&SetupGame> {
        self.games.get(self.selected_game.as_ref()?)
    }

    pub fn select_game(&mut self) {
        let selected_game = match self.selected_game.as_ref() {
            Some(game) => self.games.contains_key(game),
            None => false,
        };

        if !selected_game {
            self.selected_game = self.games.keys().next().cloned();
        }

        if let Some(selected_game) = self.selected_game.as_ref() {
            if let Some(game) = self.games.get_mut(selected_game) {
                game.update_modloader_status();
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateOut {
    version: semver::Version,
    selected_game: Option<String>,
    games: HashMap<String, SetupGame>,
}

impl StateOut {
    pub fn new(state: &State) -> Self {
        Self {
            version: crate::APP_VERSION.clone(),
            selected_game: state.selected_game.clone(),
            games: state.games.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetupGame {
    pub game: Game,
    pub install_path: String,
    pub mods: HashMap<String, InstalledMod>,
    pub bep_in_ex: Option<BepInExInfo>,
}

impl SetupGame {
    fn update_modloader_status(&mut self) {
        let install_path = Path::new(&self.install_path);
        let bep_in_ex_path = install_path.join("BepInEx");

        if bep_in_ex_path.join("core").join("BepInEx.dll").exists() {
            if let Some(bep_in_ex) = self.bep_in_ex.as_mut() {
                if let Some(expected_hash) = bep_in_ex.hash.as_ref() {
                    let new_hash = checksum(&bep_in_ex_path.join("core").join("BepInEx.dll"));
                    if let Ok(new_hash) = new_hash {
                        if &new_hash != expected_hash {
                            bep_in_ex.version = None;
                            bep_in_ex.hash = Some(new_hash);
                        }
                    }
                }
                let enabled = self.check_bep_in_ex_enabled();
                self.bep_in_ex.as_mut().unwrap().enabled = enabled;
            } else {
                let choice = dialog::message_box_yes_no(
                    "BepInEx detected",
                    "An unknown existing installation of the BepInEx ModLoader was detected. Do you want to remove it?",
                    MessageBoxIcon::Question,
                    Yes
                );
                if choice == No {
                    let bep_in_ex = BepInExInfo {
                        version: None,
                        enabled: self.check_bep_in_ex_enabled(),
                        hash: checksum(&bep_in_ex_path.join("core").join("BepInEx.dll")).ok(),
                    };
                    self.bep_in_ex = Some(bep_in_ex);
                    return;
                }

                let choice = dialog::message_box_yes_no(
                    "Removing BepInEx",
                    "Do you want to keep configs and mods?",
                    MessageBoxIcon::Question,
                    Yes,
                );
                if choice == Yes {
                    std::fs::remove_dir_all(bep_in_ex_path.join("core"))
                        .unwrap_or_msg("Failed to remove 'core' directory");
                    std::fs::remove_dir_all(bep_in_ex_path.join("cache"))
                        .unwrap_or_msg("Failed to remove 'cache' directory");
                } else {
                    std::fs::remove_dir_all(bep_in_ex_path)
                        .unwrap_or_msg("Failed to remove 'BepInEx' directory");
                }
            }
        } else {
            self.bep_in_ex = None;
        }
    }

    fn check_bep_in_ex_enabled(&self) -> bool {
        let install_path = Path::new(&self.install_path);
        install_path.join("doorstop_config.ini").exists()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BepInExInfo {
    version: Option<semver::Version>,
    enabled: bool,
    hash: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstalledMod {
    pub m: Mod,
    pub hash: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Metadata {
    pub version: semver::Version,
    pub messages: Vec<Message>,
    pub games: HashMap<String, Game>,
    pub game_mods: HashMap<String, HashMap<String, Mod>>,
    pub mods: HashMap<String, Mod>,
}

impl From<MetadataIn> for Metadata {
    fn from(meta: MetadataIn) -> Self {
        Self {
            version: meta.version,
            messages: meta.messages,
            games: meta.games.into_iter().map(|g| (g.id.clone(), g)).collect(),
            mods: meta.mods.into_iter().map(|m| (m.id.clone(), m)).collect(),
            game_mods: HashMap::new(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct MetadataOut {
    version: semver::Version,
    update: bool,
    games: HashMap<String, Game>,
    game_mods: HashMap<String, HashMap<String, Mod>>,
    mods: HashMap<String, Mod>,
}

impl MetadataOut {
    pub fn new(meta: &Metadata) -> Self {
        Self {
            version: meta.version.clone(),
            update: meta.version > crate::APP_VERSION,
            games: meta.games.clone(),
            game_mods: meta.game_mods.clone(),
            mods: meta.mods.clone(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct MetadataIn {
    version: semver::Version,
    messages: Vec<Message>,
    games: Vec<Game>,
    mods: Vec<Mod>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub id: String,
    #[serde(default = "semver::VersionReq::any")]
    pub version: semver::VersionReq,
    pub message: String,
    #[serde(default)]
    pub icon: MessageIcon,
    #[serde(default)]
    pub show_always: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageIcon {
    Info,
    Question,
    Error,
    Warning,
}

impl Default for MessageIcon {
    fn default() -> Self {
        Self::Info
    }
}

impl From<MessageIcon> for alcro::dialog::MessageBoxIcon {
    fn from(icon: MessageIcon) -> Self {
        match icon {
            MessageIcon::Info => alcro::dialog::MessageBoxIcon::Info,
            MessageIcon::Question => alcro::dialog::MessageBoxIcon::Question,
            MessageIcon::Error => alcro::dialog::MessageBoxIcon::Error,
            MessageIcon::Warning => alcro::dialog::MessageBoxIcon::Warning,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub appid: Option<String>,
}

impl Game {
    pub fn fetch_mods(&self) -> Option<HashMap<String, Mod>> {
        reqwest::blocking::get(&format!(
            "{}/{}.json",
            crate::app::GAME_MODS_URL_BASE,
            self.id
        ))
        .map_err(|e| log::error!("Failed to get game mods: {}", e))
        .ok()?
        .json::<GameMods>()
        .map_err(|e| log::error!("Failed to parse game mods: {}", e))
        .ok()
        .map(Into::into)
    }

    pub fn find_installation_dir(&self) -> Option<String> {
        let path = if cfg!(windows) {
            Path::new(r"C:\Program Files (x86)\Steam\steamapps\common").join(&self.name)
        } else {
            Path::new("~/.steam/steam/SteamApps/common").join(&self.name)
        };
        if path.is_dir() {
            Some(path.to_string_lossy().to_string())
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mod {
    pub id: String,
    pub name: String,
    pub download: String,
    pub version: semver::Version,
    pub source: Option<String>,
    pub homepage: Option<String>,
    pub description: Option<String>,
    pub ytinu_version: Option<semver::VersionReq>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameMods {
    mods: Vec<Mod>,
}

impl From<GameMods> for HashMap<String, Mod> {
    fn from(game_mods: GameMods) -> Self {
        game_mods
            .mods
            .into_iter()
            .map(|m| (m.id.clone(), m))
            .collect()
    }
}
