use std::sync::Arc;

mod application;

pub mod bridge;
pub mod complex;
pub mod data;
pub mod harness;
pub mod implementation;
pub mod model;
pub mod part;
pub mod result;
pub mod usecase;
pub mod value;

use std::sync::atomic::Ordering;

use tauri::{Emitter, Manager, RunEvent, WindowEvent};

use crate::harness::Harness;

/// # Errors
/// Reports initialization failures before the application starts serving commands.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let builder = bridge::builder();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .register_asynchronous_uri_scheme_protocol("poprako-resource", bridge::protocol::respond)
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            let directory = app.path().app_data_dir()?;

            let harness = Arc::new(Harness::open(directory)?);

            app.manage(Arc::clone(&harness));

            tauri::async_runtime::spawn(async move {
                if harness.ensure_resources().await.is_err() {
                    eprintln!(
                        "Startup resource recovery is incomplete; new resource tasks remain blocked"
                    );
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event
                && !window
                    .state::<Arc<Harness>>()
                    .allow_exit
                    .load(Ordering::Acquire)
            {
                api.prevent_close();

                let _ = window.emit("application-close-requested", ());
            }
        })
        .build(application::create_context())?
        .run(|app, event| {
            if let RunEvent::ExitRequested { api, .. } = event
                && !app
                    .state::<Arc<Harness>>()
                    .allow_exit
                    .load(Ordering::Acquire)
            {
                api.prevent_exit();

                let _ = app.emit("application-close-requested", ());
            }
        });

    Ok(())
}
