use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use alcro::dialog;

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
