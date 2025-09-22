use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::backend::DialogFutureType;
use crate::{FileDialog, FileHandle};

use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jint, jobjectArray};
use jni::JNIEnv;
use ndk_context::android_context;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static REQUEST_COUNTER: AtomicI32 = AtomicI32::new(41000);

struct PendingEntry {
    sender: crate::oneshot::Sender<Option<Vec<String>>>,
}

static PENDING: Lazy<Mutex<HashMap<i32, PendingEntry>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn with_env<F, T>(f: F) -> T
where
    F: FnOnce(&mut JNIEnv<'_>, JObject<'_>) -> T,
{
    let ctx = android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.expect("Invalid JavaVM");
    let mut env = vm.attach_current_thread().expect("Attach current thread failed");
    let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };
    f(&mut env, activity)
}

fn start_open_document_request(multiple: bool, title: Option<&str>) -> Option<i32> {
    let req = REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst);

    with_env(|env, activity| {
        // Prepare arguments
        let jtitle = title.and_then(|t| env.new_string(t).ok());
        let jtitle_val = match jtitle {
            Some(s) => JValue::Object(&s),
            None => JValue::Object(&JObject::null()),
        };

        let bridge = env
            .find_class("io/github/polymeilex/rfd/RfdBridge")
            .and_then(|cls| {
                env.call_static_method(
                    cls,
                    "openDocument",
                    "(Landroid/content/Context;[Ljava/lang/String;ZLjava/lang/String;I)V",
                    &[
                        JValue::Object(&activity),
                        JValue::Object(&JObject::null()),
                        JValue::Bool(multiple as u8),
                        jtitle_val,
                        JValue::Int(req),
                    ],
                )
            });

        match bridge {
            Ok(_) => Some(req),
            Err(_) => None,
        }
    })
}

fn copy_uris_to_cache(uris: &[String]) -> Option<Vec<PathBuf>> {
    with_env(|env, activity| {
        let string_class = env.find_class("java/lang/String").ok()?;
        let arr = env
            .new_object_array(uris.len() as i32, string_class, JObject::null())
            .ok()?;
        for (i, s) in uris.iter().enumerate() {
            let js: JString = env.new_string(s).ok()?;
            env.set_object_array_element(arr, i as i32, js).ok()?;
        }
        let res = env
            .call_static_method(
                "io/github/polymeilex/rfd/RfdBridge",
                "copyUrisToCache",
                "(Landroid/content/Context;[Ljava/lang/String;)[Ljava/lang/String;",
                &[JValue::Object(&activity), JValue::Object(&arr)],
            )
            .ok()
            .and_then(|v| v.l().ok());

        let Some(jarr_obj) = res else { return None; };
        if jarr_obj.is_null() { return None; }
        let jarr = jarr_obj.into_inner();
        let len = env.get_array_length(jarr).unwrap_or(0);
        let mut out = Vec::new();
        for i in 0..len {
            if let Ok(elt) = env.get_object_array_element(jarr, i) {
                if elt.is_null() { continue; }
                let s: String = env.get_string(&JString::from(elt)).unwrap().into();
                out.push(PathBuf::from(s));
            }
        }
        Some(out)
    })
}

#[no_mangle]
pub extern "system" fn Java_io_github_polymeilex_rfd_RfdBridge_onActivityResultCallback(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    request_code: jint,
    result_code: jint,
    uris: jobjectArray,
) {
    let mut vec_uris: Option<Vec<String>> = None;

    // RESULT_OK = -1
    if result_code == -1 && !uris.is_null() {
        let len = env.get_array_length(uris).unwrap_or(0);
        let mut tmp = Vec::new();
        for i in 0..len {
            if let Ok(elt) = env.get_object_array_element(uris, i) {
                if !elt.is_null() {
                    let s: String = env.get_string(&JString::from(elt)).unwrap().into();
                    tmp.push(s);
                }
            }
        }
        vec_uris = Some(tmp);
    }

    let maybe_sender = {
        let mut lock = PENDING.lock().unwrap();
        lock.remove(&(request_code as i32))
    };
    if let Some(entry) = maybe_sender {
        let _ = entry.sender.send(vec_uris);
    }
}

fn pick_files_android_multiple(filters: &[crate::file_dialog::Filter], allow_multiple: bool, title: Option<&str>) -> Option<Vec<PathBuf>> {
    with_env(|env, activity| {
        let req = REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst);

        let jtitle = title.and_then(|t| env.new_string(t).ok());
        let jtitle_val = match jtitle {
            Some(s) => JValue::Object(&s),
            None => JValue::Object(&JObject::null()),
        };

        // Build MIME types from extensions
        let mut mimes: Vec<String> = Vec::new();
        for f in filters.iter() {
            for ext in f.extensions.iter() {
                let ext = ext.trim().trim_start_matches('.').to_ascii_lowercase();
                if ext.is_empty() || ext == "*" { continue; }
                if let Ok(js) = env.new_string(&ext) {
                    if let Ok(m) = env.call_static_method(
                        "android/webkit/MimeTypeMap",
                        "getSingleton",
                        "()Landroid/webkit/MimeTypeMap;",
                        &[],
                    ) {
                        if let Ok(map) = m.l() {
                            if let Ok(jm) = env.call_method(
                                map,
                                "getMimeTypeFromExtension",
                                "(Ljava/lang/String;)Ljava/lang/String;",
                                &[JValue::Object(&js)],
                            ) {
                                if let Ok(obj) = jm.l() {
                                    if !obj.is_null() {
                                        if let Ok(rs) = env.get_string(&JString::from(obj)) {
                                            let s: String = rs.into();
                                            if !s.is_empty() {
                                                mimes.push(s);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if mimes.is_empty() {
            mimes.push("*/*".to_string());
        }

        // Convert MIME types to Java String[]
        let string_class = env.find_class("java/lang/String").ok()?;
        let jarr = env
            .new_object_array(mimes.len() as i32, string_class, JObject::null())
            .ok()?;
        for (i, s) in mimes.iter().enumerate() {
            let js: JString = env.new_string(s).ok()?;
            env.set_object_array_element(jarr, i as i32, js).ok()?;
        }

        // Launch picker
        env.call_static_method(
            "io/github/polymeilex/rfd/RfdBridge",
            "openDocument",
            "(Landroid/content/Context;[Ljava/lang/String;ZLjava/lang/String;I)V",
            &[
                JValue::Object(&activity),
                JValue::Object(&jarr),
                JValue::Bool(allow_multiple as u8),
                jtitle_val,
                JValue::Int(req),
            ],
        ).ok()?;

        // Register pending
        let (tx, rx) = crate::oneshot::channel();
        {
            let mut lock = PENDING.lock().unwrap();
            lock.insert(req, PendingEntry { sender: tx });
        }

        drop(activity);
        drop(env);

        let uris_opt = pollster::block_on(async move { rx.await.ok().flatten() });
        let uris = uris_opt?;
        copy_uris_to_cache(&uris)
    })
}

//
// File Picker (sync)
//

use crate::backend::FilePickerDialogImpl;
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        let paths = pick_files_android_multiple(&self.filters, false, self.title.as_deref())?;
        paths.into_iter().next()
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        pick_files_android_multiple(&self.filters, true, self.title.as_deref())
    }
}

//
// File Picker (async)
//

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        Box::pin(async move {
            let paths = pick_files_android_multiple(&self.filters, false, self.title.as_deref());
            paths.and_then(|mut v| v.into_iter().next().map(FileHandle::from))
        })
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        Box::pin(async move {
            let paths = pick_files_android_multiple(&self.filters, true, self.title.as_deref());
            paths.map(|v| v.into_iter().map(FileHandle::from).collect())
        })
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

