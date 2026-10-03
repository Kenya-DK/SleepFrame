//! Application state and on-disk persistence for workflows and methods.
//!
//! Built-in defaults are generated in code; user edits and new items are stored
//! as JSON under the OS app-data directory and shadow the built-ins by id.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::workflow::{
    self, Method, MethodSummary, Node, ShareBundle, Workflow, WorkflowSummary,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    #[serde(default = "default_modifier")]
    pub modifier: String,
    #[serde(default = "default_key")]
    pub key: String,
}

fn default_modifier() -> String {
    "None".to_string()
}

fn default_key() -> String {
    "F3".to_string()
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifier: default_modifier(),
            key: default_key(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    #[serde(default = "default_modifier")]
    modifier: String,
    #[serde(default = "default_key")]
    key: String,
    #[serde(default = "default_selected")]
    selected: String,
}

fn default_selected() -> String {
    "trading".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            modifier: default_modifier(),
            key: default_key(),
            selected: default_selected(),
        }
    }
}

pub struct Engine {
    pub dir: PathBuf,
    pub workflows: Vec<Workflow>,
    pub methods: Vec<Method>,
    pub selected: String,
    pub running: bool,
    pub status: String,
    pub stop: Arc<AtomicBool>,
    pub hotkey: HotkeyConfig,
}

impl Engine {
    pub fn new(dir: PathBuf) -> Self {
        let config = load_app_config(&dir);
        let mut engine = Self {
            dir,
            workflows: Vec::new(),
            methods: Vec::new(),
            selected: config.selected,
            running: false,
            status: "Idle".to_string(),
            stop: Arc::new(AtomicBool::new(false)),
            hotkey: HotkeyConfig {
                modifier: config.modifier,
                key: config.key,
            },
        };
        engine.reload();
        if !engine.workflows.iter().any(|w| w.id == engine.selected) {
            engine.selected = engine
                .workflows
                .first()
                .map(|w| w.id.clone())
                .unwrap_or_default();
        }
        engine
    }

    fn workflows_dir(&self) -> PathBuf {
        self.dir.join("workflows")
    }

    fn methods_dir(&self) -> PathBuf {
        self.dir.join("methods")
    }

    pub fn reload(&mut self) {
        let mut workflows = workflow::builtin_workflows();
        for stored in load_dir::<Workflow>(&self.workflows_dir()) {
            let id = stored.id.clone();
            if let Some(existing) = workflows.iter_mut().find(|item| item.id == id) {
                *existing = stored;
                existing.builtin = workflow::is_builtin_workflow(&id);
            } else {
                workflows.push(stored);
            }
        }
        self.workflows = workflows;

        let mut methods = workflow::builtin_methods();
        for stored in load_dir::<Method>(&self.methods_dir()) {
            let id = stored.id.clone();
            if let Some(existing) = methods.iter_mut().find(|item| item.id == id) {
                *existing = stored;
                existing.builtin = workflow::is_builtin_method(&id);
            } else {
                methods.push(stored);
            }
        }
        self.methods = methods;
    }

    pub fn workflow(&self, id: &str) -> Option<Workflow> {
        self.workflows.iter().find(|w| w.id == id).cloned()
    }

    pub fn method(&self, id: &str) -> Option<Method> {
        self.methods.iter().find(|m| m.id == id).cloned()
    }

    pub fn set_selected(&mut self, id: &str) {
        if self.workflows.iter().any(|w| w.id == id) {
            self.selected = id.to_string();
            self.save_app_config();
        }
    }

    pub fn save_workflow(&mut self, mut workflow: Workflow) -> Result<(), String> {
        std::fs::create_dir_all(self.workflows_dir()).map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(&workflow).map_err(|e| e.to_string())?;
        let path = self.workflows_dir().join(format!("{}.json", workflow.id));
        std::fs::write(path, content).map_err(|e| e.to_string())?;

        if let Some(existing) = self.workflows.iter_mut().find(|w| w.id == workflow.id) {
            workflow.builtin = existing.builtin;
            *existing = workflow;
        } else {
            workflow.builtin = workflow::is_builtin_workflow(&workflow.id);
            self.workflows.push(workflow);
        }
        Ok(())
    }

    pub fn create_workflow(&mut self, name: &str) -> Result<Workflow, String> {
        let name = if name.trim().is_empty() {
            "New macro"
        } else {
            name.trim()
        };
        let id = self.unique_workflow_id(name);
        let workflow = workflow::blank_workflow(id, name.to_string());
        self.save_workflow(workflow.clone())?;
        self.selected = workflow.id.clone();
        self.save_app_config();
        Ok(workflow)
    }

    fn unique_workflow_id(&self, name: &str) -> String {
        let base = workflow::slugify(name);
        let mut candidate = base.clone();
        let mut counter = 1;
        while self.workflows.iter().any(|w| w.id == candidate) {
            candidate = format!("{base}-{counter}");
            counter += 1;
        }
        candidate
    }

    pub fn delete_workflow(&mut self, id: &str) -> Result<(), String> {
        if workflow::is_builtin_workflow(id) {
            return Err("Built-in macros can only be reset, not deleted".into());
        }
        let path = self.workflows_dir().join(format!("{id}.json"));
        let _ = std::fs::remove_file(path);
        self.workflows.retain(|w| w.id != id);
        if self.selected == id {
            self.selected = self
                .workflows
                .first()
                .map(|w| w.id.clone())
                .unwrap_or_default();
            self.save_app_config();
        }
        Ok(())
    }

    pub fn reset_workflow(&mut self, id: &str) -> Result<Workflow, String> {
        let path = self.workflows_dir().join(format!("{id}.json"));
        let _ = std::fs::remove_file(path);
        self.reload();
        self.workflow(id)
            .ok_or_else(|| "Workflow not found".to_string())
    }

    pub fn save_method(&mut self, mut method: Method) -> Result<(), String> {
        std::fs::create_dir_all(self.methods_dir()).map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(&method).map_err(|e| e.to_string())?;
        let path = self.methods_dir().join(format!("{}.json", method.id));
        std::fs::write(path, content).map_err(|e| e.to_string())?;

        if let Some(existing) = self.methods.iter_mut().find(|m| m.id == method.id) {
            method.builtin = existing.builtin;
            *existing = method;
        } else {
            method.builtin = workflow::is_builtin_method(&method.id);
            self.methods.push(method);
        }
        Ok(())
    }

    pub fn create_method(&mut self, name: &str) -> Result<Method, String> {
        let name = if name.trim().is_empty() {
            "New method"
        } else {
            name.trim()
        };
        let id = self.unique_method_id(name);
        let method = workflow::blank_method(id, name.to_string());
        self.save_method(method.clone())?;
        Ok(method)
    }

    fn unique_method_id(&self, name: &str) -> String {
        let base = workflow::slugify(name);
        let mut candidate = base.clone();
        let mut counter = 1;
        while self.methods.iter().any(|m| m.id == candidate) {
            candidate = format!("{base}-{counter}");
            counter += 1;
        }
        candidate
    }

    pub fn delete_method(&mut self, id: &str) -> Result<(), String> {
        if workflow::is_builtin_method(id) {
            return Err("Built-in methods can only be reset, not deleted".into());
        }
        let path = self.methods_dir().join(format!("{id}.json"));
        let _ = std::fs::remove_file(path);
        self.methods.retain(|m| m.id != id);
        Ok(())
    }

    pub fn reset_method(&mut self, id: &str) -> Result<Method, String> {
        let path = self.methods_dir().join(format!("{id}.json"));
        let _ = std::fs::remove_file(path);
        self.reload();
        self.method(id)
            .ok_or_else(|| "Method not found".to_string())
    }

    pub fn duplicate_method(&mut self, id: &str) -> Result<Method, String> {
        let source = self.method(id).ok_or_else(|| "Method not found".to_string())?;
        let name = format!("{} copy", source.name);
        let mut method = source;
        method.id = self.unique_method_id(&name);
        method.name = name;
        method.builtin = false;
        self.save_method(method.clone())?;
        Ok(method)
    }

    pub fn export_workflow(&self, id: &str) -> Result<ShareBundle, String> {
        let workflow = self.workflow(id).ok_or_else(|| "Workflow not found".to_string())?;
        let methods = workflow::referenced_methods(&workflow, &self.methods);
        Ok(ShareBundle {
            version: 1,
            workflow,
            methods,
        })
    }

    pub fn import_workflow(&mut self, mut bundle: ShareBundle) -> Result<Workflow, String> {
        let mut remap: Vec<(String, String)> = Vec::new();
        for method in &mut bundle.methods {
            if self.methods.iter().any(|existing| existing.id == method.id) {
                let new_id = workflow::short_id("method");
                remap.push((method.id.clone(), new_id.clone()));
                method.id = new_id;
            }
            method.builtin = false;
        }

        let apply_remap = |nodes: &mut Vec<Node>, remap: &[(String, String)]| {
            for node in nodes.iter_mut() {
                if node.kind != "call_method" && node.kind != "spawn_timer" {
                    continue;
                }
                let current = node
                    .params
                    .get("method")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string());
                if let Some(current) = current {
                    if let Some((_, new_id)) =
                        remap.iter().find(|(old_id, _)| old_id == &current)
                    {
                        node.params["method"] = json!(new_id);
                    }
                }
            }
        };

        for method in &bundle.methods {
            let mut method = method.clone();
            apply_remap(&mut method.nodes, &remap);
            self.save_method(method)?;
        }

        let mut workflow = bundle.workflow;
        workflow.id = self.unique_workflow_id(&workflow.name);
        if self.workflows.iter().any(|w| w.name == workflow.name) {
            workflow.name = format!("{} (imported)", workflow.name);
        }
        workflow.builtin = false;
        apply_remap(&mut workflow.nodes, &remap);

        self.save_workflow(workflow.clone())?;
        self.selected = workflow.id.clone();
        self.save_app_config();
        Ok(workflow)
    }

    pub fn save_app_config(&self) {
        let _ = std::fs::create_dir_all(&self.dir);
        let config = AppConfig {
            modifier: self.hotkey.modifier.clone(),
            key: self.hotkey.key.clone(),
            selected: self.selected.clone(),
        };
        if let Ok(content) = serde_json::to_string_pretty(&config) {
            let _ = std::fs::write(self.dir.join("app.json"), content);
        }
    }

    pub fn request_stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn fresh_stop_flag(&mut self) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.stop = flag.clone();
        flag
    }

    pub fn workflow_summaries(&self) -> Vec<WorkflowSummary> {
        self.workflows.iter().map(WorkflowSummary::from).collect()
    }

    pub fn method_summaries(&self) -> Vec<MethodSummary> {
        self.methods.iter().map(MethodSummary::from).collect()
    }
}

pub struct AppState {
    pub engine: std::sync::Mutex<Engine>,
}

fn load_app_config(dir: &PathBuf) -> AppConfig {
    std::fs::read_to_string(dir.join("app.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<AppConfig>(&raw).ok())
        .unwrap_or_default()
}

fn load_dir<T: DeserializeOwned>(dir: &PathBuf) -> Vec<T> {
    let mut items = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return items;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(parsed) = serde_json::from_str::<T>(&raw) {
                items.push(parsed);
            }
        }
    }
    items
}
