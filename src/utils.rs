use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use alcro::dialog;
use anyhow::{bail, Context};

pub trait ErrorExt {
    type R;
    fn unwrap_or_die(self, title: &'static str) -> Self::R;
    fn unwrap_or_msg(self, title: &'static str);
}

impl<T, E: std::fmt::Display> ErrorExt for Result<T, E> {
    type R = T;
    fn unwrap_or_die(self, title: &'static str) -> T {
        self.unwrap_or_else(|error| {
            dialog::message_box_ok(
                title,
                &format!("{}:\n\n{}", title, error),
                dialog::MessageBoxIcon::Error,
            );
            log::error!("{}: {}", title, error);
            std::process::exit(-1);
        })
    }
    fn unwrap_or_msg(self, title: &'static str) {
        if let Err(error) = self {
            dialog::message_box_ok(
                title,
                &format!("{}:\n\n{}", title, error),
                dialog::MessageBoxIcon::Error,
            );
            log::error!("{}: {}", title, error);
        }
    }
}

pub fn show_error(msg: &str) {
    log::error!("{}", msg);
    dialog::message_box_ok("Error", msg, dialog::MessageBoxIcon::Error);
}

pub fn data_root() -> Result<PathBuf, app_dirs::AppDirsError> {
    app_dirs::data_root(app_dirs::AppDataType::UserData).map(|path| path.join("ytinu"))
}

pub fn data_root_unwrap() -> PathBuf {
    data_root().unwrap_or_die("Startup error: Failed to get data path")
}

pub fn checksum(path: &Path) -> Result<String, std::io::Error> {
    let mut bytes = Vec::new();
    File::open(path)?.read_to_end(&mut bytes)?;
    Ok(blake2s_simd::blake2s(&bytes).to_hex().to_string())
}

pub fn open_dir(path: &Path) {
    if !path.exists() {
        show_error(&format!(
            "'{}' doesn't exist. You might have to start the game once to generate it.",
            path.to_string_lossy()
        ));
        return;
    }

    let windir = std::env::var_os("WINDIR");
    let windir = match windir.as_ref() {
        Some(val) => Path::new(val),
        None => {
            show_error("Couldn't find explorer: WINDIR is not defined");
            return;
        }
    };

    if let Err(error) = Command::new(windir.join("explorer.exe")).arg(path).spawn() {
        show_error(&format!("Failed to open directory: {}", error));
    }
}

pub fn download_cached(url: &str, name: &str, target: &Path) -> anyhow::Result<Option<PathBuf>> {
    if let Ok(path) = app_dirs::data_root(app_dirs::AppDataType::UserCache) {
        let path = path.join("ytinu").join("cache").join(name);
        if path.exists() {
            log::info!("Found '{}' in cache", url);
        } else {
            download(url, &path)?;
        }
        Ok(Some(path))
    } else {
        download(url, target)?;
        Ok(None)
    }
}

pub fn create_parent_dirs(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create directory '{}'", parent.to_string_lossy())
        })?;
    } else {
        bail!(
            "Failed to get parent directory of: '{}'",
            path.to_string_lossy(),
        );
    }
    Ok(())
}

pub fn download(url: &str, path: &Path) -> anyhow::Result<()> {
    log::info!("Downloading '{}' to '{}'", url, path.to_string_lossy());

    create_parent_dirs(path)?;
    let mut file = File::create(path)
        .with_context(|| format!("Failed to create file: {}", path.to_string_lossy()))?;
    reqwest::blocking::get(url)
        .with_context(|| format!("Failed to download '{}'", url))?
        .copy_to(&mut file)
        .with_context(|| format!("Failed to write to '{}'", path.to_string_lossy()))?;
    Ok(())
}

pub fn unzip(from: &Path, to: &Path) -> anyhow::Result<()> {
    log::info!(
        "Unzipping '{}' to '{}'",
        from.to_string_lossy(),
        to.to_string_lossy()
    );

    create_parent_dirs(to)?;

    let file =
        File::open(from).with_context(|| format!("Failed to read '{}'", from.to_string_lossy()))?;

    zip::ZipArchive::new(&file)
        .with_context(|| format!("Failed to extract '{}'", from.to_string_lossy()))?
        .extract(to)
        .with_context(|| {
            format!(
                "Failed to extract '{}' to '{}'",
                from.to_string_lossy(),
                to.to_string_lossy()
            )
        })
}

pub fn download_cached_and_unzip(
    url: &str,
    name: &str,
    zip_target: &Path,
    unzip_target: &Path,
) -> Result<(), ()> {
    let (is_cached, path) = match crate::utils::download_cached(url, name, zip_target) {
        Ok(Some(path)) => (true, path),
        Ok(None) => (false, zip_target.to_path_buf()),
        Err(error) => {
            crate::show_error(&format!("Failed to download: {:#}", error));
            return Err(());
        }
    };
    if let Err(error) = crate::utils::unzip(&path, unzip_target) {
        crate::show_error(&format!("Failed to extract: {:#}", error));
        return Err(());
    }
    if !is_cached {
        if let Err(error) = std::fs::remove_file(&path) {
            log::error!(
                "Failed to cleanup '{}' after extraction: {}",
                path.to_string_lossy(),
                error
            );
        }
    }
    Ok(())
}

pub fn remove_file_or_dir(path: impl AsRef<Path>) -> Result<(), std::io::Error> {
    let path = path.as_ref();
    if path.is_file() {
        std::fs::remove_file(path)
    } else if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        Ok(())
    }
}
