import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
} from "react";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

import * as api from "../api";
import type {
  Connection,
  CursorPos,
  FlowNode,
  Method,
  MethodSummary,
  NodeSpec,
  ParamSpec,
  RegionSelection,
  Workflow,
} from "../types";
import { DurationField } from "./DurationField";
import { StringList } from "./StringList";

type FlowDoc = Workflow | Method;

const NODE_W = 250;
const NODE_H = 96;
const HEADER = 40;
const MIN_ZOOM = 0.3;
const MAX_ZOOM = 2.5;

const CATEGORY_LABELS: Record<string, string> = {
  trigger: "Triggers",
  input: "Inputs",
  timing: "Timing",
  flow: "Flow",
  method: "Methods",
  output: "Output",
};

const CATEGORY_ORDER = ["trigger", "input", "timing", "flow", "method", "output"];

function uid(prefix: string) {
  return `${prefix}-${Date.now().toString(36)}${Math.random().toString(36).slice(2, 7)}`;
}

function outputY(index: number, count: number) {
  return HEADER + ((index + 1) * (NODE_H - HEADER)) / (count + 1);
}

function formatTimer(milliseconds: number): string {
  if (milliseconds >= 60_000) {
    const total = Math.ceil(milliseconds / 1000);
    const minutes = Math.floor(total / 60);
    const seconds = total % 60;
    return `${minutes}:${String(seconds).padStart(2, "0")}`;
  }
  return `${(milliseconds / 1000).toFixed(1)}s`;
}

interface Props {
  kind: "workflow" | "method";
  id: string;
  reloadKey: number;
  specs: NodeSpec[];
  methods: MethodSummary[];
  activeNode: string | null;
  running: boolean;
  onSaved: () => void;
  onRun: () => void;
  onDelete: () => void;
  onReset: () => void;
}

export function FlowEditor({
  kind,
  id,
  reloadKey,
  specs,
  methods,
  activeNode,
  running,
  onSaved,
  onRun,
  onDelete,
  onReset,
}: Props) {
  const [doc, setDoc] = useState<FlowDoc | null>(null);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [pending, setPending] = useState<{ from: string; port: string } | null>(null);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [paletteQuery, setPaletteQuery] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [marquee, setMarquee] = useState<{
    x0: number;
    y0: number;
    x1: number;
    y1: number;
    additive: boolean;
  } | null>(null);

  const [zoom, setZoom] = useState(1);
  const [panning, setPanning] = useState(false);
  const [timers, setTimers] = useState<Record<string, number>>({});
  const [ocrTexts, setOcrTexts] = useState<Record<string, string>>({});
  const [regionTarget, setRegionTarget] = useState<string | null>(null);
  const regionTargetRef = useRef<string | null>(null);
  const hasDoc = doc !== null;

  const skipSave = useRef(true);
  const saveTimer = useRef<number | undefined>(undefined);
  const canvasRef = useRef<HTMLDivElement | null>(null);
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const zoomRef = useRef(1);
  zoomRef.current = zoom;
  const pan = useRef<{
    sx: number;
    sy: number;
    left: number;
    top: number;
  } | null>(null);
  const drag = useRef<{
    sx: number;
    sy: number;
    origin: Record<string, { x: number; y: number }>;
  } | null>(null);
  const onSavedRef = useRef(onSaved);
  onSavedRef.current = onSaved;
  const setParamsRef = useRef<(nodeId: string, patch: Record<string, unknown>) => void>(
    () => {},
  );

  const specByKind = useMemo(() => {
    const map: Record<string, NodeSpec> = {};
    for (const spec of specs) {
      map[spec.kind] = spec;
    }
    return map;
  }, [specs]);

  const outputsFor = (node: FlowNode): string[] => {
    if (node.kind === "condition" && node.params.mode === "switch") {
      const cases = asStringArray(node.params.cases);
      return [...cases.map((_, index) => String(index + 1)), "else"];
    }
    return specByKind[node.kind]?.outputs ?? ["out"];
  };

  useEffect(() => {
    let alive = true;
    skipSave.current = true;
    setDoc(null);
    setSelectedIds([]);
    setPending(null);
    setMarquee(null);
    setZoom(1);
    setTimers({});
    setOcrTexts({});
    setRegionTarget(null);
    regionTargetRef.current = null;
    void (async () => {
      const data =
        kind === "workflow" ? await api.getWorkflow(id) : await api.getMethod(id);
      if (alive) {
        setDoc(data ?? null);
      }
    })();
    return () => {
      alive = false;
    };
  }, [kind, id, reloadKey]);

  useEffect(() => {
    if (!doc) {
      return;
    }
    if (skipSave.current) {
      skipSave.current = false;
      return;
    }
    // Don't autosave while a macro is running (a run can update node params,
    // e.g. the OCR result, and we don't want to write the file mid-run).
    if (running) {
      setSaving(false);
      return;
    }
    window.clearTimeout(saveTimer.current);
    setSaving(true);
    saveTimer.current = window.setTimeout(async () => {
      try {
        if (kind === "workflow") {
          await api.saveWorkflow(doc as Workflow);
        } else {
          await api.saveMethod(doc as Method);
        }
        setError(null);
        onSavedRef.current();
      } catch (cause) {
        setError(String(cause));
      } finally {
        setSaving(false);
      }
    }, 450);
    return () => window.clearTimeout(saveTimer.current);
  }, [doc, kind, running]);

  const nodes = doc?.nodes ?? [];
  const connections = doc?.connections ?? [];
  const selectedSet = useMemo(() => new Set(selectedIds), [selectedIds]);
  const selectedNodes = nodes.filter((node) => selectedSet.has(node.id));
  const selected = selectedNodes.length === 1 ? selectedNodes[0] : null;
  const workflow = kind === "workflow" ? (doc as Workflow) : null;
  const accent = workflow?.accent || "#d4a537";

  const canvasWidth =
    nodes.reduce((max, node) => Math.max(max, node.x), 800) + NODE_W + 400;
  const canvasHeight =
    nodes.reduce((max, node) => Math.max(max, node.y), 500) + NODE_H + 400;

  const mutate = (patch: Partial<Workflow> & Partial<Method>) => {
    setDoc((current) =>
      current ? ({ ...current, ...patch } as FlowDoc) : current,
    );
  };

  const updateNodes = (updater: (current: FlowNode[]) => FlowNode[]) => {
    setDoc((current) =>
      current ? ({ ...current, nodes: updater(current.nodes) } as FlowDoc) : current,
    );
  };

  const addNode = (spec: NodeSpec) => {
    const params: Record<string, unknown> = {};
    for (const param of spec.params) {
      params[param.key] = param.default;
    }
    const count = nodes.length;
    const node: FlowNode = {
      id: uid(spec.kind),
      kind: spec.kind,
      name: spec.label,
      x: 120 + (count % 3) * 300,
      y: 180 + Math.floor(count / 3) * 170,
      params,
    };
    mutate({ nodes: [...nodes, node] });
    setSelectedIds([node.id]);
    setPaletteOpen(false);
  };

  const removeNodes = (ids: string[]) => {
    const remove = new Set(ids);
    setDoc((current) => {
      if (!current) {
        return current;
      }
      return {
        ...current,
        nodes: current.nodes.filter((node) => !remove.has(node.id)),
        connections: current.connections.filter(
          (connection) => !remove.has(connection.from) && !remove.has(connection.to),
        ),
      } as FlowDoc;
    });
    setSelectedIds((previous) => previous.filter((id) => !remove.has(id)));
  };

  const startRegion = (nodeId: string) => {
    regionTargetRef.current = nodeId;
    setRegionTarget(nodeId);
    api.startRegionSelect().catch((cause) => {
      setError(`Region overlay: ${String(cause)}`);
      regionTargetRef.current = null;
      setRegionTarget(null);
    });
  };

  const cancelRegion = () => {
    regionTargetRef.current = null;
    setRegionTarget(null);
    void api.cancelRegionSelect();
  };

  const duplicateSelection = () => {
    if (selectedIds.length === 0) {
      return;
    }
    const chosen = new Set(selectedIds);
    const idMap: Record<string, string> = {};
    const copies = nodes
      .filter((node) => chosen.has(node.id))
      .map((node) => {
        const newId = uid(node.kind);
        idMap[node.id] = newId;
        return {
          ...node,
          id: newId,
          x: node.x + 28,
          y: node.y + 28,
          params: JSON.parse(JSON.stringify(node.params)) as Record<string, unknown>,
        };
      });
    const internalConnections = connections
      .filter((connection) => chosen.has(connection.from) && chosen.has(connection.to))
      .map((connection) => ({
        from: idMap[connection.from],
        to: idMap[connection.to],
        from_port: connection.from_port ?? null,
      }));

    mutate({
      nodes: [...nodes, ...copies],
      connections: [...connections, ...internalConnections],
    });
    setSelectedIds(copies.map((copy) => copy.id));
  };

  const setParams = (nodeId: string, patch: Record<string, unknown>) => {
    updateNodes((list) =>
      list.map((node) =>
        node.id === nodeId
          ? { ...node, params: { ...node.params, ...patch } }
          : node,
      ),
    );
  };
  setParamsRef.current = setParams;

  const setNodeName = (nodeId: string, name: string) => {
    mutate({
      nodes: nodes.map((node) => (node.id === nodeId ? { ...node, name } : node)),
    });
  };

  const removeConnection = (index: number) => {
    mutate({ connections: connections.filter((_, i) => i !== index) });
  };

  const startConnect = (nodeId: string, port: string) => {
    setPending({ from: nodeId, port });
    setSelectedIds([nodeId]);
  };

  const onCanvasPointerDown = (event: ReactPointerEvent) => {
    if (event.target !== event.currentTarget || event.button !== 0) {
      return;
    }
    setPending(null);
    setPaletteOpen(false);

    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) {
      return;
    }
    const x = (event.clientX - rect.left) / zoom;
    const y = (event.clientY - rect.top) / zoom;
    setMarquee({
      x0: x,
      y0: y,
      x1: x,
      y1: y,
      additive: event.ctrlKey || event.metaKey || event.shiftKey,
    });
  };

  const onNodePointerDown = (event: ReactPointerEvent, node: FlowNode) => {
    if (event.button !== 0) {
      return;
    }
    const target = event.target as HTMLElement;
    if (target.closest("[data-port],button,input,select,textarea")) {
      return;
    }
    event.stopPropagation();

    if (pending && pending.from !== node.id) {
      const exists = connections.some(
        (connection) =>
          connection.from === pending.from &&
          (connection.from_port ?? "out") === pending.port &&
          connection.to === node.id,
      );
      if (!exists) {
        mutate({
          connections: [
            ...connections,
            { from: pending.from, to: node.id, from_port: pending.port },
          ],
        });
      }
      setPending(null);
      return;
    }

    const additive = event.ctrlKey || event.metaKey || event.shiftKey;
    const alreadySelected = selectedIds.includes(node.id);
    const next = alreadySelected
      ? selectedIds
      : additive
        ? [...selectedIds, node.id]
        : [node.id];
    setSelectedIds(next);

    const origin: Record<string, { x: number; y: number }> = {};
    for (const candidate of nodes) {
      if (next.includes(candidate.id)) {
        origin[candidate.id] = { x: candidate.x, y: candidate.y };
      }
    }
    drag.current = { sx: event.clientX, sy: event.clientY, origin };
  };

  const onScrollPointerDown = (event: ReactPointerEvent) => {
    if (event.button !== 1) {
      return;
    }
    const element = scrollRef.current;
    if (!element) {
      return;
    }
    event.preventDefault();
    pan.current = {
      sx: event.clientX,
      sy: event.clientY,
      left: element.scrollLeft,
      top: element.scrollTop,
    };
    element.setPointerCapture(event.pointerId);
    setPanning(true);
  };

  const onPointerMove = (event: ReactPointerEvent) => {
    const panState = pan.current;
    if (panState) {
      const element = scrollRef.current;
      if (element) {
        element.scrollLeft = panState.left - (event.clientX - panState.sx);
        element.scrollTop = panState.top - (event.clientY - panState.sy);
      }
      return;
    }

    if (marquee) {
      const rect = canvasRef.current?.getBoundingClientRect();
      if (rect) {
        const x1 = (event.clientX - rect.left) / zoom;
        const y1 = (event.clientY - rect.top) / zoom;
        setMarquee((current) => (current ? { ...current, x1, y1 } : current));
      }
      return;
    }

    const current = drag.current;
    if (!current) {
      return;
    }
    const dx = (event.clientX - current.sx) / zoom;
    const dy = (event.clientY - current.sy) / zoom;
    updateNodes((list) =>
      list.map((node) => {
        const origin = current.origin[node.id];
        return origin ? { ...node, x: origin.x + dx, y: origin.y + dy } : node;
      }),
    );
  };

  const onPointerUp = () => {
    if (pan.current) {
      pan.current = null;
      setPanning(false);
    }
    if (marquee) {
      const left = Math.min(marquee.x0, marquee.x1);
      const right = Math.max(marquee.x0, marquee.x1);
      const top = Math.min(marquee.y0, marquee.y1);
      const bottom = Math.max(marquee.y0, marquee.y1);
      const dragged = right - left > 4 || bottom - top > 4;
      if (dragged) {
        const hits = nodes
          .filter(
            (node) =>
              node.x < right &&
              node.x + NODE_W > left &&
              node.y < bottom &&
              node.y + NODE_H > top,
          )
          .map((node) => node.id);
        setSelectedIds((previous) =>
          marquee.additive ? Array.from(new Set([...previous, ...hits])) : hits,
        );
      } else if (!marquee.additive) {
        setSelectedIds([]);
      }
      setMarquee(null);
    }
    drag.current = null;
  };

  const clampZoom = (value: number) =>
    Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, value));

  const zoomAt = (clientX: number, clientY: number, nextZoom: number) => {
    const canvas = canvasRef.current;
    const scroll = scrollRef.current;
    const target = clampZoom(nextZoom);
    const currentZoom = zoomRef.current;
    if (!canvas || !scroll) {
      setZoom(target);
      return;
    }
    const rect = canvas.getBoundingClientRect();
    const worldX = (clientX - rect.left) / currentZoom;
    const worldY = (clientY - rect.top) / currentZoom;

    setZoom(target);
    zoomRef.current = target;
    requestAnimationFrame(() => {
      const currentCanvas = canvasRef.current;
      const container = scrollRef.current;
      if (!currentCanvas || !container) {
        return;
      }
      const nextRect = currentCanvas.getBoundingClientRect();
      container.scrollLeft += nextRect.left + worldX * target - clientX;
      container.scrollTop += nextRect.top + worldY * target - clientY;
    });
  };

  const zoomBy = (factor: number) => {
    const scroll = scrollRef.current;
    if (!scroll) {
      setZoom((value) => clampZoom(value * factor));
      return;
    }
    const rect = scroll.getBoundingClientRect();
    zoomAt(
      rect.left + rect.width / 2,
      rect.top + rect.height / 2,
      zoomRef.current * factor,
    );
  };

  // Wheel zoom must be a native, non-passive listener so preventDefault works.
  useEffect(() => {
    const element = scrollRef.current;
    if (!element) {
      return;
    }
    const handler = (event: WheelEvent) => {
      if (!(event.ctrlKey || event.metaKey)) {
        return;
      }
      event.preventDefault();
      const factor = event.deltaY < 0 ? 1.12 : 1 / 1.12;
      zoomAt(event.clientX, event.clientY, zoomRef.current * factor);
    };
    element.addEventListener("wheel", handler, { passive: false });
    return () => element.removeEventListener("wheel", handler);
  }, [hasDoc]);

  useEffect(() => {
    let active = true;
    const unlisteners: Array<() => void> = [];

    const track = (promise: Promise<() => void>) => {
      void promise.then((fn) => {
        if (active) {
          unlisteners.push(fn);
        } else {
          fn();
        }
      });
    };

    track(
      listen<{ node: string; remaining_ms: number }>("flow-timer", (event) => {
        const { node, remaining_ms } = event.payload;
        setTimers((previous) => {
          if (remaining_ms > 0) {
            if (previous[node] === remaining_ms) {
              return previous;
            }
            return { ...previous, [node]: remaining_ms };
          }
          if (!(node in previous)) {
            return previous;
          }
          const next = { ...previous };
          delete next[node];
          return next;
        });
      }),
    );

    track(
      listen<{ node: string; text: string }>("ocr-text", (event) => {
        const { node, text } = event.payload;
        setOcrTexts((previous) => ({ ...previous, [node]: text }));
        // Persist the last result so it stays on the node after the run.
        setParamsRef.current(node, { last_text: text });
      }),
    );

    track(
      listen<RegionSelection>("region-selected", (event) => {
        const target = regionTargetRef.current;
        regionTargetRef.current = null;
        setRegionTarget(null);
        if (target) {
          setParams(target, {
            x: event.payload.x,
            y: event.payload.y,
            width: event.payload.width,
            height: event.payload.height,
          });
        }
      }),
    );

    return () => {
      active = false;
      unlisteners.forEach((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (!running) {
      setTimers({});
      setOcrTexts({});
    }
  }, [running]);

  useEffect(() => {
    if (!paletteOpen) {
      setPaletteQuery("");
    }
  }, [paletteOpen]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT" ||
          target.isContentEditable)
      ) {
        return;
      }
      if (event.key === "Escape") {
        setPending(null);
        setSelectedIds([]);
        setMarquee(null);
        return;
      }
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "d") {
        event.preventDefault();
        duplicateSelection();
        return;
      }
      if (event.key === "Delete" && selectedIds.length > 0) {
        event.preventDefault();
        removeNodes(selectedIds);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  if (!doc) {
    return (
      <div className="flex flex-1 items-center justify-center text-sm text-slate-500">
        Loading…
      </div>
    );
  }

  return (
    <div className="flex min-w-0 flex-1 flex-col">
      <div className="flex items-center justify-between gap-3 border-b border-surface-800 px-4 py-2.5">
        <div className="flex min-w-0 items-center gap-3">
          <span
            className="h-2.5 w-2.5 shrink-0 rounded-full"
            style={{ background: accent }}
          />
          <div className="min-w-0">
            <div className="truncate text-sm font-semibold text-slate-100">
              {doc.name}
            </div>
            <div className="text-[11px] uppercase tracking-wider text-slate-500">
              {kind === "workflow" ? "Macro" : "Method"} · {nodes.length} nodes
              {saving ? " · saving…" : ""}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {error && (
            <span className="max-w-xs truncate text-xs text-red-400">{error}</span>
          )}
          <div className="relative">
            <button
              type="button"
              onClick={() => setPaletteOpen((value) => !value)}
              className="btn-ghost px-3 py-1.5 text-xs"
            >
              + Add node
            </button>
            {paletteOpen && (
              <div className="absolute right-0 z-30 mt-2 max-h-96 w-72 overflow-y-auto rounded-xl border border-surface-700 bg-surface-900 p-2 shadow-2xl shadow-black/50">
                <input
                  autoFocus
                  value={paletteQuery}
                  onChange={(event) => setPaletteQuery(event.target.value)}
                  placeholder="Search nodes…"
                  className="input mb-2 px-2 py-1.5 text-xs"
                />
                {(() => {
                  const query = paletteQuery.trim().toLowerCase();
                  return CATEGORY_ORDER.map((category) => {
                  const items = specs.filter(
                    (spec) =>
                      spec.category === category &&
                      (!query ||
                        spec.label.toLowerCase().includes(query) ||
                        spec.kind.includes(query) ||
                        spec.description.toLowerCase().includes(query)),
                  );
                  if (items.length === 0) return null;
                  return (
                    <div key={category} className="mb-1">
                      <div className="px-2 py-1 text-[10px] font-semibold uppercase tracking-widest text-slate-500">
                        {CATEGORY_LABELS[category] ?? category}
                      </div>
                      {items.map((spec) => (
                        <button
                          key={spec.kind}
                          type="button"
                          onClick={() => addNode(spec)}
                          className="flex w-full items-start gap-2.5 rounded-lg px-2 py-1.5 text-left hover:bg-surface-800"
                        >
                          <span
                            className="mt-1 h-2 w-2 shrink-0 rounded-full"
                            style={{ background: spec.color }}
                          />
                          <span>
                            <span className="block text-xs font-medium text-slate-200">
                              {spec.label}
                            </span>
                            <span className="block text-[10px] leading-tight text-slate-500">
                              {spec.description}
                            </span>
                          </span>
                        </button>
                      ))}
                    </div>
                  );
                  });
                })()}
              </div>
            )}
          </div>
          {kind === "workflow" && (
            <button
              type="button"
              onClick={onRun}
              className={running ? "btn-danger px-4 py-1.5 text-xs" : "btn-primary px-4 py-1.5 text-xs"}
            >
              {running ? "Stop" : "Run"}
            </button>
          )}
        </div>
      </div>

      <div className="flex min-h-0 flex-1">
        <div className="relative min-h-0 flex-1">
          <div
            ref={scrollRef}
            className={`absolute inset-0 overflow-auto bg-[radial-gradient(circle_at_1px_1px,rgba(148,163,184,0.12)_1px,transparent_0)] [background-size:22px_22px] ${
              panning ? "cursor-grabbing" : ""
            }`}
            onPointerDown={onScrollPointerDown}
            onPointerMove={onPointerMove}
            onPointerUp={onPointerUp}
            onPointerCancel={onPointerUp}
            onPointerLeave={onPointerUp}
            onMouseDown={(event) => {
              if (event.button === 1) {
                event.preventDefault();
              }
            }}
            onAuxClick={(event) => event.preventDefault()}
          >
            <div
              className="relative"
              style={{ width: canvasWidth * zoom, height: canvasHeight * zoom }}
            >
              <div
                ref={canvasRef}
                className="relative origin-top-left"
                style={{
                  width: canvasWidth,
                  height: canvasHeight,
                  transform: `scale(${zoom})`,
                }}
                onPointerDown={onCanvasPointerDown}
              >
            <svg
              className="pointer-events-none absolute left-0 top-0"
              width={canvasWidth}
              height={canvasHeight}
            >
              <defs>
                <marker
                  id="arrow"
                  markerWidth="10"
                  markerHeight="10"
                  refX="8"
                  refY="3"
                  orient="auto"
                >
                  <path d="M0,0 L0,6 L9,3 z" fill="#475569" />
                </marker>
              </defs>
              {connections.map((connection, index) => {
                const from = nodes.find((node) => node.id === connection.from);
                const to = nodes.find((node) => node.id === connection.to);
                if (!from || !to) return null;
                const outputs = outputsFor(from);
                const port = connection.from_port ?? "out";
                const portIndex = Math.max(0, outputs.indexOf(port));
                const x1 = from.x + NODE_W;
                const y1 = from.y + outputY(portIndex, outputs.length);
                const x2 = to.x;
                const y2 = to.y + NODE_H / 2;
                const curve = Math.max(60, Math.abs(x2 - x1) / 2);
                const path = `M ${x1} ${y1} C ${x1 + curve} ${y1}, ${x2 - curve} ${y2}, ${x2} ${y2}`;
                const midX = (x1 + x2) / 2;
                const midY = (y1 + y2) / 2;
                return (
                  <g key={index}>
                    <path
                      d={path}
                      fill="none"
                      stroke="#475569"
                      strokeWidth={2}
                      markerEnd="url(#arrow)"
                    />
                    <g
                      style={{ pointerEvents: "auto", cursor: "pointer" }}
                      onClick={(event) => {
                        event.stopPropagation();
                        removeConnection(index);
                      }}
                    >
                      <circle cx={midX} cy={midY} r={9} fill="#0f121b" stroke="#475569" />
                      <path
                        d={`M ${midX - 3} ${midY - 3} L ${midX + 3} ${midY + 3} M ${midX + 3} ${midY - 3} L ${midX - 3} ${midY + 3}`}
                        stroke="#94a3b8"
                        strokeWidth={1.5}
                      />
                    </g>
                  </g>
                );
              })}
            </svg>

            {marquee && (
              <div
                className="pointer-events-none absolute z-20 rounded border border-gold-500/70 bg-gold-500/10"
                style={{
                  left: Math.min(marquee.x0, marquee.x1),
                  top: Math.min(marquee.y0, marquee.y1),
                  width: Math.abs(marquee.x1 - marquee.x0),
                  height: Math.abs(marquee.y1 - marquee.y0),
                }}
              />
            )}

            {nodes.map((node) => {
              const spec = specByKind[node.kind];
              const outputs = outputsFor(node);
              const color = spec?.color ?? "#64748b";
              const isActive = activeNode === node.id;
              const isSelected = selectedSet.has(node.id);
              const lastOcr = ocrTexts[node.id] ?? String(node.params.last_text ?? "");
              return (
                <div
                  key={node.id}
                  className="absolute"
                  style={{ left: node.x, top: node.y, width: NODE_W, height: NODE_H }}
                >
                  <div
                    onPointerDown={(event) => onNodePointerDown(event, node)}
                    className={`flex h-full cursor-grab select-none flex-col overflow-hidden rounded-xl border bg-surface-850 shadow-lg shadow-black/30 transition-shadow ${
                      isSelected
                        ? "border-gold-500/70"
                        : "border-surface-600 hover:border-surface-500"
                    } ${isActive ? "ring-2 ring-emerald-400/70" : ""}`}
                  >
                    <div
                      className="flex items-center gap-2 border-b border-surface-700 px-3"
                      style={{ height: HEADER }}
                    >
                      <span
                        className="h-2.5 w-2.5 shrink-0 rounded-full"
                        style={{ background: color }}
                      />
                      <span className="min-w-0 flex-1 truncate text-xs font-semibold text-slate-100">
                        {node.name || spec?.label || node.kind}
                      </span>
                      <span className="shrink-0 text-[10px] uppercase tracking-wider text-slate-500">
                        {node.kind === "repeat"
                          ? "loop"
                          : node.kind}
                      </span>
                    </div>
                    <div className="px-3 py-2">
                      {timers[node.id] > 0 ? (
                        <div className="flex items-center gap-1.5 font-mono text-xs font-semibold text-gold-300">
                          <svg viewBox="0 0 20 20" fill="currentColor" className="h-3.5 w-3.5">
                            <path
                              fillRule="evenodd"
                              d="M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16Zm.75-11.5a.75.75 0 0 0-1.5 0v4c0 .2.08.39.22.53l2.5 2.5a.75.75 0 1 0 1.06-1.06L10.75 10.2V6.5Z"
                              clipRule="evenodd"
                            />
                          </svg>
                          {formatTimer(timers[node.id])}
                        </div>
                      ) : lastOcr ? (
                        <div
                          className="truncate font-mono text-[11px] leading-tight text-sky-300"
                          title={lastOcr}
                        >
                          {lastOcr}
                        </div>
                      ) : (
                        <div className="truncate text-[11px] leading-tight text-slate-500">
                          {node.kind === "send_messages"
                            ? `${asStringArray(node.params.messages).length} message(s)`
                            : spec?.description}
                        </div>
                      )}
                    </div>
                  </div>

                  <span
                    data-port
                    title="Input"
                    className="absolute -left-[7px] h-3.5 w-3.5 rounded-full border-2 border-surface-900 bg-slate-500"
                    style={{ top: NODE_H / 2 - 7 }}
                  />

                  {outputs.map((port, index) => (
                    <button
                      key={port}
                      data-port
                      type="button"
                      title={`Connect "${port}"`}
                      onClick={(event) => {
                        event.stopPropagation();
                        startConnect(node.id, port);
                      }}
                      className={`absolute -right-[7px] h-3.5 w-3.5 rounded-full border-2 border-surface-900 transition-transform hover:scale-125 ${
                        pending?.from === node.id && pending.port === port
                          ? "bg-gold-400"
                          : "bg-slate-400"
                      }`}
                      style={{ top: outputY(index, outputs.length) - 7 }}
                    />
                  ))}
                  {outputs.length > 1 &&
                    outputs.map((port, index) => (
                      <span
                        key={`${port}-label`}
                        className="pointer-events-none absolute right-1.5 text-[9px] uppercase tracking-wider text-slate-500"
                        style={{ top: outputY(index, outputs.length) - 6 }}
                      >
                        {port}
                      </span>
                    ))}
                </div>
              );
              })}
              </div>
            </div>
          </div>

          <div className="pointer-events-none absolute bottom-4 right-4 z-30 flex items-center gap-1 rounded-lg border border-surface-700 bg-surface-900/90 p-1 shadow-lg shadow-black/40 backdrop-blur">
            <button
              type="button"
              onClick={() => zoomBy(1 / 1.2)}
              className="btn-icon pointer-events-auto"
              title="Zoom out"
            >
              <svg viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
                <path d="M4 9a1 1 0 0 1 1-1h10a1 1 0 1 1 0 2H5a1 1 0 0 1-1-1Z" />
              </svg>
            </button>
            <button
              type="button"
              onClick={() => setZoom(1)}
              className="pointer-events-auto min-w-14 px-1 text-center font-mono text-xs text-slate-300 transition-colors hover:text-gold-300"
              title="Reset zoom to 100%"
            >
              {Math.round(zoom * 100)}%
            </button>
            <button
              type="button"
              onClick={() => zoomBy(1.2)}
              className="btn-icon pointer-events-auto"
              title="Zoom in"
            >
              <svg viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
                <path d="M10 4a1 1 0 0 1 1 1v4h4a1 1 0 1 1 0 2h-4v4a1 1 0 1 1-2 0v-4H5a1 1 0 1 1 0-2h4V5a1 1 0 0 1 1-1Z" />
              </svg>
            </button>
          </div>
        </div>

        <aside className="flex w-80 shrink-0 flex-col overflow-y-auto border-l border-surface-800 bg-surface-900/40 p-4">
          {pending && (
            <div className="mb-3 rounded-lg border border-gold-500/40 bg-gold-500/10 px-3 py-2 text-xs text-gold-300">
              Click a node to connect it. Click the canvas to cancel.
            </div>
          )}

          {selected ? (
            <NodeConfig
              node={selected}
              spec={specByKind[selected.kind]}
              methods={methods}
              allNodes={nodes}
              connections={connections}
              setParams={(patch) => setParams(selected.id, patch)}
              setName={(name) => setNodeName(selected.id, name)}
              onDuplicate={duplicateSelection}
              onDelete={() => removeNodes([selected.id])}
              regionPicking={regionTarget === selected.id}
              onRegionStart={() => startRegion(selected.id)}
              onRegionCancel={cancelRegion}
            />
          ) : selectedNodes.length > 1 ? (
            <MultiSelection
              count={selectedNodes.length}
              onDuplicate={duplicateSelection}
              onDelete={() => removeNodes(selectedIds)}
            />
          ) : (
            <DocSettings
              kind={kind}
              doc={doc}
              methods={methods}
              onMutate={mutate}
              onDelete={onDelete}
              onReset={onReset}
            />
          )}
        </aside>
      </div>
    </div>
  );
}

function MultiSelection({
  count,
  onDuplicate,
  onDelete,
}: {
  count: number;
  onDuplicate: () => void;
  onDelete: () => void;
}) {
  return (
    <div className="flex flex-col gap-4">
      <div>
        <h2 className="text-xs font-semibold uppercase tracking-[0.18em] text-gold-400">
          {count} nodes selected
        </h2>
        <p className="mt-1 text-xs text-slate-500">
          Drag any selected node to move them together.
        </p>
      </div>

      <div className="flex flex-col gap-2">
        <button
          type="button"
          onClick={onDuplicate}
          className="btn-ghost w-full justify-center"
        >
          Duplicate selection
        </button>
        <button
          type="button"
          onClick={onDelete}
          className="btn-danger w-full justify-center"
        >
          Delete selection
        </button>
      </div>

      <p className="rounded-lg border border-surface-700 bg-surface-900/40 px-3 py-2 text-[11px] leading-relaxed text-slate-500">
        Ctrl/Shift-click to add nodes · drag on empty canvas to rubber-band select
        · Ctrl+D to duplicate · Delete to remove · Esc to clear
      </p>
    </div>
  );
}

function RegionPicker({
  x,
  y,
  width,
  height,
  invert,
  scale,
  threshold,
  color,
  tolerance,
  picking,
  onStart,
  onCancel,
}: {
  x: number;
  y: number;
  width: number;
  height: number;
  invert: string;
  scale: number;
  threshold: boolean;
  color: string;
  tolerance: number;
  picking: boolean;
  onStart: () => void;
  onCancel: () => void;
}) {
  const [preview, setPreview] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [nonce, setNonce] = useState(0);

  useEffect(() => {
    if (width <= 0 || height <= 0) {
      setPreview(null);
      return;
    }
    let cancelled = false;
    setLoading(true);
    const timer = window.setTimeout(async () => {
      try {
        const data = await api.previewRegion(
          x,
          y,
          width,
          height,
          invert,
          scale,
          threshold,
          color,
          tolerance,
        );
        if (!cancelled) {
          setPreview(data);
          setPreviewError(null);
        }
      } catch (cause) {
        if (!cancelled) {
          setPreviewError(String(cause));
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }, 300);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [x, y, width, height, invert, scale, threshold, color, tolerance, nonce]);

  return (
    <div className="rounded-lg border border-surface-700 bg-surface-850/60 p-3">
      <div className="mb-2 flex items-center justify-between">
        <span className="label">Screen region</span>
        <span className="font-mono text-[11px] text-slate-400">
          {x}, {y} · {width}×{height}
        </span>
      </div>

      {picking ? (
        <div className="flex w-full items-center justify-between gap-2 rounded-lg border border-gold-500/60 bg-gold-500/10 px-3 py-2 text-xs font-semibold text-gold-300">
          <span className="flex items-center gap-2">
            <span className="h-2 w-2 animate-pulse rounded-full bg-gold-400" />
            Selecting region…
          </span>
          <button
            type="button"
            onClick={onCancel}
            className="rounded border border-gold-500/50 px-2 py-0.5 text-[11px] hover:bg-gold-500/20"
          >
            Cancel
          </button>
        </div>
      ) : (
        <button type="button" onClick={onStart} className="btn-ghost w-full justify-center">
          <svg viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
            <path d="M4 3a1 1 0 0 0-1 1v3a1 1 0 1 0 2 0V5h2a1 1 0 1 0 0-2H4Zm9 0a1 1 0 1 0 0 2h2v2a1 1 0 1 0 2 0V4a1 1 0 0 0-1-1h-3Zm-9 9a1 1 0 0 0-1 1v3a1 1 0 0 0 1 1h3a1 1 0 1 0 0-2H5v-2a1 1 0 0 0-1-1Zm13 0a1 1 0 0 0-1 1v2h-2a1 1 0 1 0 0 2h3a1 1 0 0 0 1-1v-3a1 1 0 0 0-1-1Z" />
          </svg>
          Pick region
        </button>
      )}

      <div className="mt-3">
        <div className="mb-1 flex items-center justify-between">
          <span className="label">Preview</span>
          <button
            type="button"
            onClick={() => setNonce((value) => value + 1)}
            className="text-[11px] text-slate-400 transition-colors hover:text-gold-300"
          >
            Refresh
          </button>
        </div>
        <div className="relative overflow-hidden rounded-lg border border-surface-700 bg-surface-900">
          {preview ? (
            <img
              src={`data:image/png;base64,${preview}`}
              alt="Region preview"
              className="block max-h-44 w-full object-contain"
            />
          ) : (
            <div className="flex h-24 items-center justify-center px-3 text-center text-[11px] text-slate-500">
              {loading ? "Capturing…" : previewError ?? "No preview yet"}
            </div>
          )}
          {loading && preview && (
            <div className="absolute inset-0 flex items-center justify-center bg-surface-950/50 text-[11px] text-slate-200">
              Refreshing…
            </div>
          )}
        </div>
      </div>

      <p className="mt-2 text-center text-[11px] text-slate-500">
        {picking
          ? "Drag a box over the screen (over the game too). Esc cancels."
          : "Captures this screen region and reads its text."}
      </p>
    </div>
  );
}

function PositionPicker({
  x,
  y,
  onApply,
}: {
  x: number;
  y: number;
  onApply: (patch: Record<string, unknown>) => void;
}) {
  const [picking, setPicking] = useState(false);
  const [live, setLive] = useState<CursorPos | null>(null);
  const onApplyRef = useRef(onApply);
  onApplyRef.current = onApply;

  useEffect(() => {
    if (!picking) {
      return;
    }
    let finished = false;
    void api.startPick();

    const tick = async () => {
      try {
        const result = await api.pollPick();
        if (finished) {
          return;
        }
        setLive({ x: result.x, y: result.y });
        if (result.clicked) {
          finished = true;
          void api.stopPick();
          setPicking(false);
          onApplyRef.current({ mode: "absolute", x: result.x, y: result.y });
        }
      } catch {
        // Ignore transient read failures.
      }
    };

    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        finished = true;
        void api.stopPick();
        setPicking(false);
      }
    };

    const id = window.setInterval(tick, 40);
    window.addEventListener("keydown", onKey, true);
    return () => {
      finished = true;
      window.clearInterval(id);
      window.removeEventListener("keydown", onKey, true);
      void api.stopPick();
    };
  }, [picking]);

  return (
    <div className="rounded-lg border border-surface-700 bg-surface-850/60 p-3">
      <div className="mb-2 flex items-center justify-between">
        <span className="label">Cursor position</span>
        <span className="font-mono text-[11px] text-slate-400">
          {x}, {y}
        </span>
      </div>

      {picking ? (
        <div className="flex w-full items-center justify-center gap-2 rounded-lg border border-gold-500/60 bg-gold-500/10 px-3 py-2 text-xs font-semibold text-gold-300">
          <span className="h-2 w-2 animate-pulse rounded-full bg-gold-400" />
          Click anywhere to capture
        </div>
      ) : (
        <button
          type="button"
          onClick={() => {
            setLive(null);
            setPicking(true);
          }}
          className="btn-ghost w-full justify-center"
        >
          <svg viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
            <path d="M3 2a1 1 0 0 1 1.7-.7l11 9A1 1 0 0 1 15 12h-4.6l2.3 5.1a1 1 0 0 1-1.8.8l-2.4-5.3L5 16.4A1 1 0 0 1 3.4 15.7L3 2Z" />
          </svg>
          Pick position
        </button>
      )}

      <p className="mt-2 text-center font-mono text-xs text-gold-300">
        {picking
          ? live
            ? `X ${live.x}  Y ${live.y}`
            : "Move the cursor…"
          : `Saved: ${x}, ${y}`}
      </p>
      <p className="mt-1 text-center text-[11px] text-slate-500">
        {picking
          ? "Click anywhere on screen to capture it. Press Esc to cancel."
          : "Captures the next click, even outside this window."}
      </p>
    </div>
  );
}

function NodeConfig({
  node,
  spec,
  methods,
  allNodes,
  connections,
  setParams,
  setName,
  onDuplicate,
  onDelete,
  regionPicking,
  onRegionStart,
  onRegionCancel,
}: {
  node: FlowNode;
  spec?: NodeSpec;
  methods: MethodSummary[];
  allNodes: FlowNode[];
  connections: Connection[];
  setParams: (patch: Record<string, unknown>) => void;
  setName: (name: string) => void;
  onDuplicate: () => void;
  onDelete: () => void;
  regionPicking: boolean;
  onRegionStart: () => void;
  onRegionCancel: () => void;
}) {
  const ocrConnected = connections.some(
    (connection) =>
      connection.to === node.id &&
      allNodes.find((candidate) => candidate.id === connection.from)?.kind === "read_text",
  );

  const paramFor = (param: ParamSpec): ParamSpec => {
    if (node.kind === "condition" && param.key === "mode" && !ocrConnected) {
      return { ...param, options: param.options.filter((option) => option !== "ocr") };
    }
    return param;
  };
  return (
    <div className="flex flex-col gap-4">
      <div>
        <div className="mb-1 flex items-center gap-2">
          <span
            className="h-2.5 w-2.5 rounded-full"
            style={{ background: spec?.color ?? "#64748b" }}
          />
          <span className="text-[10px] font-semibold uppercase tracking-widest text-slate-500">
            {spec?.label ?? node.kind}
          </span>
        </div>
        <p className="text-xs text-slate-400">{spec?.description}</p>
      </div>

      <label className="block">
        <span className="label mb-1.5 block">Label</span>
        <input
          value={node.name}
          onChange={(event) => setName(event.target.value)}
          className="input"
        />
      </label>

      {(() => {
        const groups: { name: string; params: ParamSpec[] }[] = [];
        for (const param of spec?.params ?? []) {
          if (
            param.show_if_key &&
            String(node.params[param.show_if_key] ?? "") !== (param.show_if_value ?? "")
          ) {
            continue;
          }
          const name = param.group ?? "";
          let bucket = groups.find((group) => group.name === name);
          if (!bucket) {
            bucket = { name, params: [] };
            groups.push(bucket);
          }
          bucket.params.push(param);
        }
        const showHeaders = groups.some((group) => group.name !== "");
        return groups.map((group) => (
          <div key={group.name || "_default"} className="flex flex-col gap-4">
            {showHeaders && group.name && (
              <div className="border-b border-surface-700 pb-1 text-[10px] font-semibold uppercase tracking-widest text-gold-400/80">
                {group.name}
              </div>
            )}
            {(() => {
              const rows: ReactNode[] = [];
              let inlineRun: ParamSpec[] = [];
              const flushInline = () => {
                if (inlineRun.length === 0) {
                  return;
                }
                const run = inlineRun;
                inlineRun = [];
                rows.push(
                  <div key={run.map((p) => p.key).join("-")} className="grid grid-cols-2 gap-3">
                    {run.map((param) => (
                      <label key={param.key} className="block">
                        <span className="label mb-1.5 block">{param.label}</span>
                        <ParamField
                          param={paramFor(param)}
                          value={node.params[param.key]}
                          methods={methods}
                          allNodes={allNodes}
                          selfId={node.id}
                          onChange={(value) => setParams({ [param.key]: value })}
                        />
                      </label>
                    ))}
                  </div>,
                );
              };
              for (const param of group.params) {
                if (param.inline) {
                  inlineRun.push(param);
                  continue;
                }
                flushInline();
                rows.push(
                  <label key={param.key} className="block">
                    <span className="label mb-1.5 block">{param.label}</span>
                    <ParamField
                      param={paramFor(param)}
                      value={node.params[param.key]}
                      methods={methods}
                      allNodes={allNodes}
                      selfId={node.id}
                      onChange={(value) => setParams({ [param.key]: value })}
                    />
                  </label>,
                );
              }
              flushInline();
              return rows;
            })()}
          </div>
        ));
      })()}

      {node.kind === "move_mouse" && (
        <PositionPicker
          x={Number(node.params.x ?? 0)}
          y={Number(node.params.y ?? 0)}
          onApply={(patch) => setParams(patch)}
        />
      )}

      {node.kind === "focus_warframe" && (
        <WindowPicker
          process={String(node.params.process ?? "Warframe.x64")}
          title={String(node.params.title ?? "")}
          onApply={(patch) => setParams(patch)}
        />
      )}

      {node.kind === "read_text" && (
        <RegionPicker
          x={Number(node.params.x ?? 0)}
          y={Number(node.params.y ?? 0)}
          width={Number(node.params.width ?? 200)}
          height={Number(node.params.height ?? 60)}
          invert={String(node.params.invert ?? "auto")}
          scale={Number(node.params.scale ?? 1)}
          threshold={Boolean(node.params.threshold ?? false)}
          color={String(node.params.color ?? "")}
          tolerance={Number(node.params.tolerance ?? 60)}
          picking={regionPicking}
          onStart={onRegionStart}
          onCancel={onRegionCancel}
        />
      )}

      <div className="mt-2 flex gap-2">
        <button
          type="button"
          onClick={onDuplicate}
          className="btn-ghost flex-1 justify-center"
        >
          Duplicate
        </button>
        <button
          type="button"
          onClick={onDelete}
          className="btn-danger flex-1 justify-center"
        >
          Delete
        </button>
      </div>
    </div>
  );
}

const MOD_NAMES: Record<string, string> = {
  Control: "Ctrl",
  Shift: "Shift",
  Alt: "Alt",
  Meta: "Win",
};

const COMBO_ORDER = ["Ctrl", "Shift", "Alt", "Win"];

function keyNameFromEvent(event: KeyboardEvent): string | null {
  const key = event.key;
  if (key === " ") return "Space";
  if (key === "Enter") return "Return";
  if (key === "Tab") return "Tab";
  if (key === "Backspace") return "Backspace";
  if (key.length === 1) return key;
  if (/^F\d{1,2}$/.test(key)) return key;
  const named: Record<string, string> = {
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
    Insert: "Insert",
    Delete: "Delete",
    Escape: "Escape",
    CapsLock: "CapsLock",
  };
  return named[key] ?? null;
}

function ShortcutField({
  value,
  placeholder,
  onChange,
}: {
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
}) {
  const [recording, setRecording] = useState(false);
  const [preview, setPreview] = useState("");
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;

  useEffect(() => {
    if (!recording) {
      return;
    }
    const mods = new Set<string>();
    const pretty = () => COMBO_ORDER.filter((mod) => mods.has(mod));

    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (event.key === "Escape" && mods.size === 0) {
        setRecording(false);
        return;
      }
      const modifier = MOD_NAMES[event.key];
      if (modifier) {
        mods.add(modifier);
        setPreview(pretty().join("+"));
        return;
      }
      const name = keyNameFromEvent(event);
      if (!name) {
        return;
      }
      onChangeRef.current([...pretty(), name].join("+"));
      setRecording(false);
    };

    const onKeyUp = (event: KeyboardEvent) => {
      const modifier = MOD_NAMES[event.key];
      if (modifier) {
        mods.delete(modifier);
      }
    };

    window.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("keyup", onKeyUp, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("keyup", onKeyUp, true);
    };
  }, [recording]);

  return (
    <div className="flex gap-1.5">
      <input
        value={recording ? (preview ? `${preview}+…` : "Press keys…") : value}
        readOnly={recording}
        placeholder={placeholder}
        onChange={(event) => onChange(event.target.value)}
        className={`input font-mono ${recording ? "border-gold-500/70 text-gold-300" : ""}`}
      />
      <button
        type="button"
        onClick={() => {
          setPreview("");
          setRecording((current) => !current);
        }}
        className={`btn-ghost shrink-0 px-2.5 ${recording ? "border-gold-500 text-gold-300" : ""}`}
        title={recording ? "Stop recording" : "Record shortcut"}
      >
        <svg viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
          <path
            fillRule="evenodd"
            d="M10 3a7 7 0 1 0 0 14 7 7 0 0 0 0-14ZM1 10a9 9 0 1 1 18 0 9 9 0 0 1-18 0Z"
            clipRule="evenodd"
          />
          <circle cx="10" cy="10" r="3" />
        </svg>
      </button>
    </div>
  );
}

function WindowPicker({
  process,
  title,
  onApply,
}: {
  process: string;
  title: string;
  onApply: (patch: Record<string, unknown>) => void;
}) {
  const [countdown, setCountdown] = useState<number | null>(null);
  const onApplyRef = useRef(onApply);
  onApplyRef.current = onApply;

  useEffect(() => {
    if (countdown === null) {
      return;
    }
    if (countdown <= 0) {
      void (async () => {
        try {
          const info = await api.getForegroundWindow();
          onApplyRef.current({ process: info.process, title: info.title });
        } catch {
          // Ignore read failures.
        }
        setCountdown(null);
      })();
      return;
    }
    const id = window.setTimeout(
      () => setCountdown((current) => (current === null ? null : current - 1)),
      1000,
    );
    return () => window.clearTimeout(id);
  }, [countdown]);

  return (
    <div className="rounded-lg border border-surface-700 bg-surface-850/60 p-3">
      <div className="mb-2 flex items-center justify-between">
        <span className="label">Target window</span>
        <span className="min-w-0 truncate font-mono text-[11px] text-slate-400">
          {process || "—"}
          {title ? ` · ${title}` : ""}
        </span>
      </div>

      {countdown === null ? (
        <button
          type="button"
          onClick={() => setCountdown(3)}
          className="btn-ghost w-full justify-center"
        >
          <svg viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
            <path d="M3 4a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1v2H3V4Zm0 4h14v8a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V8Zm2 2v6h10v-6H5Z" />
          </svg>
          Pick window
        </button>
      ) : (
        <div className="flex w-full items-center justify-center gap-2 rounded-lg border border-gold-500/60 bg-gold-500/10 px-3 py-2 text-xs font-semibold text-gold-300">
          <span className="h-2 w-2 animate-pulse rounded-full bg-gold-400" />
          Switch to the target window… {countdown}
        </div>
      )}

      <p className="mt-1 text-center text-[11px] text-slate-500">
        {countdown === null
          ? "Focus this window before running the macro."
          : "Captures whichever window is active when the countdown ends."}
      </p>
    </div>
  );
}

function FileField({
  value,
  placeholder,
  onChange,
}: {
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
}) {
  const browse = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Model", extensions: ["onnx", "rten"] }],
      });
      if (typeof selected === "string") {
        onChange(selected);
      }
    } catch {
      // File dialog unavailable.
    }
  };

  return (
    <div className="flex gap-1.5">
      <input
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        className="input font-mono text-[11px]"
      />
      <button type="button" onClick={browse} className="btn-ghost shrink-0 px-3 py-1.5 text-xs">
        Browse
      </button>
    </div>
  );
}

function ColorField({
  value,
  onChange,
}: {
  value: string;
  onChange: (value: string) => void;
}) {
  const trimmed = value.trim();
  const isHex = /^#?[0-9a-fA-F]{6}$/.test(trimmed);
  const swatch = isHex ? (trimmed.startsWith("#") ? trimmed : `#${trimmed}`) : "#000000";

  return (
    <div className="flex gap-1.5">
      <input
        type="color"
        value={swatch}
        onChange={(event) => onChange(event.target.value)}
        className="h-9 w-12 shrink-0 cursor-pointer rounded-lg border border-surface-600 bg-surface-900"
        title="Pick colour"
      />
      <input
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder="#bd2a2a (blank = off)"
        className="input font-mono"
      />
    </div>
  );
}

function asStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((item): item is string => typeof item === "string");
}

function NodeListField({
  value,
  nodes,
  selfId,
  onChange,
}: {
  value: string[];
  nodes: FlowNode[];
  selfId?: string;
  onChange: (value: string[]) => void;
}) {
  if (nodes.length === 0) {
    return <p className="text-[11px] text-slate-500">No nodes in this macro.</p>;
  }

  return (
    <div className="max-h-48 space-y-0.5 overflow-y-auto rounded-lg border border-surface-700 bg-surface-900/60 p-1.5">
      {nodes.map((node) => (
        <label
          key={node.id}
          className="flex cursor-pointer items-center gap-2 rounded px-1.5 py-1 text-xs text-slate-200 hover:bg-surface-800"
        >
          <input
            type="checkbox"
            checked={value.includes(node.id)}
            onChange={() =>
              onChange(
                value.includes(node.id)
                  ? value.filter((id) => id !== node.id)
                  : [...value, node.id],
              )
            }
            className="h-3.5 w-3.5 shrink-0 accent-gold-500"
          />
          <span className="min-w-0 flex-1 truncate">
            {node.name || node.kind}
            {node.id === selfId ? " (this node)" : ""}
          </span>
          <span className="shrink-0 text-[10px] uppercase tracking-wider text-slate-500">
            {node.kind}
          </span>
        </label>
      ))}
    </div>
  );
}

function ParamField({
  param,
  value,
  methods,
  allNodes,
  selfId,
  onChange,
}: {
  param: ParamSpec;
  value: unknown;
  methods: MethodSummary[];
  allNodes: FlowNode[];
  selfId?: string;
  onChange: (value: unknown) => void;
}) {
  if (param.key === "color") {
    return (
      <ColorField value={String(value ?? "")} onChange={onChange} />
    );
  }

  if (param.key === "detection_model" || param.key === "recognition_model") {
    return (
      <FileField
        value={String(value ?? "")}
        placeholder={param.placeholder}
        onChange={onChange}
      />
    );
  }

  if (param.kind === "shortcut") {
    return (
      <ShortcutField
        value={String(value ?? "")}
        placeholder={param.placeholder}
        onChange={onChange}
      />
    );
  }

  if (param.kind === "list") {
    return (
      <StringList
        value={asStringArray(value)}
        onChange={onChange}
        placeholder={param.placeholder}
      />
    );
  }

  if (param.kind === "nodes") {
    return (
      <NodeListField
        value={asStringArray(value)}
        nodes={allNodes}
        selfId={selfId}
        onChange={onChange}
      />
    );
  }

  if (param.kind === "duration") {
    return (
      <DurationField
        value={Number(value ?? 0)}
        onChange={(milliseconds) => onChange(milliseconds)}
      />
    );
  }

  if (param.key === "method") {
    return (
      <select
        value={String(value ?? "")}
        onChange={(event) => onChange(event.target.value)}
        className="select"
      >
        <option value="">Select method…</option>
        {methods.map((method) => (
          <option key={method.id} value={method.id}>
            {method.name}
          </option>
        ))}
      </select>
    );
  }

  if (param.kind === "checkbox") {
    return (
      <button
        type="button"
        onClick={() => onChange(!Boolean(value))}
        className={`inline-flex h-6 w-11 items-center rounded-full border transition-colors ${
          value ? "border-gold-500 bg-gold-500/80" : "border-surface-600 bg-surface-800"
        }`}
      >
        <span
          className={`inline-block h-4 w-4 rounded-full bg-surface-950 transition-transform ${
            value ? "translate-x-6" : "translate-x-1"
          }`}
        />
      </button>
    );
  }

  if (param.kind === "select") {
    return (
      <select
        value={String(value ?? "")}
        onChange={(event) => onChange(event.target.value)}
        className="select"
      >
        {param.options.map((option) => (
          <option key={option} value={option}>
            {option === "" ? "Choose…" : option}
          </option>
        ))}
      </select>
    );
  }

  if (param.kind === "number") {
    return (
      <input
        type="number"
        value={Number(value ?? 0)}
        min={param.min ?? undefined}
        max={param.max ?? undefined}
        onChange={(event) => onChange(Number(event.target.value))}
        className="input font-mono"
      />
    );
  }

  return (
    <input
      value={String(value ?? "")}
      placeholder={param.placeholder}
      onChange={(event) => onChange(event.target.value)}
      className="input"
    />
  );
}

function DocSettings({
  kind,
  doc,
  methods,
  onMutate,
  onDelete,
  onReset,
}: {
  kind: "workflow" | "method";
  doc: FlowDoc;
  methods: MethodSummary[];
  onMutate: (patch: Partial<Workflow> & Partial<Method>) => void;
  onDelete: () => void;
  onReset: () => void;
}) {
  void methods;

  const [confirming, setConfirming] = useState(false);
  const workflow = kind === "workflow" ? (doc as Workflow) : null;

  return (
    <div className="flex flex-col gap-4">
      <div>
        <h2 className="text-xs font-semibold uppercase tracking-[0.18em] text-gold-400">
          {kind === "workflow" ? "Macro settings" : "Method settings"}
        </h2>
        <p className="mt-1 text-xs text-slate-500">
          Select a node to configure it, or edit the details below.
        </p>
      </div>

      <label className="block">
        <span className="label mb-1.5 block">Name</span>
        <input
          value={doc.name}
          onChange={(event) => onMutate({ name: event.target.value })}
          className="input"
        />
      </label>

      <label className="block">
        <span className="label mb-1.5 block">Description</span>
        <textarea
          value={doc.description}
          onChange={(event) => onMutate({ description: event.target.value })}
          className="input h-20 resize-none"
        />
      </label>

      {workflow && (
        <label className="block">
          <span className="label mb-1.5 block">Accent</span>
          <input
            type="color"
            value={workflow.accent || "#d4a537"}
            onChange={(event) => onMutate({ accent: event.target.value })}
            className="h-9 w-full cursor-pointer rounded-lg border border-surface-600 bg-surface-900"
          />
        </label>
      )}

      <div className="mt-2 border-t border-surface-700 pt-4">
        {confirming ? (
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => {
                setConfirming(false);
                if (doc.builtin) {
                  onReset();
                } else {
                  onDelete();
                }
              }}
              className="btn-danger flex-1 justify-center"
            >
              {doc.builtin
                ? "Confirm reset"
                : `Delete ${kind === "workflow" ? "macro" : "method"}`}
            </button>
            <button
              type="button"
              onClick={() => setConfirming(false)}
              className="btn-ghost justify-center"
            >
              Cancel
            </button>
          </div>
        ) : (
          <button
            type="button"
            onClick={() => setConfirming(true)}
            className={
              doc.builtin ? "btn-ghost w-full justify-center" : "btn-danger w-full justify-center"
            }
          >
            {doc.builtin
              ? "Reset to default"
              : `Delete ${kind === "workflow" ? "macro" : "method"}`}
          </button>
        )}
      </div>
    </div>
  );
}
