use crate::backend::DialogFutureType;
use crate::{FileDialog, FileHandle};

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

//
// File Picker (sync)
//

#[cfg(not(target_arch = "wasm32"))]
use crate::backend::FilePickerDialogImpl;
#[cfg(not(target_arch = "wasm32"))]
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        None
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        None
    }
}

//
// File Picker (async)
//

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        Box::pin(async { None })
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        Box::pin(async { None })
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
        None
    }

    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        None
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
        Box::pin(async { None })
    }

    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        Box::pin(async { None })
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
        Box::pin(async { None })
    }
}

