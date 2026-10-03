//! Workflow execution engine. Walks the node graph, dispatching each node to a
//! small reusable handler. Supports branching, loops, reusable methods and
//! background method timers.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use enigo::{Button, Direction, Key};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::input::{random_range, sleep_rand, sleep_until_stopped, Input};
use crate::runtime::{notify, set_status};
use crate::win;
use crate::workflow::{Connection, Method, Node, Workflow};

enum Next {
    Port(String),
    End,
}

#[derive(Clone, Serialize)]
struct TimerPayload {
    node: String,
    remaining_ms: i64,
}

#[derive(Clone, Serialize)]
struct OcrPayload {
    node: String,
    text: String,
}

pub struct Executor {
    app: AppHandle,
    input: Input,
    stop: Arc<AtomicBool>,
    methods: Arc<Vec<Method>>,
    /// Node ids currently executing across every branch/mode/timer thread.
    running_nodes: Arc<Mutex<HashSet<String>>>,
    /// Most recent OCR result, shared across branches so Conditions can read it.
    last_ocr: Arc<Mutex<String>>,
    /// Per-node cursor for `send_messages` nodes with sequential ordering.
    list_indices: HashMap<String, usize>,
}

impl Executor {
    pub fn new(
        app: AppHandle,
        stop: Arc<AtomicBool>,
        methods: Arc<Vec<Method>>,
        running_nodes: Arc<Mutex<HashSet<String>>>,
        last_ocr: Arc<Mutex<String>>,
    ) -> Result<Self, String> {
        Ok(Self {
            app,
            input: Input::new()?,
            stop,
            methods,
            running_nodes,
            last_ocr,
            list_indices: HashMap::new(),
        })
    }

    fn last_ocr(&self) -> String {
        self.last_ocr
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default()
    }

    fn set_last_ocr(&self, text: &str) {
        if let Ok(mut value) = self.last_ocr.lock() {
            *value = text.to_string();
        }
    }

    /// Replaces `{{ocr}}` with the most recent OCR text.
    fn apply_ocr(&self, text: String) -> String {
        if text.contains("{{ocr}}") {
            text.replace("{{ocr}}", &self.last_ocr())
        } else {
            text
        }
    }

    fn eval_condition(&self, kind: &str, arg: &str, node: &Node) -> bool {
        match kind {
            "warframe_running" => win::find_warframe_pid().is_some(),
            "warframe_closed" => win::find_warframe_pid().is_none(),
            "nodes_running" => {
                let ids: Vec<String> = if arg.trim().is_empty() {
                    p_string_list(node, "nodes")
                } else {
                    arg.split(',')
                        .map(|part| part.trim().to_string())
                        .filter(|part| !part.is_empty())
                        .collect()
                };
                if ids.is_empty() {
                    false
                } else if p_str(node, "match", "any") == "all" {
                    ids.iter().all(|id| self.is_node_running(id))
                } else {
                    ids.iter().any(|id| self.is_node_running(id))
                }
            }
            "ocr_contains" => self.last_ocr().to_lowercase().contains(&arg.to_lowercase()),
            "ocr_not_contains" => !self.last_ocr().to_lowercase().contains(&arg.to_lowercase()),
            "ocr_equals" => self.last_ocr().trim().eq_ignore_ascii_case(arg.trim()),
            "always" => true,
            _ => false,
        }
    }

    fn mark_running(&self, id: &str, running: bool) {
        if let Ok(mut set) = self.running_nodes.lock() {
            if running {
                set.insert(id.to_string());
            } else {
                set.remove(id);
            }
        }
    }

    fn is_node_running(&self, id: &str) -> bool {
        self.running_nodes
            .lock()
            .map(|set| set.contains(id))
            .unwrap_or(false)
    }

    fn emit_timer(&self, node_id: &str, remaining_ms: i64) {
        let _ = self.app.emit(
            "flow-timer",
            TimerPayload {
                node: node_id.to_string(),
                remaining_ms,
            },
        );
    }

    /// Sleeps for `ms` while emitting the remaining time so the UI can show a
    /// live countdown on the node. Returns `false` if a stop was requested.
    fn sleep_with_timer(&self, ms: u64, node_id: &str) -> bool {
        let deadline = Instant::now() + Duration::from_millis(ms);
        loop {
            if self.stop.load(Ordering::Relaxed) {
                self.emit_timer(node_id, 0);
                return false;
            }
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            let remaining = deadline - now;
            self.emit_timer(node_id, remaining.as_millis() as i64);
            std::thread::sleep(remaining.min(Duration::from_millis(200)));
        }
        self.emit_timer(node_id, 0);
        true
    }

    pub fn run(&mut self, workflow: &Workflow) -> Result<(), String> {
        self.list_indices.clear();

        let nodes = workflow.nodes.clone();
        let connections = workflow.connections.clone();

        let start = match nodes.iter().find(|node| node.kind == "start") {
            Some(node) => node.id.clone(),
            None => return Err("Workflow has no Start node".into()),
        };

        self.run_nodes(&nodes, &connections, &start, None);
        Ok(())
    }

    fn run_nodes(
        &mut self,
        nodes: &[Node],
        connections: &[Connection],
        start: &str,
        end_at: Option<&str>,
    ) {
        let mut current = Some(start.to_string());
        let mut guard: u32 = 0;

        while let Some(id) = current {
            if self.stop.load(Ordering::Relaxed) {
                return;
            }
            if end_at == Some(id.as_str()) {
                return;
            }
            guard += 1;
            if guard > 250_000 {
                return;
            }

            let Some(node) = nodes.iter().find(|node| node.id == id).cloned() else {
                return;
            };

            let _ = self.app.emit("flow-node", &node.id);

            self.mark_running(&node.id, true);
            let outcome = self.exec_node(nodes, connections, &node);
            self.mark_running(&node.id, false);

            match outcome {
                Next::End => return,
                Next::Port(port) => {
                    let targets: Vec<String> = connections
                        .iter()
                        .filter(|connection| {
                            connection.from == node.id && connection.port() == port
                        })
                        .map(|connection| connection.to.clone())
                        .collect();

                    match targets.len() {
                        0 => return,
                        1 => current = Some(targets[0].clone()),
                        _ => {
                            // Several links from the same output run together,
                            // each as its own parallel branch.
                            self.run_branches(nodes, connections, &targets, end_at);
                            return;
                        }
                    }
                }
            }
        }
    }

    /// Runs several starting nodes concurrently, each on its own thread and
    /// input backend, and waits for them all to finish.
    fn run_branches(
        &self,
        nodes: &[Node],
        connections: &[Connection],
        targets: &[String],
        end_at: Option<&str>,
    ) {
        let mut handles = Vec::new();
        for target in targets {
            let app = self.app.clone();
            let stop = self.stop.clone();
            let methods = self.methods.clone();
            let running_nodes = self.running_nodes.clone();
            let last_ocr = self.last_ocr.clone();
            let branch_nodes = nodes.to_vec();
            let branch_connections = connections.to_vec();
            let branch_target = target.clone();
            let branch_end = end_at.map(|value| value.to_string());

            handles.push(std::thread::spawn(move || {
                if let Ok(mut executor) =
                    Executor::new(app, stop.clone(), methods, running_nodes, last_ocr)
                {
                    executor.run_nodes(
                        &branch_nodes,
                        &branch_connections,
                        &branch_target,
                        branch_end.as_deref(),
                    );
                }
            }));
        }

        for handle in handles {
            let _ = handle.join();
        }
    }

    fn exec_node(&mut self, nodes: &[Node], connections: &[Connection], node: &Node) -> Next {
        match node.kind.as_str() {
            "start" => Next::Port("out".into()),
            "end" => Next::End,

            "focus_warframe" => {
                let process = p_str(node, "process", "Warframe.x64");
                let title = p_str(node, "title", "");
                focus_window(&mut self.input, &process, &title);
                Next::Port("out".into())
            }

            "key_down" => {
                emit_combo(&mut self.input, &p_str(node, "key", ""), ComboMode::Down);
                Next::Port("out".into())
            }
            "key_up" => {
                emit_combo(&mut self.input, &p_str(node, "key", ""), ComboMode::Up);
                Next::Port("out".into())
            }
            "key_press" => {
                emit_combo(&mut self.input, &p_str(node, "key", ""), ComboMode::Press);
                Next::Port("out".into())
            }
            "shortcut" => {
                emit_combo(&mut self.input, &p_str(node, "keys", ""), ComboMode::Press);
                Next::Port("out".into())
            }
            "type_text" => {
                let text = self.apply_ocr(p_str(node, "text", ""));
                if !text.is_empty() {
                    self.input.text(&text);
                }
                Next::Port("out".into())
            }
            "send_messages" => {
                let list = p_string_list(node, "messages");
                if !list.is_empty() {
                    let random = p_str(node, "order", "sequential") == "random";
                    let message = if random {
                        list[random_range(0, list.len() as i32) as usize % list.len()].clone()
                    } else {
                        let index = self.list_indices.entry(node.id.clone()).or_insert(0);
                        let value = list[*index % list.len()].clone();
                        *index = (*index + 1) % list.len();
                        value
                    };
                    let message = self.apply_ocr(message);
                    self.input.text(&message);
                    sleep_rand(50, 100);
                }
                Next::Port("out".into())
            }
            "click" => {
                let button = parse_button(&p_str(node, "button", "left"));
                self.input.click_button(button);
                Next::Port("out".into())
            }
            "double_click" => {
                let button = parse_button(&p_str(node, "button", "left"));
                self.input.click_button(button);
                sleep_rand(40, 90);
                self.input.click_button(button);
                Next::Port("out".into())
            }
            "move_mouse" => {
                let mode = p_str(node, "mode", "warframe_random");
                if mode == "absolute" {
                    let x = p_i64(node, "x", 0) as i32;
                    let y = p_i64(node, "y", 0) as i32;
                    self.input.move_to(x, y);
                } else {
                    let range = p_i64(node, "range", 100).max(0) as i32;
                    if let Some((cx, cy)) = warframe_center() {
                        self.input.move_to(
                            random_range(cx - range, cx + range),
                            random_range(cy - range, cy + range),
                        );
                    }
                }
                Next::Port("out".into())
            }

            "delay" => {
                let min = p_i64(node, "min_ms", 250).max(0);
                let max = p_i64(node, "max_ms", 500).max(min);
                let millis = if max == min {
                    min
                } else {
                    random_range(min as i32, max as i32).max(0) as i64
                };
                if p_bool(node, "countdown", false) {
                    self.countdown(&node.id, millis, p_i64(node, "notify_secs", 5));
                } else {
                    self.sleep_with_timer(millis as u64, &node.id);
                }
                Next::Port("out".into())
            }
            "hold_key" => {
                let key_spec = p_str(node, "key", "");
                let min = p_i64(node, "min_ms", 250).max(0);
                let max = p_i64(node, "max_ms", 500).max(min);
                let millis = if max == min {
                    min
                } else {
                    random_range(min as i32, max as i32).max(0) as i64
                };

                emit_combo(&mut self.input, &key_spec, ComboMode::Down);
                let completed = self.sleep_with_timer(millis as u64, &node.id);
                emit_combo(&mut self.input, &key_spec, ComboMode::Up);

                if completed {
                    Next::Port("out".into())
                } else {
                    Next::End
                }
            }

            
            "condition" => {
                let mode = p_str(node, "mode", "");

                if mode == "switch" {
                    let cases = p_string_list(node, "cases");
                    for (index, case) in cases.iter().enumerate() {
                        let (kind, arg) = match case.split_once(':') {
                            Some((kind, arg)) => (kind.trim(), arg.trim()),
                            None => (case.trim(), ""),
                        };
                        if self.eval_condition(kind, arg, node) {
                            return Next::Port((index + 1).to_string());
                        }
                    }
                    return Next::Port("else".into());
                }

                let (kind, arg) = match mode.as_str() {
                    "warframe" => (
                        if p_str(node, "state", "running") == "closed" {
                            "warframe_closed"
                        } else {
                            "warframe_running"
                        },
                        String::new(),
                    ),
                    "nodes" => ("nodes_running", String::new()),
                    "ocr" => (
                        match p_str(node, "ocr_op", "contains").as_str() {
                            "equals" => "ocr_equals",
                            "not_contains" => "ocr_not_contains",
                            _ => "ocr_contains",
                        },
                        p_str(node, "value", ""),
                    ),
                    "always" => ("always", String::new()),
                    "never" => ("never", String::new()),
                    _ => ("", String::new()),
                };

                let result = !kind.is_empty() && self.eval_condition(kind, &arg, node);
                Next::Port(if result { "true" } else { "false" }.into())
            }
            "repeat" => {
                let count = p_i64(node, "count", 0);
                let interval_min = p_i64(node, "interval_min", 0).max(0);
                let interval_max = p_i64(node, "interval_max", 0).max(interval_min);

                let body = connections
                    .iter()
                    .find(|connection| connection.from == node.id && connection.port() == "body")
                    .map(|connection| connection.to.clone());

                let Some(body) = body else {
                    return Next::Port("done".into());
                };

                let mut iterations: i64 = 0;
                loop {
                    if self.stop.load(Ordering::Relaxed) {
                        return Next::End;
                    }
                    if count > 0 && iterations >= count {
                        break;
                    }

                    self.run_nodes(nodes, connections, &body, Some(&node.id));
                    iterations += 1;

                    // Wait between iterations (but not after the final one).
                    let has_more = count == 0 || iterations < count;
                    if has_more && interval_max > 0 {
                        let millis = if interval_max == interval_min {
                            interval_min
                        } else {
                            random_range(interval_min as i32, interval_max as i32).max(0) as i64
                        };
                        if !self.sleep_with_timer(millis as u64, &node.id) {
                            return Next::End;
                        }
                    }
                }
                Next::Port("done".into())
            }

            "call_method" => {
                let method_id = p_str(node, "method", "");
                if let Some((method_nodes, method_connections)) = self.method_graph(&method_id) {
                    if let Some(start) = method_nodes
                        .iter()
                        .find(|candidate| candidate.kind == "start")
                        .map(|candidate| candidate.id.clone())
                    {
                        self.run_nodes(&method_nodes, &method_connections, &start, None);
                    }
                }
                Next::Port("out".into())
            }
            "spawn_timer" => {
                let method_id = p_str(node, "method", "");
                let min = p_i64(node, "min_ms", 2500).max(1);
                let max = p_i64(node, "max_ms", 4000).max(min);

                if let Some((method_nodes, method_connections)) = self.method_graph(&method_id) {
                    let app = self.app.clone();
                    let stop = self.stop.clone();
                    let methods = self.methods.clone();
                    let running_nodes = self.running_nodes.clone();
                    let last_ocr = self.last_ocr.clone();
                    let timer_node = node.id.clone();

                    std::thread::spawn(move || {
                        let start = match method_nodes
                            .iter()
                            .find(|candidate| candidate.kind == "start")
                        {
                            Some(candidate) => candidate.id.clone(),
                            None => return,
                        };
                        let Ok(mut executor) =
                            Executor::new(app, stop.clone(), methods, running_nodes, last_ocr)
                        else {
                            return;
                        };

                        loop {
                            if stop.load(Ordering::Relaxed) {
                                break;
                            }
                            executor.run_nodes(&method_nodes, &method_connections, &start, None);
                            if stop.load(Ordering::Relaxed) {
                                break;
                            }
                            let millis = if max == min {
                                min
                            } else {
                                random_range(min as i32, max as i32).max(1) as i64
                            };
                            if !executor.sleep_with_timer(millis as u64, &timer_node) {
                                break;
                            }
                        }
                    });
                }
                Next::Port("out".into())
            }

            "read_text" => {
                let x = p_i64(node, "x", 0) as i32;
                let y = p_i64(node, "y", 0) as i32;
                let width = p_i64(node, "width", 200).max(1) as i32;
                let height = p_i64(node, "height", 60).max(1) as i32;
                let allowed = p_str(node, "allowed", "");
                let detection = p_str(node, "detection_model", "");
                let recognition = p_str(node, "recognition_model", "");
                let options = crate::ocr::PreprocessOptions {
                    invert: crate::ocr::InvertMode::parse(&p_str(node, "invert", "auto")),
                    scale: p_i64(node, "scale", 1).clamp(1, 4) as u32,
                    threshold: p_bool(node, "threshold", false),
                    color: crate::ocr::parse_color(&p_str(node, "color", "")),
                    tolerance: p_i64(node, "tolerance", 60).clamp(0, 255) as u32,
                    allowed_chars: if allowed.is_empty() { None } else { Some(allowed) },
                    detection_model: if detection.is_empty() {
                        None
                    } else {
                        Some(detection)
                    },
                    recognition_model: if recognition.is_empty() {
                        None
                    } else {
                        Some(recognition)
                    },
                };

                match crate::ocr::read_text(x, y, width, height, &options) {
                    Ok(text) => {
                        let text = text.trim().to_string();
                        self.set_last_ocr(&text);
                        let _ = self.app.emit(
                            "ocr-text",
                            OcrPayload {
                                node: node.id.clone(),
                                text: text.clone(),
                            },
                        );
                        set_status(&self.app, &format!("OCR: {text}"));
                        // The result is always "sent" to the next nodes: it is
                        // available as {{ocr}} and to Condition OCR checks.
                    }
                    Err(error) => {
                        let _ = self.app.emit(
                            "ocr-text",
                            OcrPayload {
                                node: node.id.clone(),
                                text: format!("Error: {error}"),
                            },
                        );
                    }
                }
                Next::Port("out".into())
            }
            "notify" => {
                notify(
                    &self.app,
                    &p_str(node, "title", "SleepFrame"),
                    &p_str(node, "body", ""),
                );
                Next::Port("out".into())
            }

            _ => Next::Port("out".into()),
        }
    }

    fn method_graph(&self, method_id: &str) -> Option<(Vec<Node>, Vec<Connection>)> {
        if method_id.is_empty() {
            return None;
        }
        self.methods
            .iter()
            .find(|method| method.id == method_id)
            .map(|method| (method.nodes.clone(), method.connections.clone()))
    }

    fn countdown(&self, node_id: &str, total_ms: i64, notify_secs: i64) {
        let total = (total_ms / 1000).max(0);
        let mut elapsed: i64 = 0;

        loop {
            if self.stop.load(Ordering::Relaxed) {
                self.emit_timer(node_id, 0);
                return;
            }
            let remaining = (total - elapsed).max(0);
            let text = format!("Next run in {}", format_seconds(remaining));
            set_status(&self.app, &text);
            self.emit_timer(node_id, remaining * 1000);

            if notify_secs > 0 && remaining == notify_secs {
                notify(&self.app, "SleepFrame", &text);
            }
            if remaining == 0 {
                self.emit_timer(node_id, 0);
                return;
            }
            if !sleep_until_stopped(&self.stop, 1000) {
                self.emit_timer(node_id, 0);
                return;
            }
            elapsed += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn p_str(node: &Node, key: &str, default: &str) -> String {
    node.params
        .get(key)
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| default.to_string())
}

fn p_i64(node: &Node, key: &str, default: i64) -> i64 {
    node.params
        .get(key)
        .and_then(|value| value.as_i64())
        .or_else(|| {
            node.params
                .get(key)
                .and_then(|value| value.as_f64())
                .map(|value| value as i64)
        })
        .unwrap_or(default)
}

fn p_bool(node: &Node, key: &str, default: bool) -> bool {
    node.params
        .get(key)
        .and_then(|value| value.as_bool())
        .unwrap_or(default)
}

fn p_string_list(node: &Node, key: &str) -> Vec<String> {
    node.params
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(|text| text.to_string()))
                .filter(|text| !text.trim().is_empty())
                .collect()
        })
        .unwrap_or_default()
}

enum ComboMode {
    Down,
    Up,
    Press,
}

fn is_modifier(key: &Key) -> bool {
    matches!(key, Key::Control | Key::Shift | Key::Alt | Key::Meta)
}

/// Parses a shortcut such as `Ctrl+Shift+A` into an ordered list of keys
/// (modifiers first, main key last).
fn parse_combo(spec: &str) -> Option<Vec<Key>> {
    let mut keys = Vec::new();
    for part in spec.split('+').map(str::trim).filter(|part| !part.is_empty()) {
        keys.push(parse_key(part)?);
    }
    if keys.is_empty() {
        return None;
    }
    // Normalise so modifiers are held before the main key.
    keys.sort_by_key(|key| if is_modifier(key) { 0 } else { 1 });
    Some(keys)
}

fn emit_combo(input: &mut Input, spec: &str, mode: ComboMode) {
    let Some(keys) = parse_combo(spec) else {
        return;
    };

    match mode {
        ComboMode::Down => {
            for key in &keys {
                input.key(key.clone(), Direction::Press);
                sleep_rand(15, 35);
            }
        }
        ComboMode::Up => {
            for key in keys.iter().rev() {
                input.key(key.clone(), Direction::Release);
                sleep_rand(15, 35);
            }
        }
        ComboMode::Press => {
            if keys.len() == 1 {
                input.key(keys[0].clone(), Direction::Click);
                return;
            }
            let (main, modifiers) = keys.split_last().unwrap();
            for key in modifiers {
                input.key(key.clone(), Direction::Press);
                sleep_rand(15, 35);
            }
            input.key(main.clone(), Direction::Click);
            for key in modifiers.iter().rev() {
                input.key(key.clone(), Direction::Release);
                sleep_rand(15, 35);
            }
        }
    }
}

fn parse_key(name: &str) -> Option<Key> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }

    match trimmed.to_ascii_lowercase().as_str() {
        "ctrl" | "control" | "lcontrol" | "leftcontrol" => Some(Key::Control),
        "shift" | "lshift" | "leftshift" => Some(Key::Shift),
        "alt" | "lalt" | "leftalt" => Some(Key::Alt),
        "win" | "meta" | "super" | "cmd" | "command" | "lwin" => Some(Key::Meta),
        "space" | "spacebar" => Some(Key::Space),
        "enter" | "return" => Some(Key::Return),
        "backspace" | "back" => Some(Key::Backspace),
        "tab" => Some(Key::Tab),
        "escape" | "esc" => Some(Key::Escape),
        "capslock" | "caps" => Some(Key::CapsLock),
        "up" | "uparrow" => Some(Key::UpArrow),
        "down" | "downarrow" => Some(Key::DownArrow),
        "left" | "leftarrow" => Some(Key::LeftArrow),
        "right" | "rightarrow" => Some(Key::RightArrow),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" | "pgup" => Some(Key::PageUp),
        "pagedown" | "pgdn" => Some(Key::PageDown),
        "insert" | "ins" => Some(Key::Insert),
        "delete" | "del" => Some(Key::Delete),
        "printscreen" | "printscr" | "print" => Some(Key::PrintScr),
        "pause" => Some(Key::Pause),
        "scrolllock" => Some(Key::Scroll),
        "comma" => Some(Key::Unicode(',')),
        "period" | "dot" => Some(Key::Unicode('.')),
        "slash" => Some(Key::Unicode('/')),
        "backslash" => Some(Key::Unicode('\\')),
        "semicolon" => Some(Key::Unicode(';')),
        "quote" | "apostrophe" => Some(Key::Unicode('\'')),
        "backtick" | "grave" => Some(Key::Unicode('`')),
        "minus" | "dash" => Some(Key::Unicode('-')),
        "equals" => Some(Key::Unicode('=')),
        _ => {
            if let Some(number) = trimmed
                .strip_prefix('f')
                .or_else(|| trimmed.strip_prefix('F'))
                .and_then(|rest| rest.parse::<u8>().ok())
            {
                return function_key(number);
            }
            let mut chars = trimmed.chars();
            match (chars.next(), chars.next()) {
                (Some(ch), None) => Some(Key::Unicode(ch)),
                _ => None,
            }
        }
    }
}

fn function_key(number: u8) -> Option<Key> {
    Some(match number {
        1 => Key::F1,
        2 => Key::F2,
        3 => Key::F3,
        4 => Key::F4,
        5 => Key::F5,
        6 => Key::F6,
        7 => Key::F7,
        8 => Key::F8,
        9 => Key::F9,
        10 => Key::F10,
        11 => Key::F11,
        12 => Key::F12,
        13 => Key::F13,
        14 => Key::F14,
        15 => Key::F15,
        16 => Key::F16,
        17 => Key::F17,
        18 => Key::F18,
        19 => Key::F19,
        20 => Key::F20,
        21 => Key::F21,
        22 => Key::F22,
        23 => Key::F23,
        24 => Key::F24,
        _ => return None,
    })
}

fn parse_button(name: &str) -> Button {
    match name {
        "right" => Button::Right,
        "middle" => Button::Middle,
        _ => Button::Left,
    }
}

fn warframe_center() -> Option<(i32, i32)> {
    let pid = win::find_warframe_pid()?;
    let hwnd = win::main_window(pid)?;
    let rect = win::window_rect(hwnd)?;
    Some(rect.center())
}

/// Brings a window into focus, nudges the cursor near its centre and clicks
/// once. The window is matched by title (if given) or by process name.
fn focus_window(input: &mut Input, process: &str, title: &str) {
    let window = if title.trim().is_empty() {
        win::find_process_pid(process).and_then(win::main_window)
    } else {
        win::find_window_by_title(title)
    };

    let Some(hwnd) = window else {
        return;
    };
    let Some(rect) = win::window_rect(hwnd) else {
        return;
    };
    let (center_x, center_y) = rect.center();

    win::set_foreground(hwnd);
    sleep_rand(50, 100);

    input.move_to(
        random_range(center_x - 100, center_x + 100),
        random_range(center_y - 100, center_y + 100),
    );
    sleep_rand(250, 500);

    input.click_button(Button::Left);
}

pub fn format_seconds(total: i64) -> String {
    let total = total.max(0);
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
