mod file_dialog;
mod jni_future;
mod message_dialog;
mod utils;

use jni::objects::{JClass, JObjectArray};
use jni::sys::jlong;
use jni::JNIEnv;
use std::path::PathBuf;

use utils::{complete_file_request, complete_message_request, init_helper_class, jstring_array_to_paths};

pub use utils::sync_save_file;

#[no_mangle]
pub extern "system" fn Java_rfd_RfdHelper_nativeInit(mut env: JNIEnv, _class: JClass) {
    init_helper_class(&mut env);
}

#[no_mangle]
pub extern "system" fn Java_rfd_RfdHelper_nativeOnFilesSelected(
    mut env: JNIEnv,
    _class: JClass,
    request_id: jlong,
    paths: JObjectArray,
) {
    let request_id = request_id as u64;
    let result = if paths.is_null() {
        None
    } else {
        jstring_array_to_paths(&mut env, &paths)
    };
    complete_file_request(request_id, result);
}

#[no_mangle]
pub extern "system" fn Java_rfd_RfdHelper_nativeOnCancelled(
    _env: JNIEnv,
    _class: JClass,
    request_id: jlong,
) {
    let request_id = request_id as u64;
    complete_file_request(request_id, None);
}

#[no_mangle]
pub extern "system" fn Java_rfd_RfdHelper_nativeOnMessageResult(
    _env: JNIEnv,
    _class: JClass,
    request_id: jlong,
    result_code: jni::sys::jint,
) {
    let request_id = request_id as u64;
    complete_message_request(request_id, result_code);
}

#[no_mangle]
pub extern "system" fn Java_rfd_RfdHelper_nativeOnSaveFileSelected(
    mut env: JNIEnv,
    _class: JClass,
    request_id: jlong,
    path: jni::objects::JString,
) {
    let request_id = request_id as u64;
    let result = if path.is_null() {
        None
    } else if let Ok(s) = env.get_string(&path) {
        s.to_str().ok().map(|p| vec![PathBuf::from(p)])
    } else {
        None
    };
    complete_file_request(request_id, result);
}
