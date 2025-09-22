use crate::backend::DialogFutureType;
use crate::{FileDialog, FileHandle};

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(target_os = "ios")]
use apple_utils::file_type::FileType;
#[cfg(target_os = "ios")]
use apple_utils::ios::FilePicker;

//
// File Picker (sync)
//

#[cfg(not(target_arch = "wasm32"))]
use crate::backend::FilePickerDialogImpl;
#[cfg(not(target_arch = "wasm32"))]
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        #[cfg(target_os = "ios")]
        {
            let picker = build_picker_from_file_dialog(&self, false);
            let mut res = pollster::block_on(picker.open());
            res.into_iter().next()
        }
        #[cfg(not(target_os = "ios"))]
        {
            None
        }
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        #[cfg(target_os = "ios")]
        {
            let picker = build_picker_from_file_dialog(&self, true);
            let res = pollster::block_on(picker.open());
            if res.is_empty() { None } else { Some(res) }
        }
        #[cfg(not(target_os = "ios"))]
        {
            None
        }
    }
}

//
// File Picker (async)
//

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        Box::pin(async move {
            #[cfg(target_os = "ios")]
            {
                let picker = build_picker_from_file_dialog(&self, false);
                let mut paths = picker.open().await;
                paths.into_iter().next().map(FileHandle::wrap)
            }
            #[cfg(not(target_os = "ios"))]
            {
                None
            }
        })
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        Box::pin(async move {
            #[cfg(target_os = "ios")]
            {
                let picker = build_picker_from_file_dialog(&self, true);
                let paths = picker.open().await;
                if paths.is_empty() {
                    None
                } else {
                    Some(paths.into_iter().map(FileHandle::wrap).collect())
                }
            }
            #[cfg(not(target_os = "ios"))]
            {
                None
            }
        })
    }
}

//
// Folder Picker (sync)
//

#[cfg(not(target_arch = "wasm32"))]
use crate::backend::FolderPickerDialogImpl;
#[cfg(not(target_arch = "wasm32"))]
impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        #[cfg(target_os = "ios")]
        {
            let picker = build_folder_picker_from_file_dialog(&self, false);
            let mut res = pollster::block_on(picker.open());
            res.into_iter().next()
        }
        #[cfg(not(target_os = "ios"))]
        {
            None
        }
    }

    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        #[cfg(target_os = "ios")]
        {
            let picker = build_folder_picker_from_file_dialog(&self, true);
            let res = pollster::block_on(picker.open());
            if res.is_empty() { None } else { Some(res) }
        }
        #[cfg(not(target_os = "ios"))]
        {
            None
        }
    }
}

//
// Folder Picker (async)
//

#[cfg(not(target_arch = "wasm32"))]
use crate::backend::AsyncFolderPickerDialogImpl;
#[cfg(not(target_arch = "wasm32"))]
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        Box::pin(async move {
            #[cfg(target_os = "ios")]
            {
                let picker = build_folder_picker_from_file_dialog(&self, false);
                let mut paths = picker.open().await;
                paths.into_iter().next().map(FileHandle::wrap)
            }
            #[cfg(not(target_os = "ios"))]
            {
                None
            }
        })
    }

    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        Box::pin(async move {
            #[cfg(target_os = "ios")]
            {
                let picker = build_folder_picker_from_file_dialog(&self, true);
                let paths = picker.open().await;
                if paths.is_empty() {
                    None
                } else {
                    Some(paths.into_iter().map(FileHandle::wrap).collect())
                }
            }
            #[cfg(not(target_os = "ios"))]
            {
                None
            }
        })
    }
}

//
// File Save (sync)
//

#[cfg(not(target_arch = "wasm32"))]
use crate::backend::FileSaveDialogImpl;
#[cfg(not(target_arch = "wasm32"))]
impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        None
    }
}

//
// File Save (async)
//

use crate::backend::AsyncFileSaveDialogImpl;
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        Box::pin(async move { None })
    }
}

#[cfg(target_os = "ios")]
fn build_picker_from_file_dialog(fd: &FileDialog, multiple: bool) -> FilePicker {
    let mut filters: Vec<FileType> = Vec::new();
    if fd.filters.is_empty() {
        filters.push(FileType::Any);
    } else {
        for f in &fd.filters {
            if f.extensions.is_empty() {
                filters.push(FileType::Any);
            } else {
                for ext in &f.extensions {
                    filters.push(FileType::Extension(ext.clone()));
                }
            }
        }
    }

    FilePicker {
        present_animated: true,
        filters,
        multiple_selection: multiple,
        show_file_extensions: false,
        copy_files: false,
        directory_path: fd.starting_directory.clone(),
    }
}

#[cfg(target_os = "ios")]
fn build_folder_picker_from_file_dialog(fd: &FileDialog, multiple: bool) -> FilePicker {
    let filters = vec![FileType::UniformTypeIdentifier("public.folder".to_string())];

    FilePicker {
        present_animated: true,
        filters,
        multiple_selection: multiple,
        show_file_extensions: false,
        copy_files: false,
        directory_path: fd.starting_directory.clone(),
    }
}

