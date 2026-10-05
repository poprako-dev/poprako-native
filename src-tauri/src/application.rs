#[cfg(target_os = "macos")]
use objc2_foundation::{NSBundle, NSString};

#[cfg(target_os = "macos")]
fn localize(mut context: tauri::Context<tauri::Wry>) -> tauri::Context<tauri::Wry> {
    let name = NSBundle::mainBundle()
        .objectForInfoDictionaryKey(&NSString::from_str("CFBundleDisplayName"))
        .and_then(|value| value.downcast::<NSString>().ok());

    if let Some(name) = name {
        let name = name.to_string();

        context.package_info_mut().name.clone_from(&name);

        if let Some(window) = context.config_mut().app.windows.first_mut() {
            window.title = name;
        }
    }

    context
}

pub fn create_context() -> tauri::Context<tauri::Wry> {
    let context = tauri::generate_context!();

    #[cfg(target_os = "macos")]
    return localize(context);

    #[cfg(not(target_os = "macos"))]
    context
}
