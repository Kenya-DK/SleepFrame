export type ParamKind =
  | "text"
  | "number"
  | "select"
  | "checkbox"
  | "shortcut"
  | "list"
  | "nodes"
  | "duration";

export interface ParamSpec {
  key: string;
  label: string;
  kind: ParamKind;
  default: unknown;
  options: string[];
  min: number | null;
  max: number | null;
  placeholder: string;
  group: string;
  inline: boolean;
  show_if_key?: string | null;
  show_if_value?: string | null;
}

export interface NodeSpec {
  kind: string;
  label: string;
  category: string;
  description: string;
  help: string;
  color: string;
  outputs: string[];
  params: ParamSpec[];
}

export interface FlowNode {
  id: string;
  kind: string;
  name: string;
  x: number;
  y: number;
  params: Record<string, unknown>;
}

export interface Connection {
  from: string;
  to: string;
  from_port?: string | null;
}

export interface Workflow {
  id: string;
  name: string;
  description: string;
  accent: string;
  notify_time_secs: number;
  nodes: FlowNode[];
  connections: Connection[];
  builtin: boolean;
}

export interface Method {
  id: string;
  name: string;
  description: string;
  nodes: FlowNode[];
  connections: Connection[];
  builtin: boolean;
}

export interface WorkflowSummary {
  id: string;
  name: string;
  description: string;
  accent: string;
  builtin: boolean;
  node_count: number;
}

export interface MethodSummary {
  id: string;
  name: string;
  description: string;
  builtin: boolean;
  node_count: number;
}

export interface HotkeyConfig {
  modifier: string;
  key: string;
}

export interface StateDto {
  selected: string;
  running: boolean;
  status: string;
  workflows: WorkflowSummary[];
  methods: MethodSummary[];
  hotkey: HotkeyConfig;
}

export interface CursorPos {
  x: number;
  y: number;
}

export interface PickResult {
  x: number;
  y: number;
  clicked: boolean;
}

export interface RegionSelection {
  x: number;
  y: number;
  width: number;
  height: number;
}
