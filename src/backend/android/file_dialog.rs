use std::path::PathBuf;

use crate::backend::DialogFutureType;
use crate::{FileDialog, FileHandle};

//
// File Picker (sync)
//

use crate::backend::FilePickerDialogImpl;
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
        Box::pin(std::future::ready(None))
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        Box::pin(std::future::ready(None))
    }
}

//
// Folder Picker (sync)
//

use crate::backend::FolderPickerDialogImpl;
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

use crate::backend::AsyncFolderPickerDialogImpl;
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        Box::pin(std::future::ready(None))
    }

    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        Box::pin(std::future::ready(None))
    }
}

//
// File Save (sync)
//

use crate::backend::FileSaveDialogImpl;
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
        Box::pin(std::future::ready(None))
    }
}

