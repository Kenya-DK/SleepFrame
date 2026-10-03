//! Node/workflow model, node registry and the built-in workflows/methods.
//!
//! A macro is a `Workflow`: a graph of `Node`s joined by `Connection`s. Nodes
//! are small reusable units (send a key, wait, branch, call a method, ...).
//! `Method`s are named, reusable sub-graphs that a workflow can call any number
//! of times, or run on their own background timer.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

fn empty_object() -> Value {
    json!({})
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default = "empty_object")]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: String,
    pub to: String,
    /// Which output port of `from` this connection leaves from. Defaults to
    /// `"out"`. Flow nodes such as `branch` and `repeat` expose extra ports.
    #[serde(default)]
    pub from_port: Option<String>,
}

impl Connection {
    pub fn port(&self) -> &str {
        self.from_port.as_deref().unwrap_or("out")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub accent: String,
    #[serde(default)]
    pub notify_time_secs: u64,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub connections: Vec<Connection>,
    /// True when a built-in default with this id exists (used for "reset").
    #[serde(default, skip_deserializing)]
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub connections: Vec<Connection>,
    #[serde(default, skip_deserializing)]
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub accent: String,
    pub builtin: bool,
    pub node_count: usize,
}

impl From<&Workflow> for WorkflowSummary {
    fn from(workflow: &Workflow) -> Self {
        Self {
            id: workflow.id.clone(),
            name: workflow.name.clone(),
            description: workflow.description.clone(),
            accent: workflow.accent.clone(),
            builtin: workflow.builtin,
            node_count: workflow.nodes.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MethodSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub builtin: bool,
    pub node_count: usize,
}

impl From<&Method> for MethodSummary {
    fn from(method: &Method) -> Self {
        Self {
            id: method.id.clone(),
            name: method.name.clone(),
            description: method.description.clone(),
            builtin: method.builtin,
            node_count: method.nodes.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareBundle {
    pub version: u32,
    pub workflow: Workflow,
    #[serde(default)]
    pub methods: Vec<Method>,
}

// ---------------------------------------------------------------------------
// Ids
// ---------------------------------------------------------------------------

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn short_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{:x}{:x}", nanos & 0xffff_ffff, count)
}

pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if (ch.is_whitespace() || ch == '-' || ch == '_') && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "macro".to_string()
    } else {
        trimmed
    }
}

// ---------------------------------------------------------------------------
// Node registry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ParamSpec {
    pub key: String,
    pub label: String,
    pub kind: String,
    pub default: Value,
    pub options: Vec<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub placeholder: String,
    /// Optional section name; params with the same group are shown together.
    #[serde(default)]
    pub group: String,
    /// When true, this param can share a row with the next inline param.
    #[serde(default)]
    pub inline: bool,
    /// When set, this param is only shown while `show_if_key` equals `show_if_value`.
    #[serde(default)]
    pub show_if_key: Option<String>,
    #[serde(default)]
    pub show_if_value: Option<String>,
}

impl ParamSpec {
    fn group(mut self, group: &str) -> Self {
        self.group = group.to_string();
        self
    }

    fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    fn show_if(mut self, key: &str, value: &str) -> Self {
        self.show_if_key = Some(key.to_string());
        self.show_if_value = Some(value.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeSpec {
    pub kind: String,
    pub label: String,
    pub category: String,
    pub description: String,
    /// Longer "how it works" explanation shown in the node reference.
    pub help: String,
    pub color: String,
    pub outputs: Vec<String>,
    pub params: Vec<ParamSpec>,
}

fn text(key: &str, label: &str, default: &str, placeholder: &str) -> ParamSpec {
    ParamSpec {
        key: key.into(),
        label: label.into(),
        kind: "text".into(),
        default: json!(default),
        options: vec![],
        min: None,
        max: None,
        placeholder: placeholder.into(),
        group: String::new(),
        inline: false,
        show_if_key: None,
        show_if_value: None,
    }
}

fn number(key: &str, label: &str, default: f64, min: f64, max: f64) -> ParamSpec {
    ParamSpec {
        key: key.into(),
        label: label.into(),
        kind: "number".into(),
        default: json!(default),
        options: vec![],
        min: Some(min),
        max: Some(max),
        placeholder: String::new(),
        group: String::new(),
        inline: false,
        show_if_key: None,
        show_if_value: None,
    }
}

fn select(key: &str, label: &str, default: &str, options: &[&str]) -> ParamSpec {
    ParamSpec {
        key: key.into(),
        label: label.into(),
        kind: "select".into(),
        default: json!(default),
        options: options.iter().map(|o| o.to_string()).collect(),
        min: None,
        max: None,
        placeholder: String::new(),
        group: String::new(),
        inline: false,
        show_if_key: None,
        show_if_value: None,
    }
}

/// A duration parameter stored as milliseconds, edited with a unit picker.
fn duration(key: &str, label: &str, default_ms: f64) -> ParamSpec {
    ParamSpec {
        key: key.into(),
        label: label.into(),
        kind: "duration".into(),
        default: json!(default_ms),
        options: vec![],
        min: Some(0.0),
        max: None,
        placeholder: String::new(),
        group: String::new(),
        inline: false,
        show_if_key: None,
        show_if_value: None,
    }
}

fn node_list(key: &str, label: &str) -> ParamSpec {
    ParamSpec {
        key: key.into(),
        label: label.into(),
        kind: "nodes".into(),
        default: json!([]),
        options: vec![],
        min: None,
        max: None,
        placeholder: String::new(),
        group: String::new(),
        inline: false,
        show_if_key: None,
        show_if_value: None,
    }
}

fn string_list(key: &str, label: &str) -> ParamSpec {
    ParamSpec {
        key: key.into(),
        label: label.into(),
        kind: "list".into(),
        default: json!([]),
        options: vec![],
        min: None,
        max: None,
        placeholder: "Type a value and press Enter".into(),
        group: String::new(),
        inline: false,
        show_if_key: None,
        show_if_value: None,
    }
}

fn shortcut(key: &str, label: &str, default: &str) -> ParamSpec {
    ParamSpec {
        key: key.into(),
        label: label.into(),
        kind: "shortcut".into(),
        default: json!(default),
        options: vec![],
        min: None,
        max: None,
        placeholder: "Ctrl+Shift+A".into(),
        group: String::new(),
        inline: false,
        show_if_key: None,
        show_if_value: None,
    }
}

fn checkbox(key: &str, label: &str, default: bool) -> ParamSpec {
    ParamSpec {
        key: key.into(),
        label: label.into(),
        kind: "checkbox".into(),
        default: json!(default),
        options: vec![],
        min: None,
        max: None,
        placeholder: String::new(),
        group: String::new(),
        inline: false,
        show_if_key: None,
        show_if_value: None,
    }
}

fn spec(
    kind: &str,
    label: &str,
    category: &str,
    description: &str,
    color: &str,
    outputs: &[&str],
    params: Vec<ParamSpec>,
) -> NodeSpec {
    NodeSpec {
        kind: kind.into(),
        label: label.into(),
        category: category.into(),
        description: description.into(),
        help: String::new(),
        color: color.into(),
        outputs: outputs.iter().map(|o| o.to_string()).collect(),
        params,
    }
}

/// Longer "how it works" text for the node reference.
fn node_help(kind: &str) -> &'static str {
    match kind {
        "start" => "The entry point. Every macro runs from here; exactly one Start is expected.",
        "end" => "Ends this branch of the workflow. Nothing after an End is reached.",
        "focus_warframe" => "Brings a window to the front, moves the cursor near its centre and left-clicks, so the game receives later input. Match by process name, or by window title.",
        "key_down" => "Presses and holds a key (or combination) until a matching Key up, or until the macro stops.",
        "key_up" => "Releases a key that was held with Key down.",
        "key_press" => "Presses and releases a key once. Accepts combinations like Ctrl+Space.",
        "shortcut" => "Sends a whole key combination: holds the modifiers, taps the main key, then releases in reverse.",
        "hold_key" => "Key down + wait + key up in one node. Presses a key or combo, waits a randomised duration, then releases.",
        "type_text" => "Types a literal string. Use the {{ocr}} placeholder to insert the most recent OCR result.",
        "send_messages" => "Types the next entry from a list of strings and cycles through them, or picks one at random.",
        "click" => "Clicks a mouse button once at the current cursor position (held briefly so games register it).",
        "double_click" => "Clicks a mouse button twice in quick succession.",
        "move_mouse" => "Moves the cursor to an absolute point, or to a random spot near the Warframe window centre.",
        "delay" => "Waits a randomised amount of time. With Countdown on it shows a running timer and can notify shortly before it ends.",
        "condition" => "Routes execution. Pick a Mode; each mode has its own settings. Simple modes route to true/false, while Switch mode has one output per case plus else.",
        "repeat" => "Repeats the connected body. Times 0 loops forever; add an interval to repeat every X time. The wait is skipped after the last iteration.",
        "call_method" => "Runs one of your reusable methods inline, then continues.",
        "spawn_timer" => "Runs a method repeatedly in the background on its own randomised interval, without blocking the rest of the macro.",
        "notify" => "Shows a desktop notification (tray balloon) with a title and body.",
        "read_text" => "Captures a screen region and reads the text in it with a bundled OCR engine. The result is sent to the next nodes. Disabled by default.",
        _ => "",
    }
}

const COLOR_TRIGGER: &str = "#34d399";
const COLOR_INPUT: &str = "#38bdf8";
const COLOR_TIMING: &str = "#fbbf24";
const COLOR_FLOW: &str = "#a78bfa";
const COLOR_METHOD: &str = "#f472b6";
const COLOR_OUTPUT: &str = "#fb7185";

/// Feature flag for the Read text (OCR) node. Set to `false` to hide it.
pub const OCR_ENABLED: bool = true;

pub fn node_specs() -> Vec<NodeSpec> {
    let mut specs = vec![
        spec(
            "start",
            "Start",
            "trigger",
            "Entry point of the macro.",
            COLOR_TRIGGER,
            &["out"],
            vec![],
        ),
        spec(
            "end",
            "End",
            "trigger",
            "Stops this branch of the workflow.",
            COLOR_TRIGGER,
            &[],
            vec![],
        ),
        spec(
            "focus_warframe",
            "Focus window",
            "input",
            "Brings a window to the foreground, nudges the cursor and clicks once.",
            COLOR_INPUT,
            &["out"],
            vec![
                text("process", "Process", "Warframe.x64", "e.g. Warframe.x64, notepad"),
                text("title", "Window title (optional)", "", "match by title instead"),
            ],
        ),
        spec(
            "key_down",
            "Key down",
            "input",
            "Holds a key or shortcut down (e.g. Ctrl+Shift).",
            COLOR_INPUT,
            &["out"],
            vec![shortcut("key", "Key / shortcut", "Ctrl")],
        ),
        spec(
            "key_up",
            "Key up",
            "input",
            "Releases a key or shortcut.",
            COLOR_INPUT,
            &["out"],
            vec![shortcut("key", "Key / shortcut", "Space")],
        ),
        spec(
            "key_press",
            "Press key",
            "input",
            "Presses and releases a key once. Accepts shortcuts like Ctrl+Space.",
            COLOR_INPUT,
            &["out"],
            vec![shortcut("key", "Key / shortcut", "Return")],
        ),
        spec(
            "shortcut",
            "Shortcut",
            "input",
            "Presses a full key combination in one go, e.g. Ctrl+Shift+A or Alt+F4.",
            COLOR_INPUT,
            &["out"],
            vec![shortcut("keys", "Shortcut", "Ctrl+Shift+A")],
        ),
        spec(
            "type_text",
            "Type text",
            "input",
            "Types a literal string of text.",
            COLOR_INPUT,
            &["out"],
            vec![text("text", "Text", "", "WTS …")],
        ),
        spec(
            "send_messages",
            "Send messages",
            "input",
            "Types the next entry from a list of strings. Add as many as you like.",
            COLOR_INPUT,
            &["out"],
            vec![
                string_list("messages", "Messages"),
                select("order", "Order", "sequential", &["sequential", "random"]),
            ],
        ),
        spec(
            "click",
            "Click",
            "input",
            "Clicks a mouse button.",
            COLOR_INPUT,
            &["out"],
            vec![select("button", "Button", "left", &["left", "right", "middle"])],
        ),
        spec(
            "double_click",
            "Double click",
            "input",
            "Double-clicks a mouse button.",
            COLOR_INPUT,
            &["out"],
            vec![select("button", "Button", "left", &["left", "right", "middle"])],
        ),
        spec(
            "move_mouse",
            "Move mouse",
            "input",
            "Moves the cursor to a point or to a random spot near the Warframe window centre.",
            COLOR_INPUT,
            &["out"],
            vec![
                select(
                    "mode",
                    "Mode",
                    "warframe_random",
                    &["warframe_random", "absolute"],
                ),
                number("x", "X", 0.0, -10000.0, 10000.0),
                number("y", "Y", 0.0, -10000.0, 10000.0),
                number("range", "Random range", 100.0, 0.0, 2000.0),
            ],
        ),
        spec(
            "delay",
            "Wait",
            "timing",
            "Waits a randomised amount of time. Enable countdown to show and notify a \"next run\" timer.",
            COLOR_TIMING,
            &["out"],
            vec![
                duration("min_ms", "Wait at least", 250.0),
                duration("max_ms", "Wait at most", 500.0),
                checkbox("countdown", "Countdown timer", false),
                number("notify_secs", "Notify seconds before", 5.0, 0.0, 600.0),
            ],
        ),
        spec(
            "hold_key",
            "Hold key",
            "input",
            "Holds a key (or shortcut) down for a duration, then releases it — key down + wait + key up in one node.",
            COLOR_INPUT,
            &["out"],
            vec![
                shortcut("key", "Key / shortcut", "Ctrl+Space"),
                duration("min_ms", "Hold at least", 250.0),
                duration("max_ms", "Hold at most", 500.0),
            ],
        ),

        spec(
            "condition",
            "Condition",
            "flow",
            "Pick a mode, then configure that mode's settings. Modes route to true/false (or one output per case in Switch mode).",
            COLOR_FLOW,
            &["true", "false"],
            vec![
                select("mode", "Mode", "", &["", "warframe", "nodes", "ocr", "switch"])
                    .group("Mode"),
                select("state", "State", "running", &["running", "closed"])
                    .group("Warframe")
                    .show_if("mode", "warframe"),
                node_list("nodes", "Nodes to watch")
                    .group("Nodes")
                    .show_if("mode", "nodes"),
                select("match", "Match", "any", &["any", "all"])
                    .group("Nodes")
                    .show_if("mode", "nodes"),
                select(
                    "ocr_op",
                    "Compare",
                    "contains",
                    &["contains", "equals", "not_contains"],
                )
                .group("OCR")
                .show_if("mode", "ocr"),
                text("value", "Value", "", "text to compare against the OCR result")
                    .group("OCR")
                    .show_if("mode", "ocr"),
                string_list("cases", "Cases (kind:arg, one per output)")
                    .group("Switch")
                    .show_if("mode", "switch"),
            ],
        ),
        spec(
            "repeat",
            "Repeat",
            "flow",
            "Repeats the connected body. Use 0 times for an endless loop, and an interval to repeat every X time.",
            COLOR_FLOW,
            &["body", "done"],
            vec![
                number("count", "Times (0 = forever)", 0.0, 0.0, 1_000_000.0),
                duration("interval_min", "Wait at least", 0.0),
                duration("interval_max", "Wait at most", 0.0),
            ],
        ),
        spec(
            "call_method",
            "Call method",
            "method",
            "Runs one of your reusable methods.",
            COLOR_METHOD,
            &["out"],
            vec![select("method", "Method", "", &[])],
        ),
        spec(
            "spawn_timer",
            "Method timer",
            "method",
            "Runs a method repeatedly in the background on its own randomised interval.",
            COLOR_METHOD,
            &["out"],
            vec![
                select("method", "Method", "", &[]),
                duration("min_ms", "Every (min)", 2500.0),
                duration("max_ms", "Every (max)", 4000.0),
            ],
        ),
        // "read_text" (OCR) is added below when `OCR_ENABLED` is true.
        spec(
            "notify",
            "Notification",
            "output",
            "Shows a desktop notification.",
            COLOR_OUTPUT,
            &["out"],
            vec![
                text("title", "Title", "SleepFrame", ""),
                text("body", "Body", "", "Message …"),
            ],
        ),
    ];

    if OCR_ENABLED {
        specs.push(read_text_spec());
    }

    for spec in &mut specs {
        spec.help = node_help(&spec.kind).to_string();
    }

    specs
}

#[allow(dead_code)]
fn read_text_spec() -> NodeSpec {
    spec(
        "read_text",
        "Read text (OCR)",
        "input",
        "Captures a screen region and reads the text in it. Uses a bundled OCR engine — nothing to install.",
        COLOR_INPUT,
        &["out"],
        vec![
            number("x", "X", 0.0, -10_000.0, 10_000.0).group("Region").inline(),
            number("y", "Y", 0.0, -10_000.0, 10_000.0).group("Region").inline(),
            number("width", "Width", 200.0, 1.0, 10_000.0).group("Region").inline(),
            number("height", "Height", 60.0, 1.0, 10_000.0).group("Region").inline(),
            select("invert", "Invert", "auto", &["auto", "on", "off"]).group("Preprocessing"),
            number("scale", "Upscale", 1.0, 1.0, 4.0).group("Preprocessing"),
            checkbox("threshold", "Binarize (Otsu)", false).group("Preprocessing"),
            text("color", "Only colour", "", "#bd2a2a (blank = off)").group("Colour filter"),
            number("tolerance", "Colour tolerance", 60.0, 0.0, 255.0).group("Colour filter"),
            text("allowed", "Only characters", "", "0123456789.%+- (blank = all)")
                .group("Characters"),
            text("detection_model", "Detection model", "", "path to .onnx/.rten (blank = built-in)")
                .group("Model"),
            text("recognition_model", "Recognition model", "", "path to .onnx/.rten (blank = built-in)")
                .group("Model"),
        ],
    )
}

// ---------------------------------------------------------------------------
// Built-in methods
// ---------------------------------------------------------------------------

fn node(id: &str, kind: &str, name: &str, x: f64, y: f64, params: Value) -> Node {
    Node {
        id: id.into(),
        kind: kind.into(),
        name: name.into(),
        x,
        y,
        params,
    }
}

fn conn(from: &str, port: &str, to: &str) -> Connection {
    Connection {
        from: from.into(),
        to: to.into(),
        from_port: Some(port.into()),
    }
}

pub fn builtin_methods() -> Vec<Method> {
    Vec::new()
}

// ---------------------------------------------------------------------------
// Built-in workflows
// ---------------------------------------------------------------------------

pub fn builtin_workflows() -> Vec<Workflow> {
    vec![trading_workflow(), index_workflow()]
}

fn trading_workflow() -> Workflow {
    Workflow {
        id: "trading".into(),
        name: "Trading".into(),
        description: "Cycles through your enabled trade messages in chat on a two-minute interval."
            .into(),
        accent: "#d4a537".into(),
        notify_time_secs: 5,
        nodes: vec![
            node("t-start", "start", "Start", 40.0, 40.0, json!({})),
            node(
                "t-repeat",
                "repeat",
                "Every run",
                280.0,
                40.0,
                json!({ "count": 0 }),
            ),
            node(
                "t-focus",
                "focus_warframe",
                "Focus Warframe",
                40.0,
                220.0,
                json!({}),
            ),
            node(
                "t-msg",
                "send_messages",
                "Trade messages",
                280.0,
                220.0,
                json!({ "messages": [], "order": "sequential" }),
            ),
            node(
                "t-enter",
                "key_press",
                "Send (Enter)",
                520.0,
                220.0,
                json!({ "key": "Return" }),
            ),
            node(
                "t-chat",
                "key_press",
                "Open chat (t)",
                760.0,
                220.0,
                json!({ "key": "t" }),
            ),
            node(
                "t-back",
                "key_press",
                "Clear t",
                1000.0,
                220.0,
                json!({ "key": "Backspace" }),
            ),
            node(
                "t-wait",
                "delay",
                "Wait 2 minutes",
                1240.0,
                220.0,
                json!({ "min_ms": 120000, "max_ms": 120000, "countdown": true, "notify_secs": 5 }),
            ),
        ],
        connections: vec![
            conn("t-start", "out", "t-repeat"),
            conn("t-repeat", "body", "t-focus"),
            conn("t-focus", "out", "t-msg"),
            conn("t-msg", "out", "t-enter"),
            conn("t-enter", "out", "t-chat"),
            conn("t-chat", "out", "t-back"),
            conn("t-back", "out", "t-wait"),
            conn("t-wait", "out", "t-repeat"),
        ],
        builtin: true,
    }
}

fn index_workflow() -> Workflow {
    Workflow {
        id: "the_index".into(),
        name: "The Index".into(),
        description: "Two modes run together: Protective Sling casts and operator-mode refreshes."
            .into(),
        accent: "#38bdf8".into(),
        notify_time_secs: 0,
        nodes: vec![
            node("i-start", "start", "Start", 60.0, 200.0, json!({})),
            node(
                "i-focus",
                "focus_warframe",
                "Focus Warframe",
                300.0,
                200.0,
                json!({}),
            ),
            node(
                "i-loop",
                "repeat",
                "Endless",
                540.0,
                200.0,
                json!({ "count": 0, "interval_min": 0, "interval_max": 0 }),
            ),
            node(
                "i-sling",
                "hold_key",
                "Protective Sling",
                540.0,
                370.0,
                json!({ "key": "Ctrl+Space", "min_ms": 300, "max_ms": 600 }),
            ),
            node(
                "i-wait1",
                "delay",
                "Wait 2.5–4s",
                780.0,
                370.0,
                json!({ "min_ms": 2500, "max_ms": 4000, "countdown": false, "notify_secs": 5 }),
            ),
            node(
                "i-op",
                "hold_key",
                "Operator mode",
                1020.0,
                370.0,
                json!({ "key": "5", "min_ms": 70, "max_ms": 130 }),
            ),
            node(
                "i-wait2",
                "delay",
                "Wait 15–20s",
                1260.0,
                370.0,
                json!({ "min_ms": 15000, "max_ms": 20000, "countdown": false, "notify_secs": 5 }),
            ),
        ],
        connections: vec![
            conn("i-start", "out", "i-focus"),
            conn("i-focus", "out", "i-loop"),
            conn("i-loop", "body", "i-sling"),
            conn("i-sling", "out", "i-wait1"),
            conn("i-wait1", "out", "i-op"),
            conn("i-op", "out", "i-wait2"),
            conn("i-wait2", "out", "i-loop"),
        ],
        builtin: true,
    }
}

pub fn is_builtin_workflow(id: &str) -> bool {
    ["trading", "the_index"].contains(&id)
}

pub fn is_builtin_method(_id: &str) -> bool {
    false
}

/// Creates an empty workflow with a start node.
pub fn blank_workflow(id: String, name: String) -> Workflow {
    Workflow {
        id,
        name,
        description: String::new(),
        accent: "#d4a537".into(),
        notify_time_secs: 0,
        nodes: vec![node("start", "start", "Start", 120.0, 160.0, json!({}))],
        connections: vec![],
        builtin: false,
    }
}

/// Creates a blank method with start/end nodes.
pub fn blank_method(id: String, name: String) -> Method {
    Method {
        id,
        name,
        description: String::new(),
        nodes: vec![
            node("start", "start", "Start", 80.0, 200.0, json!({})),
            node("end", "end", "End", 520.0, 200.0, json!({})),
        ],
        connections: vec![conn("start", "out", "end")],
        builtin: false,
    }
}

/// Collects the distinct methods referenced (directly) by a workflow.
pub fn referenced_methods(workflow: &Workflow, methods: &[Method]) -> Vec<Method> {
    let mut ids: Vec<String> = Vec::new();
    for node in &workflow.nodes {
        if node.kind != "call_method" && node.kind != "spawn_timer" {
            continue;
        }
        if let Some(id) = node.params.get("method").and_then(|v| v.as_str()) {
            if !id.is_empty() && !ids.iter().any(|existing| existing == id) {
                ids.push(id.to_string());
            }
        }
    }
    ids.iter()
        .filter_map(|id| methods.iter().find(|m| &m.id == id).cloned())
        .collect()
}
