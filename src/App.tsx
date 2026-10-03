import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import * as api from "./api";
import type { NodeSpec, StateDto } from "./types";
import { Badge } from "./components/ui";
import { FlowEditor } from "./components/FlowEditor";
import { NodeReference } from "./components/NodeReference";
import { ShareDialog } from "./components/ShareDialog";
import { Sidebar, type EditorTarget } from "./components/Sidebar";

export default function App() {
  const [state, setState] = useState<StateDto | null>(null);
  const [specs, setSpecs] = useState<NodeSpec[]>([]);
  const [target, setTarget] = useState<EditorTarget | null>(null);
  const [reloadKey, setReloadKey] = useState(0);
  const [activeNode, setActiveNode] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [shareOpen, setShareOpen] = useState(false);
  const [referenceOpen, setReferenceOpen] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setState(await api.getState());
      setError(null);
    } catch (cause) {
      setError(String(cause));
    }
  }, []);

  const refreshRef = useRef(refresh);
  refreshRef.current = refresh;

  useEffect(() => {
    void refresh();
    void api.getNodeSpecs().then(setSpecs).catch(() => undefined);

    let alive = true;
    const unlisten: Array<() => void> = [];

    void (async () => {
      const onStatus = await listen<string>("macro-status", (event) =>
        setState((prev) => (prev ? { ...prev, status: event.payload } : prev)),
      );
      const onStarted = await listen("macro-started", () => void refreshRef.current());
      const onStopped = await listen("macro-stopped", () => {
        setActiveNode(null);
        void refreshRef.current();
      });
      const onNode = await listen<string>("flow-node", (event) =>
        setActiveNode(event.payload),
      );
      const handlers = [onStatus, onStarted, onStopped, onNode];

      if (alive) {
        unlisten.push(...handlers);
      } else {
        handlers.forEach((handler) => handler());
      }
    })();

    return () => {
      alive = false;
      unlisten.forEach((handler) => handler());
    };
  }, [refresh]);

  const run = useCallback(
    async (action: () => Promise<unknown>) => {
      setBusy(true);
      try {
        await action();
        await refresh();
        setError(null);
      } catch (cause) {
        setError(String(cause));
      } finally {
        setBusy(false);
      }
    },
    [refresh],
  );

  const openTarget = useCallback(
    (next: EditorTarget) => {
      setTarget(next);
      setReloadKey((key) => key + 1);
      if (next.kind === "workflow") {
        void api.selectWorkflow(next.id).then(refresh);
      }
    },
    [refresh],
  );

  const onNewWorkflow = useCallback(
    (name: string) => {
      void run(async () => {
        const workflow = await api.createWorkflow(name);
        setTarget({ kind: "workflow", id: workflow.id });
        setReloadKey((key) => key + 1);
      });
    },
    [run],
  );

  const onNewMethod = useCallback(
    (name: string) => {
      void run(async () => {
        const method = await api.createMethod(name);
        setTarget({ kind: "method", id: method.id });
        setReloadKey((key) => key + 1);
      });
    },
    [run],
  );

  const onImported = useCallback(
    (workflow: { id: string }) => {
      setTarget({ kind: "workflow", id: workflow.id });
      setReloadKey((key) => key + 1);
      void refresh();
    },
    [refresh],
  );

  const removeTarget = useCallback(
    (next: EditorTarget) => {
      void run(async () => {
        if (next.kind === "workflow") {
          await api.deleteWorkflow(next.id);
        } else {
          await api.deleteMethod(next.id);
        }
        setTarget((current) =>
          current && current.kind === next.kind && current.id === next.id
            ? null
            : current,
        );
      });
    },
    [run],
  );

  const resetTarget = useCallback(
    (next: EditorTarget) => {
      void run(async () => {
        if (next.kind === "workflow") {
          await api.resetWorkflow(next.id);
        } else {
          await api.resetMethod(next.id);
        }
        setReloadKey((key) => key + 1);
      });
    },
    [run],
  );

  if (!state) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-slate-400">
        Loading SleepFrame…
      </div>
    );
  }

  const running = state.running;
  const selected = state.workflows.find((workflow) => workflow.id === state.selected);
  const hotkeyLabel = `${
    state.hotkey.modifier === "None" ? "" : `${state.hotkey.modifier}+`
  }${state.hotkey.key}`;

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-surface-800 px-5 py-3">
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg border border-gold-500/40 bg-gold-500/10">
            <svg viewBox="0 0 24 24" fill="currentColor" className="h-5 w-5 text-gold-400">
              <path d="M13 2 4.5 13.5H11l-1 8.5 8.5-11.5H12l1-8.5Z" />
            </svg>
          </div>
          <div>
            <h1 className="text-base font-bold tracking-wide text-slate-100">
              SleepFrame
            </h1>
            <p className="text-[11px] uppercase tracking-[0.2em] text-slate-500">
              Warframe macro studio
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Badge tone={running ? "gold" : "slate"}>{running ? "Running" : "Idle"}</Badge>

          <div className="ml-2 flex items-center gap-3">
            <div className="text-right">
              <div className="text-[10px] uppercase tracking-widest text-slate-500">
                {selected?.name ?? "No macro"}
              </div>
              <div className="font-mono text-xs text-gold-300">{state.status}</div>
            </div>
            <div className="text-right">
              <div className="text-[10px] uppercase tracking-widest text-slate-500">
                Hotkey
              </div>
              <div className="font-mono text-xs text-slate-300">{hotkeyLabel}</div>
            </div>
            <button
              type="button"
              disabled={busy}
              onClick={() => run(running ? api.stopFlow : api.startFlow)}
              className={running ? "btn-danger px-6 py-2" : "btn-primary px-6 py-2"}
            >
              {running ? "Stop" : "Start"}
            </button>
          </div>
        </div>
      </header>

      <div className="flex min-h-0 flex-1">
        <Sidebar
          state={state}
          target={target}
          busy={busy}
          onOpen={openTarget}
          onNewWorkflow={onNewWorkflow}
          onNewMethod={onNewMethod}
          onDelete={removeTarget}
          onReset={resetTarget}
          onHotkey={(modifier, key) => run(() => api.setHotkey(modifier, key))}
          onShare={() => {
            if (target?.kind === "workflow") {
              void api.selectWorkflow(target.id);
            }
            setShareOpen(true);
          }}
          onReference={() => setReferenceOpen(true)}
        />

        {target ? (
          <FlowEditor
            key={`${target.kind}:${target.id}:${reloadKey}`}
            kind={target.kind}
            id={target.id}
            reloadKey={reloadKey}
            specs={specs}
            methods={state.methods}
            activeNode={activeNode}
            running={running}
            onSaved={refresh}
            onRun={() => run(running ? api.stopFlow : api.startFlow)}
            onDelete={() => removeTarget(target)}
            onReset={() => resetTarget(target)}
          />
        ) : (
          <div className="flex flex-1 flex-col items-center justify-center gap-4 text-center">
            <div className="max-w-sm">
              <h2 className="text-lg font-semibold text-slate-200">
                Build a macro
              </h2>
              <p className="mt-2 text-sm text-slate-400">
                Pick a macro from the sidebar to open the node editor, or create
                a new one. Wire reusable nodes into a flow, and build reusable
                methods for sequences you want to share across macros.
              </p>
            </div>
            {error && (
              <div className="rounded-xl border border-red-500/40 bg-red-500/10 px-4 py-2 text-sm text-red-300">
                {error}
              </div>
            )}
          </div>
        )}
      </div>

      <ShareDialog
        open={shareOpen}
        workflowId={target?.kind === "workflow" ? target.id : state.selected}
        onClose={() => setShareOpen(false)}
        onImported={onImported}
      />

      {referenceOpen && (
        <NodeReference specs={specs} onClose={() => setReferenceOpen(false)} />
      )}
    </div>
  );
}
