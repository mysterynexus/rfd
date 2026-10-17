mod file_dialog;
mod jni_future;
mod message_dialog;
mod utils;

use jni::errors::{LogErrorAndDefault, Result};
use jni::objects::{JClass, JObjectArray, JString};
use jni::sys::{jint, jlong};
use jni::EnvUnowned;
use std::path::PathBuf;

use utils::{
    complete_file_request, complete_message_request, init_helper_class, jstring_array_to_paths,
};

pub use utils::sync_save_file;

#[no_mangle]
pub extern "system" fn Java_rfd_RfdHelper_nativeInit<'local>(
    mut unowned_env: EnvUnowned<'local>,
    class: JClass<'local>,
) {
    unowned_env
        .with_env(|env| init_helper_class(env, &class))
        .resolve::<LogErrorAndDefault>();
}

#[no_mangle]
pub extern "system" fn Java_rfd_RfdHelper_nativeOnFilesSelected<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    request_id: jlong,
    paths: JObjectArray<'local, JString<'local>>,
) {
    unowned_env
        .with_env(|env| -> Result<()> {
            let result = if paths.is_null() {
                Ok(None)
            } else {
                jstring_array_to_paths(env, &paths)
            };
            result
                .inspect_err(|_| complete_file_request(request_id as u64, None))
                .map(|paths| complete_file_request(request_id as u64, paths))
        })
        .resolve::<LogErrorAndDefault>();
}

#[no_mangle]
pub extern "system" fn Java_rfd_RfdHelper_nativeOnCancelled<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    request_id: jlong,
) {
    unowned_env
        .with_env(|_| -> Result<()> {
            complete_file_request(request_id as u64, None);
            Ok(())
        })
        .resolve::<LogErrorAndDefault>();
}

#[no_mangle]
pub extern "system" fn Java_rfd_RfdHelper_nativeOnMessageResult<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    request_id: jlong,
    result_code: jint,
) {
    unowned_env
        .with_env(|_| -> Result<()> {
            complete_message_request(request_id as u64, result_code);
            Ok(())
        })
        .resolve::<LogErrorAndDefault>();
}

#[no_mangle]
pub extern "system" fn Java_rfd_RfdHelper_nativeOnSaveFileSelected<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    request_id: jlong,
    path: JString<'local>,
) {
    unowned_env
        .with_env(|env| -> Result<()> {
            let result = if path.is_null() {
                Ok(None)
            } else {
                path.try_to_string(env)
                    .map(|path| Some(vec![PathBuf::from(path)]))
            };
            result
                .inspect_err(|_| complete_file_request(request_id as u64, None))
                .map(|paths| complete_file_request(request_id as u64, paths))
        })
        .resolve::<LogErrorAndDefault>();
}
