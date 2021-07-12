use std::path::PathBuf;

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

pub async fn select_file(
    save: bool,
    filetype: FileType,
    base_dir: Option<PathBuf>,
) -> Option<String> {
    let (filter_name, extensions) = filetype.filter_name_extensions();
    let mut dialog = native_dialog::FileDialog::new().add_filter(filter_name, extensions);
    if let Some(p) = &base_dir {
        dialog = dialog.set_location(p);
    }

    let res = if save {
        dialog.show_save_single_file()
    } else {
        dialog.show_open_single_file()
    };

    res.map_or(None, |p| {
        p.map(|path| path.into_os_string().into_string().unwrap_or_default())
    })
}

#[derive(Debug, Clone)]
pub enum SaveError {
    FileError,
    WriteError,
    FormatError,
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

        let json =
            serde_json::to_string_pretty(&self.metrics).map_err(|_| SaveError::FormatError)?;

        let mut file = async_std::fs::File::create(self.path)
            .await
            .map_err(|_| SaveError::FileError)?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|_| SaveError::WriteError)?;

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl SavedState {
    async fn save(self) -> Result<(), SaveError> {
        let window = web_sys::window().map_err(|_| SaveError::FileError)?;

        let storage = window.local_storage().map_err(|_| SaveError::FileError)?;

        let json =
            serde_json::to_string_pretty(&self.metrics).map_err(|_| SaveError::FormatError)?;

        storage
            .set_item("state", &json)
            .map_err(|_| SaveError::WriteError)?;

        Ok(())
    }
}
