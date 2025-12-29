use dispatch2::run_on_main;
use objc2_foundation::NSArray;

use super::delegate_future::DelegateFuture;
use super::document_picker;
use super::utils::{filters_to_uttypes, folder_uttypes, get_root_view_controller, path_to_nsurl};
use crate::backend::DialogFutureType;
use crate::{FileDialog, FileHandle};

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let (future, state) = DelegateFuture::new();
        run_on_main(move |mtm| {
            let Some(vc) = get_root_view_controller(mtm) else {
                state.lock().unwrap().complete(None);
                return;
            };
            let types = filters_to_uttypes(&self);
            let picker = document_picker::build_pick_dialog(
                &types,
                false,
                self.starting_directory.as_deref(),
                mtm,
                state,
            );
            document_picker::present(&picker, &vc);
        });
        Box::pin(async move {
            future
                .await
                .and_then(|paths| paths.into_iter().next().map(FileHandle::wrap))
        })
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let (future, state) = DelegateFuture::new();
        run_on_main(move |mtm| {
            let Some(vc) = get_root_view_controller(mtm) else {
                state.lock().unwrap().complete(None);
                return;
            };
            let types = filters_to_uttypes(&self);
            let picker = document_picker::build_pick_dialog(
                &types,
                true,
                self.starting_directory.as_deref(),
                mtm,
                state,
            );
            document_picker::present(&picker, &vc);
        });
        Box::pin(async move {
            future
                .await
                .map(|paths| paths.into_iter().map(FileHandle::wrap).collect())
        })
    }
}

use crate::backend::AsyncFolderPickerDialogImpl;
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        let (future, state) = DelegateFuture::new();
        run_on_main(move |mtm| {
            let Some(vc) = get_root_view_controller(mtm) else {
                state.lock().unwrap().complete(None);
                return;
            };
            let types = folder_uttypes();
            let picker = document_picker::build_pick_dialog(
                &types,
                false,
                self.starting_directory.as_deref(),
                mtm,
                state,
            );
            document_picker::present(&picker, &vc);
        });
        Box::pin(async move {
            future
                .await
                .and_then(|paths| paths.into_iter().next().map(FileHandle::wrap))
        })
    }

    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let (future, state) = DelegateFuture::new();
        run_on_main(move |mtm| {
            let Some(vc) = get_root_view_controller(mtm) else {
                state.lock().unwrap().complete(None);
                return;
            };
            let types = folder_uttypes();
            let picker = document_picker::build_pick_dialog(
                &types,
                true,
                self.starting_directory.as_deref(),
                mtm,
                state,
            );
            document_picker::present(&picker, &vc);
        });
        Box::pin(async move {
            future
                .await
                .map(|paths| paths.into_iter().map(FileHandle::wrap).collect())
        })
    }
}

use crate::backend::AsyncFileSaveDialogImpl;
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let (future, state) = DelegateFuture::new();
        run_on_main(move |mtm| {
            let Some(vc) = get_root_view_controller(mtm) else {
                state.lock().unwrap().complete(None);
                return;
            };
            let file_name = self.file_name.as_deref().unwrap_or("Untitled");
            let temp_path = std::env::temp_dir().join(file_name);
            if std::fs::write(&temp_path, b"").is_err() {
                state.lock().unwrap().complete(None);
                return;
            }
            let Some(url) = path_to_nsurl(&temp_path) else {
                state.lock().unwrap().complete(None);
                return;
            };
            let urls = NSArray::from_retained_slice(&[url]);
            let picker = document_picker::build_export_dialog(
                &urls,
                true,
                self.starting_directory.as_deref(),
                mtm,
                state,
            );
            document_picker::present(&picker, &vc);
        });
        Box::pin(async move {
            future
                .await
                .and_then(|paths| paths.into_iter().next().map(FileHandle::wrap))
        })
    }
}
