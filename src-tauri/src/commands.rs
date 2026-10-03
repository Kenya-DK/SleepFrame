//! Tauri commands exposed to the frontend.
// (Includes Snipping-Tool style region selection via an overlay window.)

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State,
};

use crate::runtime;
use crate::state::{AppState, HotkeyConfig};
use crate::win;
use crate::workflow::{
    node_specs, Method, MethodSummary, NodeSpec, ShareBundle, Workflow, WorkflowSummary,
};

#[derive(Debug, Clone, Serialize)]
pub struct StateDto {
    pub selected: String,
    pub running: bool,
    pub status: String,
    pub workflows: Vec<WorkflowSummary>,
    pub methods: Vec<MethodSummary>,
    pub hotkey: HotkeyConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegionSelection {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Shows the fullscreen overlay window for Snipping-Tool style region selection.
#[tauri::command]
pub fn start_region_select(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("region-overlay")
        .ok_or_else(|| "Region overlay window is missing".to_string())?;

    if let Some(monitor) = app.primary_monitor().ok().flatten() {
        let position = monitor.position();
        let size = monitor.size();
        let _ = window.set_position(PhysicalPosition::new(position.x, position.y));
        let _ = window.set_size(PhysicalSize::new(size.width, size.height));
    }

    window.show().map_err(|error| error.to_string())?;
    let _ = window.set_always_on_top(true);
    let _ = window.set_focus();
    Ok(())
}

/// Called by the overlay once a region has been dragged out.
#[tauri::command]
pub fn submit_region(app: AppHandle, x: f64, y: f64, width: f64, height: f64) {
    let scale = app
        .get_webview_window("region-overlay")
        .and_then(|window| window.current_monitor().ok().flatten())
        .map(|monitor| monitor.scale_factor())
        .unwrap_or(1.0);

    let region = RegionSelection {
        x: (x * scale).round() as i32,
        y: (y * scale).round() as i32,
        width: (width * scale).round() as i32,
        height: (height * scale).round() as i32,
    };

    if let Some(window) = app.get_webview_window("region-overlay") {
        let _ = window.hide();
    }
    let _ = app.emit_to("main", "region-selected", region);
}

#[tauri::command]
pub fn cancel_region_select(app: AppHandle) {
    if let Some(window) = app.get_webview_window("region-overlay") {
        let _ = window.hide();
    }
}

/// Captures a region, applies the OCR preprocessing, and returns a base64 PNG.
#[tauri::command]
pub fn preview_region(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    invert: String,
    scale: u32,
    threshold: bool,
    color: String,
    tolerance: u32,
) -> Result<String, String> {
    let options = crate::ocr::PreprocessOptions {
        invert: crate::ocr::InvertMode::parse(&invert),
        scale: scale.clamp(1, 4),
        threshold,
        color: crate::ocr::parse_color(&color),
        tolerance: tolerance.min(255),
        allowed_chars: None,
        detection_model: None,
        recognition_model: None,
    };
    crate::ocr::preview_png(x, y, width, height, &options)
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub process: String,
    pub title: String,
}

/// Returns the executable name and title of the current foreground window.
#[tauri::command]
pub fn foreground_window_info() -> WindowInfo {
    let hwnd = win::foreground_window();
    let pid = win::pid_of_window(hwnd);
    WindowInfo {
        process: win::process_name_by_pid(pid).unwrap_or_default(),
        title: win::window_title(hwnd),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PickResult {
    pub x: i32,
    pub y: i32,
    /// True on the edge where the left mouse button changed from up to down.
    pub clicked: bool,
}

#[derive(Clone, Copy)]
struct PickState {
    armed: bool,
    was_down: bool,
}

static PICK: std::sync::Mutex<PickState> = std::sync::Mutex::new(PickState {
    armed: false,
    was_down: false,
});

/// Arms the position picker so the next mouse click anywhere is reported.
#[tauri::command]
pub fn start_pick() {
    let mut state = PICK.lock().expect("pick mutex poisoned");
    state.armed = true;
    state.was_down = win::left_button_down();
}

#[tauri::command]
pub fn stop_pick() {
    PICK.lock().expect("pick mutex poisoned").armed = false;
}

/// Polled while picking: returns the live cursor position and whether a new
/// click happened since the previous poll.
#[tauri::command]
pub fn poll_pick() -> PickResult {
    let (x, y) = win::cursor_pos();
    let down = win::left_button_down();

    let mut state = PICK.lock().expect("pick mutex poisoned");
    let clicked = state.armed && down && !state.was_down;
    state.was_down = down;

    PickResult { x, y, clicked }
}

#[tauri::command]
pub fn get_state(state: State<AppState>) -> StateDto {
    let engine = state.engine.lock().expect("engine mutex poisoned");
    StateDto {
        selected: engine.selected.clone(),
        running: engine.running,
        status: engine.status.clone(),
        workflows: engine.workflow_summaries(),
        methods: engine.method_summaries(),
        hotkey: engine.hotkey.clone(),
    }
}

#[tauri::command]
pub fn get_node_specs() -> Vec<NodeSpec> {
    node_specs()
}

#[tauri::command]
pub fn select_workflow(state: State<AppState>, id: String) {
    let mut engine = state.engine.lock().expect("engine mutex poisoned");
    if !engine.running {
        engine.set_selected(&id);
    }
}

#[tauri::command]
pub fn start_flow(app: AppHandle) {
    runtime::start(&app);
}

#[tauri::command]
pub fn stop_flow(app: AppHandle) {
    runtime::stop(&app);
}

#[tauri::command]
pub fn toggle_flow(app: AppHandle) {
    runtime::toggle(&app);
}

#[tauri::command]
pub fn set_hotkey(app: AppHandle, state: State<AppState>, modifier: String, key: String) {
    let config = {
        let mut engine = state.engine.lock().expect("engine mutex poisoned");
        engine.hotkey = HotkeyConfig { modifier, key };
        engine.save_app_config();
        engine.hotkey.clone()
    };
    let _ = runtime::apply_hotkey(&app, &config);
}

#[tauri::command]
pub fn get_workflow(state: State<AppState>, id: String) -> Option<Workflow> {
    state.engine.lock().expect("engine mutex poisoned").workflow(&id)
}

#[tauri::command]
pub fn create_workflow(state: State<AppState>, name: String) -> Result<Workflow, String> {
    state
        .engine
        .lock()
        .expect("engine mutex poisoned")
        .create_workflow(&name)
}

#[tauri::command]
pub fn save_workflow(state: State<AppState>, workflow: Workflow) -> Result<(), String> {
    state
        .engine
        .lock()
        .expect("engine mutex poisoned")
        .save_workflow(workflow)
}

#[tauri::command]
pub fn delete_workflow(state: State<AppState>, id: String) -> Result<(), String> {
    state
        .engine
        .lock()
        .expect("engine mutex poisoned")
        .delete_workflow(&id)
}

#[tauri::command]
pub fn reset_workflow(state: State<AppState>, id: String) -> Result<Workflow, String> {
    state
        .engine
        .lock()
        .expect("engine mutex poisoned")
        .reset_workflow(&id)
}

#[tauri::command]
pub fn get_method(state: State<AppState>, id: String) -> Option<Method> {
    state.engine.lock().expect("engine mutex poisoned").method(&id)
}

#[tauri::command]
pub fn create_method(state: State<AppState>, name: String) -> Result<Method, String> {
    state
        .engine
        .lock()
        .expect("engine mutex poisoned")
        .create_method(&name)
}

#[tauri::command]
pub fn save_method(state: State<AppState>, method: Method) -> Result<(), String> {
    state
        .engine
        .lock()
        .expect("engine mutex poisoned")
        .save_method(method)
}

#[tauri::command]
pub fn delete_method(state: State<AppState>, id: String) -> Result<(), String> {
    state
        .engine
        .lock()
        .expect("engine mutex poisoned")
        .delete_method(&id)
}

#[tauri::command]
pub fn reset_method(state: State<AppState>, id: String) -> Result<Method, String> {
    state
        .engine
        .lock()
        .expect("engine mutex poisoned")
        .reset_method(&id)
}

#[tauri::command]
pub fn duplicate_method(state: State<AppState>, id: String) -> Result<Method, String> {
    state
        .engine
        .lock()
        .expect("engine mutex poisoned")
        .duplicate_method(&id)
}

#[tauri::command]
pub fn export_workflow(state: State<AppState>, id: String) -> Result<String, String> {
    let bundle = state
        .engine
        .lock()
        .expect("engine mutex poisoned")
        .export_workflow(&id)?;
    let json = serde_json::to_string(&bundle).map_err(|error| error.to_string())?;
    Ok(STANDARD.encode(json))
}

#[tauri::command]
pub fn import_workflow(state: State<AppState>, code: String) -> Result<Workflow, String> {
    let trimmed = code.trim();
    let raw = match STANDARD.decode(trimmed) {
        Ok(bytes) => String::from_utf8(bytes).map_err(|error| error.to_string())?,
        Err(_) => trimmed.to_string(),
    };
    let bundle: ShareBundle = serde_json::from_str(&raw).map_err(|error| error.to_string())?;
    state
        .engine
        .lock()
        .expect("engine mutex poisoned")
        .import_workflow(bundle)
}
