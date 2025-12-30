use super::jni_future::JniFuture;
use super::utils::{buttons_to_spec, call_show_message, register_message_request, with_jni_helper};
use crate::backend::DialogFutureType;
use crate::message_dialog::{MessageDialog, MessageDialogResult};

use crate::backend::AsyncMessageDialogImpl;
impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<MessageDialogResult> {
        let (future, state) = JniFuture::new();
        let request_id = register_message_request(state.clone(), self.buttons.clone());
        let launched = with_jni_helper(false, |env, helper| {
            let button_spec = buttons_to_spec(&self.buttons);
            call_show_message(env, helper, request_id, &self.title, &self.description, &button_spec)?;
            Ok(true)
        });
        if !launched {
            state.lock().unwrap().complete(MessageDialogResult::Cancel);
        }
        Box::pin(future)
    }
}
