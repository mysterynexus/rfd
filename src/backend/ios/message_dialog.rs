use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

use block2::RcBlock;
use dispatch2::run_on_main;
use objc2_foundation::NSString;
use objc2_ui_kit::{UIAlertAction, UIAlertActionStyle, UIAlertController, UIAlertControllerStyle};

use super::delegate_future::{DelegateFuture, DelegateFutureState};
use super::utils::get_root_view_controller;
use crate::backend::DialogFutureType;
use crate::message_dialog::{MessageButtons, MessageDialog, MessageDialogResult};

use crate::backend::AsyncMessageDialogImpl;
impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<MessageDialogResult> {
        let (future, state) = DelegateFuture::new();
        run_on_main(move |mtm| {
            let Some(presenting_vc) = get_root_view_controller(mtm) else {
                state.lock().unwrap().complete(MessageDialogResult::Cancel);
                return;
            };
            let title = NSString::from_str(&self.title);
            let message = NSString::from_str(&self.description);
            let alert = unsafe {
                UIAlertController::alertControllerWithTitle_message_preferredStyle(
                    Some(&title),
                    Some(&message),
                    UIAlertControllerStyle::Alert,
                    mtm,
                )
            };
            let buttons: Vec<(&str, MessageDialogResult, UIAlertActionStyle)> = match &self.buttons
            {
                MessageButtons::Ok => {
                    vec![("OK", MessageDialogResult::Ok, UIAlertActionStyle::Default)]
                }
                MessageButtons::OkCancel => vec![
                    ("OK", MessageDialogResult::Ok, UIAlertActionStyle::Default),
                    (
                        "Cancel",
                        MessageDialogResult::Cancel,
                        UIAlertActionStyle::Cancel,
                    ),
                ],
                MessageButtons::YesNo => vec![
                    ("Yes", MessageDialogResult::Yes, UIAlertActionStyle::Default),
                    ("No", MessageDialogResult::No, UIAlertActionStyle::Default),
                ],
                MessageButtons::YesNoCancel => vec![
                    ("Yes", MessageDialogResult::Yes, UIAlertActionStyle::Default),
                    ("No", MessageDialogResult::No, UIAlertActionStyle::Default),
                    (
                        "Cancel",
                        MessageDialogResult::Cancel,
                        UIAlertActionStyle::Cancel,
                    ),
                ],
                MessageButtons::OkCustom(ok) => vec![(
                    ok.as_str(),
                    MessageDialogResult::Custom(ok.clone()),
                    UIAlertActionStyle::Default,
                )],
                MessageButtons::OkCancelCustom(ok, cancel) => vec![
                    (
                        ok.as_str(),
                        MessageDialogResult::Custom(ok.clone()),
                        UIAlertActionStyle::Default,
                    ),
                    (
                        cancel.as_str(),
                        MessageDialogResult::Custom(cancel.clone()),
                        UIAlertActionStyle::Cancel,
                    ),
                ],
                MessageButtons::YesNoCancelCustom(yes, no, cancel) => vec![
                    (
                        yes.as_str(),
                        MessageDialogResult::Custom(yes.clone()),
                        UIAlertActionStyle::Default,
                    ),
                    (
                        no.as_str(),
                        MessageDialogResult::Custom(no.clone()),
                        UIAlertActionStyle::Default,
                    ),
                    (
                        cancel.as_str(),
                        MessageDialogResult::Custom(cancel.clone()),
                        UIAlertActionStyle::Cancel,
                    ),
                ],
            };
            for (title, result, style) in buttons {
                let state_clone: Arc<Mutex<DelegateFutureState<MessageDialogResult>>> =
                    state.clone();
                let handler = RcBlock::new(move |_: NonNull<UIAlertAction>| {
                    state_clone.lock().unwrap().complete(result.clone());
                });
                let action = unsafe {
                    UIAlertAction::actionWithTitle_style_handler(
                        Some(&NSString::from_str(title)),
                        style,
                        Some(&handler),
                        mtm,
                    )
                };
                unsafe { alert.addAction(&action) };
            }
            unsafe {
                presenting_vc.presentViewController_animated_completion(&alert, true, None);
            }
        });
        Box::pin(future)
    }
}
