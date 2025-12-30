use super::jni_future::JniFuture;
use super::utils::{
    call_pick_file, call_pick_folder, call_save_file, filters_to_mime_types, get_first_mime_type,
    register_file_request, with_jni_helper,
};
use crate::backend::DialogFutureType;
use crate::{FileDialog, FileHandle};

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let (future, state) = JniFuture::new();
        let request_id = register_file_request(state.clone());
        let launched = with_jni_helper(false, |env, helper| {
            let mime_types = filters_to_mime_types(&self);
            call_pick_file(env, helper, request_id, &mime_types, false)?;
            Ok(true)
        });
        if !launched {
            state.lock().unwrap().complete(None);
        }
        Box::pin(async move {
            future
                .await
                .and_then(|paths| paths.into_iter().next().map(FileHandle::wrap))
        })
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let (future, state) = JniFuture::new();
        let request_id = register_file_request(state.clone());
        let launched = with_jni_helper(false, |env, helper| {
            let mime_types = filters_to_mime_types(&self);
            call_pick_file(env, helper, request_id, &mime_types, true)?;
            Ok(true)
        });
        if !launched {
            state.lock().unwrap().complete(None);
        }
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
        let (future, state) = JniFuture::new();
        let request_id = register_file_request(state.clone());
        let launched = with_jni_helper(false, |env, helper| {
            call_pick_folder(env, helper, request_id)?;
            Ok(true)
        });
        if !launched {
            state.lock().unwrap().complete(None);
        }
        Box::pin(async move {
            future
                .await
                .and_then(|paths| paths.into_iter().next().map(FileHandle::wrap))
        })
    }

    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let (future, state) = JniFuture::new();
        let request_id = register_file_request(state.clone());
        let launched = with_jni_helper(false, |env, helper| {
            call_pick_folder(env, helper, request_id)?;
            Ok(true)
        });
        if !launched {
            state.lock().unwrap().complete(None);
        }
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
        let (future, state) = JniFuture::new();
        let request_id = register_file_request(state.clone());
        let launched = with_jni_helper(false, |env, helper| {
            let mime_type = get_first_mime_type(&self);
            let file_name = self.file_name.as_deref().unwrap_or("Untitled");
            call_save_file(env, helper, request_id, &mime_type, file_name)?;
            Ok(true)
        });
        if !launched {
            state.lock().unwrap().complete(None);
        }
        Box::pin(async move {
            future
                .await
                .and_then(|paths| paths.into_iter().next().map(FileHandle::wrap))
        })
    }
}
