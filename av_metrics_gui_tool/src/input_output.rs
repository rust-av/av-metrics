use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::metrics::MetricsAggregator;

pub enum FileType {
    Y4m,
    FFmpeg,
    Json,
}

impl FileType {
    fn filter_name_extensions(&self) -> (&str, &[&str]) {
        match *self {
            Self::Y4m => ("Y4M", &["y4m"]),
            Self::FFmpeg => ("Multimedia Files", &["mkv", "mp4", "avi"]),
            Self::Json => ("Json", &["json"]),
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn get_path(file_handle: rfd::FileHandle) -> Option<String> {
    Some(file_handle.file_name())
}

#[cfg(not(target_arch = "wasm32"))]
fn get_path(file_handle: rfd::FileHandle) -> Option<String> {
    file_handle
        .path()
        .to_path_buf()
        .into_os_string()
        .into_string()
        .ok()
}

pub fn get_root_path() -> &'static Path {
    Path::new("/")
}

#[cfg(target_arch = "wasm32")]
pub async fn select_file(
    _save: bool,
    filetype: FileType,
    base_dir: Option<PathBuf>,
) -> Option<String> {
    let (filter_name, extensions) = filetype.filter_name_extensions();
    let directory = base_dir.unwrap_or_else(|| get_root_path().to_path_buf());

    let file_handle = rfd::AsyncFileDialog::new()
        .add_filter(filter_name, extensions)
        .set_directory(directory)
        .pick_file()
        .await;

    file_handle.map(get_path).flatten()
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn select_file(
    save: bool,
    filetype: FileType,
    base_dir: Option<PathBuf>,
) -> Option<String> {
    let (filter_name, extensions) = filetype.filter_name_extensions();
    let directory = base_dir.unwrap_or_else(|| get_root_path().to_path_buf());

    let async_dialog = rfd::AsyncFileDialog::new()
        .add_filter(filter_name, extensions)
        .set_directory(directory);

    let file_handle = if save {
        async_dialog.save_file().await
    } else {
        async_dialog.pick_file().await
    };

    file_handle.and_then(get_path)
}

#[cfg(any(not(target_arch = "wasm32"), target_os = "macos"))]
pub fn select_macos_file(
    save: bool,
    filetype: FileType,
    base_dir: Option<PathBuf>,
) -> Option<String> {
    let (filter_name, extensions) = filetype.filter_name_extensions();
    let directory = base_dir.unwrap_or_else(|| get_root_path().to_path_buf());

    let sync_dialog = rfd::FileDialog::new()
        .add_filter(filter_name, extensions)
        .set_directory(directory);

    if save {
        sync_dialog
            .save_file()
            .and_then(|p| p.into_os_string().into_string().ok())
    } else {
        sync_dialog
            .pick_file()
            .and_then(|p| p.into_os_string().into_string().ok())
    }
}

#[cfg(target_arch = "wasm32")]
pub fn select_macos_file(
    _save: bool,
    _filetype: FileType,
    _base_dir: Option<PathBuf>,
) -> Option<String> {
    None
}

#[derive(Debug, Clone)]
pub enum SaveError {
    File,
    Write,
    Format,
}

#[derive(Debug, Clone, Serialize)]
pub struct SavedState {
    pub metrics: MetricsAggregator,
    pub path: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl SavedState {
    pub async fn save(self) -> Result<(), SaveError> {
        use async_std::prelude::*;

        let json = serde_json::to_string_pretty(&self.metrics).map_err(|_| SaveError::Format)?;

        let mut file = async_std::fs::File::create(self.path)
            .await
            .map_err(|_| SaveError::File)?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|_| SaveError::Write)?;

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl SavedState {
    pub async fn save(self) -> Result<(), SaveError> {
        let storage = Self::storage().ok_or(SaveError::File)?;

        let json = serde_json::to_string_pretty(&self.metrics).map_err(|_| SaveError::Format)?;

        storage
            .set_item("state", &json)
            .map_err(|_| SaveError::Write)?;

        Ok(())
    }

    fn storage() -> Option<web_sys::Storage> {
        let window = web_sys::window()?;

        window.local_storage().ok()?
    }
}
