use std::path::Path;

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_foundation::{NSArray, NSString, NSURL};
use objc2_ui_kit::{UIApplication, UIViewController, UIWindowScene};
use objc2_uniform_type_identifiers::UTType;

use crate::FileDialog;

pub fn get_root_view_controller(mtm: MainThreadMarker) -> Option<Retained<UIViewController>> {
    let app = UIApplication::sharedApplication(mtm);
    let scenes = unsafe { app.connectedScenes() };
    for scene in scenes {
        if let Ok(window_scene) = scene.downcast::<UIWindowScene>() {
            if let Some(window) = unsafe { window_scene.keyWindow() } {
                return window.rootViewController();
            }
        }
    }
    None
}

pub fn filters_to_uttypes(dialog: &FileDialog) -> Retained<NSArray<UTType>> {
    let mut types: Vec<Retained<UTType>> = Vec::new();
    for filter in &dialog.filters {
        for ext in &filter.extensions {
            let ns_ext = NSString::from_str(ext);
            if let Some(ut) = unsafe { UTType::typeWithFilenameExtension(&ns_ext) } {
                types.push(ut);
            }
        }
    }
    if types.is_empty() {
        if let Some(item) =
            unsafe { UTType::typeWithIdentifier(&NSString::from_str("public.item")) }
        {
            types.push(item);
        }
    }
    NSArray::from_retained_slice(&types)
}

pub fn folder_uttypes() -> Retained<NSArray<UTType>> {
    let folder = unsafe { UTType::typeWithIdentifier(&NSString::from_str("public.folder")) }
        .expect("public.folder UTType should exist");
    NSArray::from_retained_slice(&[folder])
}

pub fn path_to_nsurl(path: &Path) -> Option<Retained<NSURL>> {
    let path_str = path.to_str()?;
    let ns_path = NSString::from_str(path_str);
    Some(unsafe { NSURL::fileURLWithPath(&ns_path) })
}
