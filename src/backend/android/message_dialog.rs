use crate::backend::DialogFutureType;
use crate::message_dialog::{MessageButtons, MessageDialog, MessageDialogResult};

use crate::backend::MessageDialogImpl;
impl MessageDialogImpl for MessageDialog {
    fn show(self) -> MessageDialogResult {
        match self.buttons {
            MessageButtons::Ok | MessageButtons::OkCustom(_) => MessageDialogResult::Ok,
            MessageButtons::OkCancel
            | MessageButtons::YesNo
            | MessageButtons::YesNoCancel
            | MessageButtons::OkCancelCustom(..)
            | MessageButtons::YesNoCancelCustom(..) => MessageDialogResult::Cancel,
        }
    }
}

use crate::backend::AsyncMessageDialogImpl;
impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<MessageDialogResult> {
        let val = MessageDialogImpl::show(self);
        Box::pin(std::future::ready(val))
    }
}

