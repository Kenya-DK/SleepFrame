import { invoke } from "@tauri-apps/api/core";

import type {
  Method,
  NodeSpec,
  PickResult,
  StateDto,
  Workflow,
} from "./types";

export const getState = () => invoke<StateDto>("get_state");

export const getNodeSpecs = () => invoke<NodeSpec[]>("get_node_specs");

export const selectWorkflow = (id: string) =>
  invoke<void>("select_workflow", { id });

export const startFlow = () => invoke<void>("start_flow");

export const stopFlow = () => invoke<void>("stop_flow");

export const toggleFlow = () => invoke<void>("toggle_flow");

export const setHotkey = (modifier: string, key: string) =>
  invoke<void>("set_hotkey", { modifier, key });

export const getWorkflow = (id: string) =>
  invoke<Workflow | null>("get_workflow", { id });

export const createWorkflow = (name: string) =>
  invoke<Workflow>("create_workflow", { name });

export const saveWorkflow = (workflow: Workflow) =>
  invoke<void>("save_workflow", { workflow });

export const deleteWorkflow = (id: string) =>
  invoke<void>("delete_workflow", { id });

export const resetWorkflow = (id: string) =>
  invoke<Workflow>("reset_workflow", { id });

export const getMethod = (id: string) =>
  invoke<Method | null>("get_method", { id });

export const createMethod = (name: string) =>
  invoke<Method>("create_method", { name });

export const saveMethod = (method: Method) =>
  invoke<void>("save_method", { method });

export const deleteMethod = (id: string) =>
  invoke<void>("delete_method", { id });

export const resetMethod = (id: string) =>
  invoke<Method>("reset_method", { id });

export const duplicateMethod = (id: string) =>
  invoke<Method>("duplicate_method", { id });

export const exportWorkflow = (id: string) =>
  invoke<string>("export_workflow", { id });

export const importWorkflow = (code: string) =>
  invoke<Workflow>("import_workflow", { code });

export const getForegroundWindow = () =>
  invoke<{ process: string; title: string }>("foreground_window_info");

export const startPick = () => invoke<void>("start_pick");

export const stopPick = () => invoke<void>("stop_pick");

export const pollPick = () => invoke<PickResult>("poll_pick");

export const startRegionSelect = () => invoke<void>("start_region_select");

export const submitRegion = (
  x: number,
  y: number,
  width: number,
  height: number,
) => invoke<void>("submit_region", { x, y, width, height });

export const cancelRegionSelect = () => invoke<void>("cancel_region_select");

export const previewRegion = (
  x: number,
  y: number,
  width: number,
  height: number,
  invert: string,
  scale: number,
  threshold: boolean,
  color: string,
  tolerance: number,
) =>
  invoke<string>("preview_region", {
    x,
    y,
    width,
    height,
    invert,
    scale,
    threshold,
    color,
    tolerance,
  });
