use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use jni::objects::{JClass, JObject, JObjectArray, JString};
use jni::refs::{Global, IntoAuto};
use jni::sys::jlong;
use jni::{jni_sig, jni_str, Env, JValue, JavaVM};

use super::jni_future::JniFutureState;
use crate::message_dialog::{MessageButtons, MessageDialogResult};
use crate::FileDialog;

static HELPER_CLASS: OnceLock<Global<JClass<'static>>> = OnceLock::new();

pub fn init_helper_class(env: &Env, class: &JClass) -> jni::errors::Result<()> {
    if HELPER_CLASS.get().is_none() {
        let _ = HELPER_CLASS.set(env.new_global_ref(class)?);
    }
    Ok(())
}

pub fn get_vm() -> Option<JavaVM> {
    let ctx = ndk_context::android_context();
    let vm_ptr = ctx.vm().cast::<jni::sys::JavaVM>();
    if vm_ptr.is_null() {
        return None;
    }
    Some(unsafe { JavaVM::from_raw(vm_ptr) })
}

pub fn with_jni_helper<F, T>(default: T, f: F) -> T
where
    F: FnOnce(&mut Env, &JObject) -> jni::errors::Result<T>,
{
    let Some(vm) = get_vm() else { return default };
    let result = vm.attach_current_thread_for_scope(|env| {
        let helper = get_rfd_helper(env)?;
        f(env, &helper)
    });
    result.unwrap_or_else(|error| {
        log::error!("Android dialog failed: {error}");
        default
    })
}

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);
static PENDING_REQUESTS: OnceLock<Mutex<HashMap<u64, PendingRequest>>> = OnceLock::new();

fn pending_requests() -> &'static Mutex<HashMap<u64, PendingRequest>> {
    PENDING_REQUESTS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub enum PendingRequest {
    FilePick(Arc<Mutex<JniFutureState<Option<Vec<PathBuf>>>>>),
    MessageDialog {
        state: Arc<Mutex<JniFutureState<MessageDialogResult>>>,
        buttons: MessageButtons,
    },
}

pub fn register_file_request(state: Arc<Mutex<JniFutureState<Option<Vec<PathBuf>>>>>) -> u64 {
    let id = NEXT_REQUEST_ID.fetch_add(1, Ordering::SeqCst);
    pending_requests()
        .lock()
        .unwrap()
        .insert(id, PendingRequest::FilePick(state));
    id
}

pub fn register_message_request(
    state: Arc<Mutex<JniFutureState<MessageDialogResult>>>,
    buttons: MessageButtons,
) -> u64 {
    let id = NEXT_REQUEST_ID.fetch_add(1, Ordering::SeqCst);
    pending_requests()
        .lock()
        .unwrap()
        .insert(id, PendingRequest::MessageDialog { state, buttons });
    id
}

pub fn complete_file_request(request_id: u64, paths: Option<Vec<PathBuf>>) {
    if let Some(request) = pending_requests().lock().unwrap().remove(&request_id) {
        if let PendingRequest::FilePick(state) = request {
            state.lock().unwrap().complete(paths);
        }
    }
}

pub fn complete_message_request(request_id: u64, result_code: i32) {
    if let Some(request) = pending_requests().lock().unwrap().remove(&request_id) {
        if let PendingRequest::MessageDialog { state, buttons } = request {
            let result = result_from_code(result_code, &buttons);
            state.lock().unwrap().complete(result);
        }
    }
}

const RESULT_OK: i32 = 0;
pub const RESULT_CANCEL: i32 = 1;
const RESULT_YES: i32 = 2;
const RESULT_NO: i32 = 3;
const RESULT_CUSTOM_BASE: i32 = 100;

fn result_from_code(code: i32, buttons: &MessageButtons) -> MessageDialogResult {
    match code {
        RESULT_OK => MessageDialogResult::Ok,
        RESULT_CANCEL => MessageDialogResult::Cancel,
        RESULT_YES => MessageDialogResult::Yes,
        RESULT_NO => MessageDialogResult::No,
        c if c >= RESULT_CUSTOM_BASE => {
            let idx = (c - RESULT_CUSTOM_BASE) as usize;
            match buttons {
                MessageButtons::OkCustom(ok) if idx == 0 => MessageDialogResult::Custom(ok.clone()),
                MessageButtons::OkCancelCustom(ok, cancel) => match idx {
                    0 => MessageDialogResult::Custom(ok.clone()),
                    1 => MessageDialogResult::Custom(cancel.clone()),
                    _ => MessageDialogResult::Cancel,
                },
                MessageButtons::YesNoCancelCustom(yes, no, cancel) => match idx {
                    0 => MessageDialogResult::Custom(yes.clone()),
                    1 => MessageDialogResult::Custom(no.clone()),
                    2 => MessageDialogResult::Custom(cancel.clone()),
                    _ => MessageDialogResult::Cancel,
                },
                _ => MessageDialogResult::Cancel,
            }
        }
        _ => MessageDialogResult::Cancel,
    }
}

pub fn buttons_to_spec(buttons: &MessageButtons) -> Vec<(&str, i32)> {
    match buttons {
        MessageButtons::Ok => vec![("OK", RESULT_OK)],
        MessageButtons::OkCancel => vec![("OK", RESULT_OK), ("Cancel", RESULT_CANCEL)],
        MessageButtons::YesNo => vec![("Yes", RESULT_YES), ("No", RESULT_NO)],
        MessageButtons::YesNoCancel => {
            vec![
                ("Yes", RESULT_YES),
                ("No", RESULT_NO),
                ("Cancel", RESULT_CANCEL),
            ]
        }
        MessageButtons::OkCustom(ok) => vec![(ok.as_str(), RESULT_CUSTOM_BASE)],
        MessageButtons::OkCancelCustom(ok, cancel) => {
            vec![
                (ok.as_str(), RESULT_CUSTOM_BASE),
                (cancel.as_str(), RESULT_CUSTOM_BASE + 1),
            ]
        }
        MessageButtons::YesNoCancelCustom(yes, no, cancel) => vec![
            (yes.as_str(), RESULT_CUSTOM_BASE),
            (no.as_str(), RESULT_CUSTOM_BASE + 1),
            (cancel.as_str(), RESULT_CUSTOM_BASE + 2),
        ],
    }
}

pub fn filters_to_mime_types(dialog: &FileDialog) -> Vec<String> {
    let mut types = Vec::new();
    for filter in &dialog.filters {
        for ext in &filter.extensions {
            types.push(extension_to_mime(ext).to_string());
        }
    }
    if types.is_empty() {
        types.push("*/*".to_string());
    }
    types
}

fn extension_to_mime(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "txt" | "text" => "text/plain",
        "htm" | "html" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",
        "csv" => "text/csv",
        "xml" => "application/xml",
        "json" => "application/json",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" | "gzip" => "application/gzip",
        "tar" => "application/x-tar",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "tif" | "tiff" => "image/tiff",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        "m4a" => "audio/mp4",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "mkv" => "video/x-matroska",
        "rs" => "text/x-rust",
        "py" => "text/x-python",
        "java" => "text/x-java-source",
        "c" | "h" => "text/x-c",
        "cpp" | "hpp" | "cc" | "cxx" => "text/x-c++",
        _ => "*/*",
    }
}

pub fn get_first_mime_type(dialog: &FileDialog) -> String {
    for filter in &dialog.filters {
        for ext in &filter.extensions {
            let mime = extension_to_mime(ext);
            if mime != "*/*" {
                return mime.to_string();
            }
        }
    }
    "*/*".to_string()
}

pub fn call_pick_file(
    env: &mut Env,
    helper: &JObject,
    request_id: u64,
    mime_types: &[String],
    multiple: bool,
) -> Result<(), jni::errors::Error> {
    let array = JObjectArray::<JString>::new(env, mime_types.len(), JString::null())?;
    for (index, mime) in mime_types.iter().enumerate() {
        let java_string = env.new_string(mime)?.auto();
        array.set_element(env, index, &java_string)?;
    }
    env.call_method(
        helper,
        jni_str!("pickFile"),
        jni_sig!("(J[Ljava/lang/String;Z)V"),
        &[
            JValue::Long(request_id as jlong),
            JValue::Object(&array),
            JValue::Bool(multiple),
        ],
    )?;
    Ok(())
}

pub fn call_pick_folder(
    env: &mut Env,
    helper: &JObject,
    request_id: u64,
) -> Result<(), jni::errors::Error> {
    env.call_method(
        helper,
        jni_str!("pickFolder"),
        jni_sig!("(J)V"),
        &[JValue::Long(request_id as jlong)],
    )?;
    Ok(())
}

pub fn call_save_file(
    env: &mut Env,
    helper: &JObject,
    request_id: u64,
    mime_type: &str,
    file_name: &str,
) -> Result<(), jni::errors::Error> {
    let mime = env.new_string(mime_type)?;
    let name = env.new_string(file_name)?;
    env.call_method(
        helper,
        jni_str!("saveFile"),
        jni_sig!("(JLjava/lang/String;Ljava/lang/String;)V"),
        &[
            JValue::Long(request_id as jlong),
            JValue::Object(&mime),
            JValue::Object(&name),
        ],
    )?;
    Ok(())
}

pub fn call_show_message(
    env: &mut Env,
    helper: &JObject,
    request_id: u64,
    title: &str,
    message: &str,
    buttons: &[(&str, i32)],
) -> Result<(), jni::errors::Error> {
    let java_title = env.new_string(title)?;
    let java_message = env.new_string(message)?;
    let labels = JObjectArray::<JString>::new(env, buttons.len(), JString::null())?;
    let results = env.new_int_array(buttons.len())?;
    let mut result_values: Vec<i32> = Vec::with_capacity(buttons.len());
    for (index, (label, result)) in buttons.iter().enumerate() {
        let java_label = env.new_string(label)?.auto();
        labels.set_element(env, index, &java_label)?;
        result_values.push(*result);
    }
    results.set_region(env, 0, &result_values)?;
    env.call_method(
        helper,
        jni_str!("showMessageDialog"),
        jni_sig!("(JLjava/lang/String;Ljava/lang/String;[Ljava/lang/String;[I)V"),
        &[
            JValue::Long(request_id as jlong),
            JValue::Object(&java_title),
            JValue::Object(&java_message),
            JValue::Object(&labels),
            JValue::Object(&results),
        ],
    )?;
    Ok(())
}

pub fn get_rfd_helper<'local>(env: &mut Env<'local>) -> jni::errors::Result<JObject<'local>> {
    let helper_class = HELPER_CLASS.get().ok_or(jni::errors::Error::NullPtr(
        "RfdHelper class not initialized",
    ))?;
    env.call_static_method(
        helper_class,
        jni_str!("getInstance"),
        jni_sig!("()Lrfd/RfdHelper;"),
        &[],
    )?
    .into_object()
}

pub fn jstring_array_to_paths(
    env: &mut Env,
    array: &JObjectArray<JString>,
) -> jni::errors::Result<Option<Vec<PathBuf>>> {
    let length = array.len(env)?;
    if length == 0 {
        return Ok(None);
    }
    let mut paths = Vec::with_capacity(length);
    for index in 0..length {
        let java_string = array.get_element(env, index)?.auto();
        paths.push(PathBuf::from(java_string.try_to_string(env)?));
    }
    Ok(Some(paths))
}

pub fn sync_save_file(path: &std::path::Path) -> bool {
    let Some(vm) = get_vm() else { return false };
    let Some(helper_class) = HELPER_CLASS.get() else {
        return false;
    };
    vm.attach_current_thread_for_scope(|env| -> jni::errors::Result<bool> {
        let java_path = env.new_string(path.to_string_lossy())?;
        env.call_static_method(
            helper_class,
            jni_str!("syncSaveFile"),
            jni_sig!("(Ljava/lang/String;)Z"),
            &[JValue::Object(&java_path)],
        )?
        .into_bool()
    })
    .unwrap_or_else(|error| {
        log::error!("Android file sync failed: {error}");
        false
    })
}
