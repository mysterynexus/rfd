use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSURL};
use objc2_ui_kit::{UIDocumentPickerDelegate, UIDocumentPickerViewController, UIViewController};
use objc2_uniform_type_identifiers::UTType;

use super::delegate_future::DelegateFutureState;

pub struct PickerDelegateIvars {
    state: Arc<Mutex<DelegateFutureState<Option<Vec<PathBuf>>>>>,
    prevent_drop: Mutex<Option<Retained<PickerDelegate>>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "RFDPickerDelegate"]
    #[ivars = PickerDelegateIvars]
    pub struct PickerDelegate;

    unsafe impl NSObjectProtocol for PickerDelegate {}

    unsafe impl UIDocumentPickerDelegate for PickerDelegate {
        #[unsafe(method(documentPicker:didPickDocumentsAtURLs:))]
        fn document_picker_did_pick_documents(
            &self,
            _controller: &UIDocumentPickerViewController,
            urls: &NSArray<NSURL>,
        ) {
            let paths: Vec<PathBuf> = urls
                .to_vec()
                .into_iter()
                .filter_map(|url| {
                    unsafe { url.startAccessingSecurityScopedResource() };
                    unsafe { url.path() }.map(|p| PathBuf::from(p.to_string()))
                })
                .collect();
            let result = if paths.is_empty() { None } else { Some(paths) };
            self.complete(result);
        }

        #[unsafe(method(documentPickerWasCancelled:))]
        fn document_picker_was_cancelled(&self, _controller: &UIDocumentPickerViewController) {
            self.complete(None);
        }
    }
);

impl PickerDelegate {
    fn new(
        mtm: MainThreadMarker,
        state: Arc<Mutex<DelegateFutureState<Option<Vec<PathBuf>>>>>,
    ) -> Retained<Self> {
        let this = mtm.alloc::<Self>();
        let this = this.set_ivars(PickerDelegateIvars {
            state,
            prevent_drop: Mutex::new(None),
        });
        unsafe { msg_send![super(this), init] }
    }

    fn complete(&self, result: Option<Vec<PathBuf>>) {
        self.ivars().state.lock().unwrap().complete(result);
        *self.ivars().prevent_drop.lock().unwrap() = None;
    }
}

pub fn build_pick_dialog(
    content_types: &NSArray<UTType>,
    allows_multiple: bool,
    starting_directory: Option<&std::path::Path>,
    mtm: MainThreadMarker,
    state: Arc<Mutex<DelegateFutureState<Option<Vec<PathBuf>>>>>,
) -> Retained<UIDocumentPickerViewController> {
    let picker = unsafe {
        UIDocumentPickerViewController::initForOpeningContentTypes(mtm.alloc(), content_types)
    };
    unsafe { picker.setAllowsMultipleSelection(allows_multiple) };
    let delegate = PickerDelegate::new(mtm, state);
    unsafe {
        picker.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    }
    *delegate.ivars().prevent_drop.lock().unwrap() = Some(delegate.clone());
    if let Some(dir) = starting_directory {
        if let Some(url) = super::utils::path_to_nsurl(dir) {
            unsafe { picker.setDirectoryURL(Some(&url)) };
        }
    }
    picker
}

pub fn build_export_dialog(
    urls: &NSArray<NSURL>,
    as_copy: bool,
    starting_directory: Option<&std::path::Path>,
    mtm: MainThreadMarker,
    state: Arc<Mutex<DelegateFutureState<Option<Vec<PathBuf>>>>>,
) -> Retained<UIDocumentPickerViewController> {
    let picker = unsafe {
        UIDocumentPickerViewController::initForExportingURLs_asCopy(mtm.alloc(), urls, as_copy)
    };
    let delegate = PickerDelegate::new(mtm, state);
    unsafe {
        picker.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    }
    *delegate.ivars().prevent_drop.lock().unwrap() = Some(delegate.clone());
    if let Some(dir) = starting_directory {
        if let Some(url) = super::utils::path_to_nsurl(dir) {
            unsafe { picker.setDirectoryURL(Some(&url)) };
        }
    }
    picker
}

pub fn present(picker: &UIDocumentPickerViewController, presenting_vc: &UIViewController) {
    unsafe {
        presenting_vc.presentViewController_animated_completion(picker, true, None);
    }
}
