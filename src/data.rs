use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::{Path, PathBuf},
};

use alcro::dialog::{self, MessageBoxIcon, YesNo::*};
use anyhow::{bail, ensure, Context};
use app_dirs::AppDataType;
use semver::Version;
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

    pub fn current_game_mut(&mut self) -> Option<&mut SetupGame> {
        self.games.get_mut(self.selected_game.as_ref()?)
    }

    pub fn open_dir(&self, dir: &str) {
        if let Some(game) = self.current_game() {
            match dir {
                "game" => crate::utils::open_dir(game.install_path()),
                "mods" => {
                    crate::utils::open_dir(&game.install_path().join("BepInEx").join("plugins"))
                }
                "config" => {
                    crate::utils::open_dir(&game.install_path().join("BepInEx").join("configs"))
                }
                _ => crate::show_error(&format!("Unknown directory: {}", dir)),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateOut {
    version: semver::Version,
    selected_game: Option<String>,
    games: HashMap<String, SetupGame>,
    os: &'static str,
}

impl StateOut {
    pub fn new(state: &State) -> Self {
        Self {
            version: crate::APP_VERSION.clone(),
            selected_game: state.selected_game.clone(),
            games: state.games.clone(),
            os: std::env::consts::OS,
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
    pub fn install_path(&self) -> &Path {
        Path::new(&self.install_path)
    }

    pub fn plugins_path(&self) -> PathBuf {
        self.install_path().join("BepInEx").join("plugins")
    }

    pub fn update_mods_meta(&mut self, new_mods: &HashMap<String, Mod>) {
        for (id, m) in self.mods.iter_mut() {
            if let Some(new_m) = new_mods.get(id) {
                m.m = new_m.clone();
            }
        }
    }

    pub fn install_mod(&mut self, m: Mod) {
        if let Err(error) = self.install_mod_impl(m) {
            crate::show_error(&format!("{:#}", error));
        }
    }

    pub fn install_mod_impl(&mut self, m: Mod) -> anyhow::Result<()> {
        ensure!(!self.mods.contains_key(&m.id), "Mod already installed");

        let url = reqwest::Url::parse(&m.download)
            .with_context(|| format!("Invalid download url: '{}'", m.download))?;
        let file = url
            .path_segments()
            .and_then(|ps| ps.last())
            .with_context(|| format!("Invalid download url (no file): '{}'", m.download))?;

        if file.ends_with(".dll") {
            crate::utils::download(
                &m.download,
                &self.plugins_path().join(&format!("{}.dll", m.id)),
            )
            .context("Failed to download mod")?;
        } else if file.ends_with(".zip") {
            let fname = format!("{}_{}.zip", self.game.id, m.id);
            let target_dir = if m.extract_to_root {
                self.install_path().to_path_buf()
            } else {
                self.plugins_path().join(&m.id)
            };
            if crate::utils::download_cached_and_unzip(
                &m.download,
                &fname,
                &self.plugins_path().join(&fname),
                &target_dir,
            )
            .is_err()
            {
                return Ok(());
            }
        } else {
            bail!(
                "Unrecognized file type in download URL: {}\nValid types are only .zip and .dll",
                m.download
            );
        }

        self.mods.insert(
            m.id.clone(),
            InstalledMod {
                version: m.version.clone(),
                enabled: true,
                m,
            },
        );

        Ok(())
    }

    pub fn update_mod(&mut self, mod_id: &str) {
        match self.remove_mod_impl(mod_id) {
            Ok(m) => self.install_mod(m),
            Err(error) => crate::show_error(&format!("{:#}", error)),
        }
    }

    pub fn remove_mod(&mut self, mod_id: &str) {
        if let Err(error) = self.remove_mod_impl(mod_id) {
            crate::show_error(&format!("{:#}", error));
        }
    }

    fn remove_mod_impl(&mut self, mod_id: &str) -> anyhow::Result<Mod> {
        let m = self.mods.get(mod_id).context("Mod is not installed")?;

        if let Some(files) = &m.m.files {
            for file in files {
                let path = self.install_path().join(&file);
                crate::utils::remove_file_or_dir(path)
                    .with_context(|| format!("Failed to remove '{}'", file))?;
            }
        } else {
            let dirs = self
                .plugins_path()
                .read_dir()
                .context("Failed to list files in plugins directory")?;
            for entry in dirs {
                let entry = entry.context("Failed to list files in plugins directory")?;
                let fname = entry.file_name();
                let fname = fname.to_string_lossy();
                if fname.starts_with(mod_id) {
                    crate::utils::remove_file_or_dir(entry.path())
                        .with_context(|| format!("Failed to remove '{}'", fname))?;
                }
            }
        }

        Ok(self.mods.remove(mod_id).unwrap().m)
    }

    pub fn toggle_modloader_installed(&mut self) {
        if self.bep_in_ex.is_some() {
            let choice = dialog::message_box_yes_no(
                "Are you sure?",
                "Are you sure?\n\
                 This will remove all mods and all stored configuration.\n\
                 You can disable the Mod Loader instead if you just want to start the games without loading any mods.",
                MessageBoxIcon::Question,
                No
            );
            if choice == No {
                return;
            }
            let path = self.install_path().join("BepInEx");
            if path.is_dir() {
                std::fs::remove_dir_all(path).unwrap_or_msg("Failed to remove BepInEx directory");
            }
            let path = self.install_path().join("doorstop_config.ini");
            if path.is_file() {
                std::fs::remove_file(path).unwrap_or_msg("Failed to remove doorstop_config.ini");
            }
            let path = self.install_path().join("winhttp.dll");
            if path.is_file() {
                std::fs::remove_file(path).unwrap_or_msg("Failed to remove winhttp.dll");
            }
            self.mods.clear();
            self.bep_in_ex = None;
        } else {
            let target = self.install_path().join(crate::app::BEP_IN_EX_FILE_NAME);
            if crate::utils::download_cached_and_unzip(
                crate::app::BEP_IN_EX_DOWNLOAD_URL,
                crate::app::BEP_IN_EX_FILE_NAME,
                &target,
                self.install_path(),
            )
            .is_err()
            {
                return;
            }

            self.bep_in_ex = Some(BepInExInfo {
                enabled: true,
                version: Some(crate::app::BEP_IN_EX_VERSION.clone()),
                hash: crate::utils::checksum(
                    &self
                        .install_path()
                        .join("BepInEx")
                        .join("core")
                        .join("BepInEx.dll"),
                )
                .ok(),
            });
        }
    }

    pub fn toggle_modloader_enabled(&mut self) {
        if let Some(bep_in_ex) = &mut self.bep_in_ex {
            let install_path = Path::new(&self.install_path);
            let ini = install_path.join("doorstop_config.ini");
            let ini_target = install_path.join("BepInEx").join("doorstop_config.ini");
            let dll = install_path.join("winhttp.dll");
            let dll_target = install_path.join("BepInEx").join("winhttp.dll");

            if bep_in_ex.enabled {
                if let Err(error) = std::fs::rename(ini, ini_target) {
                    crate::show_error(&format!("Failed to move 'doorstop_config.ini': {}", error));
                    return;
                }
                if let Err(error) = std::fs::rename(dll, dll_target) {
                    crate::show_error(&format!("Failed to move 'winhttp.dll': {}", error));
                    return;
                }
                bep_in_ex.enabled = false;
            } else {
                if let Err(error) = std::fs::rename(ini_target, ini) {
                    crate::show_error(&format!("Failed to move 'doorstop_config.ini': {}", error));
                    return;
                }
                if let Err(error) = std::fs::rename(dll_target, dll) {
                    crate::show_error(&format!("Failed to move 'winhttp.dll': {}", error));
                    return;
                }
                bep_in_ex.enabled = true;
            }
        } else {
            crate::show_error(
                "Tried to enable Mod Loader but BepInEx is not installed for this game.",
            );
        }
    }

    pub fn update_modloader_status(&mut self) {
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
    pub version: Version,
    pub enabled: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Metadata {
    pub version: semver::Version,
    pub downloads: HashMap<String, String>,
    pub messages: Vec<Message>,
    pub games: HashMap<String, Game>,
    pub game_mods: HashMap<String, HashMap<String, Mod>>,
    pub mods: HashMap<String, Mod>,
}

impl From<MetadataIn> for Metadata {
    fn from(meta: MetadataIn) -> Self {
        Self {
            version: meta.version,
            downloads: meta.downloads,
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
    downloads: HashMap<String, String>,
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
    pub recommended_mods: Vec<String>,
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
    #[serde(default)]
    pub extract_to_root: bool,
    pub files: Option<Vec<String>>,
    #[serde(default)]
    pub dev_mod: bool,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub dark_mode: DarkMode,
    pub show_dev_mods: bool,
    pub port: u16,
    pub check_for_updates: bool,
    pub open_ui: OpenUIConfig,
}

impl Config {
    pub fn load() -> Self {
        match Self::load_impl() {
            Ok(config) => config,
            Err(error) => {
                crate::show_error(&format!("Failed to load config file: {:#}", error));
                Self::default()
            }
        }
    }

    fn path() -> PathBuf {
        crate::utils::app_dir(AppDataType::UserConfig)
            .unwrap_or_die("Startup error: Failed to get config directory")
            .join("config.json")
    }

    fn load_impl() -> anyhow::Result<Self> {
        let path = Config::path();
        if path.is_file() {
            let file = File::open(path).context("Failed to open config file")?;
            serde_json::from_reader(file).context("Failed to deserialize config file")
        } else {
            Ok(Self::default())
        }
    }

    pub fn store(&self) {
        if let Err(error) = self.store_impl() {
            crate::show_error(&format!("Failed to save config file: {:#}", error));
        }
    }

    fn store_impl(&self) -> anyhow::Result<()> {
        let file = File::create(Config::path()).context("Failed to create config file")?;
        serde_json::to_writer(file, self).context("Failed to serialize config file")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dark_mode: DarkMode::System,
            show_dev_mods: false,
            port: 0,
            check_for_updates: true,
            open_ui: OpenUIConfig::Chromium,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum DarkMode {
    System,
    Dark,
    Light,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum OpenUIConfig {
    Chromium,
    Browser,
    None,
}
