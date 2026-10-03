//! Runtime glue: tray icons, status/notification helpers, the global hotkey and
//! starting/stopping the selected workflow.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use tauri::image::Image;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_notification::NotificationExt;

use crate::executor::Executor;
use crate::state::{AppState, Engine, HotkeyConfig};

pub fn idle_icon() -> Image<'static> {
    Image::from_bytes(include_bytes!("../icons/icon.png")).expect("valid idle icon")
}

fn running_icon() -> Image<'static> {
    Image::from_bytes(include_bytes!("../icons/icon_on.png")).expect("valid running icon")
}

fn with_engine<R>(app: &AppHandle, f: impl FnOnce(&mut Engine) -> R) -> R {
    let state = app.state::<AppState>();
    let mut guard = state.engine.lock().expect("engine mutex poisoned");
    f(&mut guard)
}

pub fn set_status(app: &AppHandle, text: &str) {
    with_engine(app, |engine| engine.status = text.to_string());
    let _ = app.emit("macro-status", text);
}

pub fn update_tray(app: &AppHandle, running: bool) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_icon(Some(if running { running_icon() } else { idle_icon() }));
        let _ = tray.set_tooltip(Some(if running {
            "SleepFrame - Running"
        } else {
            "SleepFrame - Idle"
        }));
    }
}

pub fn notify(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

/// (Re)registers the single global shortcut that toggles the selected workflow.
pub fn apply_hotkey(app: &AppHandle, config: &HotkeyConfig) -> Result<(), String> {
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();

    let accelerator = if config.modifier == "None" || config.modifier.is_empty() {
        config.key.clone()
    } else {
        format!("{}+{}", config.modifier, config.key)
    };

    let shortcut: Shortcut = accelerator
        .parse()
        .map_err(|_| format!("Invalid hotkey: {accelerator}"))?;

    let handle = app.clone();
    shortcuts
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                toggle(&handle);
            }
        })
        .map_err(|error| error.to_string())
}

pub fn toggle(app: &AppHandle) {
    let running = with_engine(app, |engine| engine.running);
    if running {
        stop(app);
    } else {
        start(app);
    }
}

pub fn start(app: &AppHandle) {
    let (workflow, stop, methods, running_nodes, last_ocr) = {
        let state = app.state::<AppState>();
        let mut engine = state.engine.lock().expect("engine mutex poisoned");
        if engine.running {
            return;
        }
        let selected = engine.selected.clone();
        let Some(workflow) = engine.workflows.iter().find(|w| w.id == selected).cloned() else {
            return;
        };
        engine.running = true;
        engine.status = "Starting...".to_string();
        let stop = engine.fresh_stop_flag();
        let methods = Arc::new(engine.methods.clone());
        let running_nodes = Arc::new(Mutex::new(HashSet::new()));
        let last_ocr = Arc::new(Mutex::new(String::new()));
        (workflow, stop, methods, running_nodes, last_ocr)
    };

    update_tray(app, true);
    let _ = app.emit("macro-started", &workflow.id);
    set_status(app, "Starting...");

    let handle = app.clone();
    std::thread::spawn(move || {
        match Executor::new(handle.clone(), stop, methods, running_nodes, last_ocr) {
            Ok(mut executor) => {
                if let Err(error) = executor.run(&workflow) {
                    set_status(&handle, &format!("Error: {error}"));
                }
            }
            Err(error) => set_status(&handle, &format!("Input error: {error}")),
        }
        finish(&handle);
    });
}

pub fn stop(app: &AppHandle) {
    let was_running = {
        let state = app.state::<AppState>();
        let mut engine = state.engine.lock().expect("engine mutex poisoned");
        if !engine.running {
            false
        } else {
            engine.running = false;
            engine.status = "Idle".to_string();
            engine.request_stop();
            true
        }
    };

    if was_running {
        update_tray(app, false);
        let _ = app.emit("macro-stopped", ());
        let _ = app.emit("macro-status", "Idle");
        notify(app, "SleepFrame", "Macro stopped");
    }
}

fn finish(app: &AppHandle) {
    let still_running = {
        let state = app.state::<AppState>();
        let mut engine = state.engine.lock().expect("engine mutex poisoned");
        if engine.running {
            engine.running = false;
            engine.status = "Idle".to_string();
            true
        } else {
            false
        }
    };

    if still_running {
        update_tray(app, false);
        let _ = app.emit("macro-stopped", ());
        let _ = app.emit("macro-status", "Idle");
        notify(app, "SleepFrame", "Macro stopped");
    }
}
