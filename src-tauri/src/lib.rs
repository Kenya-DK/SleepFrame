mod commands;
mod executor;
mod input;
mod log_processor;
pub mod ocr;
mod runtime;
mod state;
mod win;
mod workflow;

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use state::{AppState, Engine};

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::with_id("main")
        .icon(runtime::idle_icon())
        .tooltip("SleepFrame")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "show" => show_main_window(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();

            let settings_dir = handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("settings");

            let engine = Engine::new(settings_dir);
            let hotkey = engine.hotkey.clone();
            app.manage(AppState {
                engine: Mutex::new(engine),
            });

            setup_tray(&handle)?;
            if let Some(window) = handle.get_webview_window("main") {
                let _ = window.set_icon(runtime::idle_icon());
            }
            let _ = runtime::apply_hotkey(&handle, &hotkey);
            log_processor::start();

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::get_node_specs,
            commands::select_workflow,
            commands::start_flow,
            commands::stop_flow,
            commands::toggle_flow,
            commands::set_hotkey,
            commands::get_workflow,
            commands::create_workflow,
            commands::save_workflow,
            commands::delete_workflow,
            commands::reset_workflow,
            commands::get_method,
            commands::create_method,
            commands::save_method,
            commands::delete_method,
            commands::reset_method,
            commands::duplicate_method,
            commands::export_workflow,
            commands::import_workflow,
            commands::start_pick,
            commands::stop_pick,
            commands::poll_pick,
            commands::start_region_select,
            commands::submit_region,
            commands::cancel_region_select,
            commands::preview_region,
            commands::foreground_window_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
