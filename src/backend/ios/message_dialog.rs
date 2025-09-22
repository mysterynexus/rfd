use crate::backend::DialogFutureType;
use crate::message_dialog::{MessageDialog, MessageDialogResult};

use crate::backend::{AsyncMessageDialogImpl, MessageDialogImpl};

impl MessageDialogImpl for MessageDialog {
    fn show(self) -> MessageDialogResult {
        MessageDialogResult::Cancel
    }
}

impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<MessageDialogResult> {
        Box::pin(async { MessageDialogResult::Cancel })
    }
}

