import { useState } from "react";

import type { StateDto } from "../types";
import { Badge } from "./ui";
import { HotkeyPanel } from "./HotkeyPanel";

export interface EditorTarget {
  kind: "workflow" | "method";
  id: string;
}

interface Props {
  state: StateDto;
  target: EditorTarget | null;
  busy: boolean;
  onOpen: (target: EditorTarget) => void;
  onNewWorkflow: (name: string) => void;
  onNewMethod: (name: string) => void;
  onDelete: (target: EditorTarget) => void;
  onReset: (target: EditorTarget) => void;
  onHotkey: (modifier: string, key: string) => void;
  onShare: () => void;
  onReference: () => void;
}

function CreateInline({
  placeholder,
  onCreate,
}: {
  placeholder: string;
  onCreate: (name: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const [value, setValue] = useState("");

  const submit = () => {
    onCreate(value.trim() || placeholder);
    setValue("");
    setOpen(false);
  };

  if (!open) {
    return (
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="flex w-full items-center justify-center gap-1.5 rounded-lg border border-dashed border-surface-600 py-2 text-xs font-semibold text-slate-400 transition-colors hover:border-gold-500/60 hover:text-gold-400"
      >
        <svg viewBox="0 0 20 20" fill="currentColor" className="h-3.5 w-3.5">
          <path d="M10 3a1 1 0 0 1 1 1v5h5a1 1 0 1 1 0 2h-5v5a1 1 0 1 1-2 0v-5H4a1 1 0 1 1 0-2h5V4a1 1 0 0 1 1-1Z" />
        </svg>
        New
      </button>
    );
  }

  return (
    <div className="flex gap-1.5">
      <input
        autoFocus
        value={value}
        onChange={(event) => setValue(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter") submit();
          if (event.key === "Escape") setOpen(false);
        }}
        placeholder={placeholder}
        className="input px-2 py-1.5 text-xs"
      />
      <button type="button" onClick={submit} className="btn-primary px-3 py-1.5 text-xs">
        Add
      </button>
    </div>
  );
}

function TrashIcon() {
  return (
    <svg viewBox="0 0 20 20" fill="currentColor" className="h-3.5 w-3.5">
      <path
        fillRule="evenodd"
        d="M8.75 1a.75.75 0 0 0-.75.75V3H3.5a.75.75 0 0 0 0 1.5h.61l.9 11.2A2.75 2.75 0 0 0 7.75 18h4.5a2.75 2.75 0 0 0 2.74-2.53l.9-11.2h.61a.75.75 0 0 0 0-1.5H12V1.75a.75.75 0 0 0-.75-.75h-2.5ZM6.62 4.5l.88 11a1.25 1.25 0 0 0 1.25 1.15h4.5a1.25 1.25 0 0 0 1.25-1.15l.88-11H6.62Z"
        clipRule="evenodd"
      />
    </svg>
  );
}

function ResetIcon() {
  return (
    <svg viewBox="0 0 20 20" fill="currentColor" className="h-3.5 w-3.5">
      <path
        fillRule="evenodd"
        d="M4.5 8A5.5 5.5 0 1 0 6 4.6V2.75a.75.75 0 0 0-1.5 0v3.5c0 .41.34.75.75.75h3.5a.75.75 0 0 0 0-1.5H6.9A4 4 0 1 1 5.5 12.6a.75.75 0 1 0-1.36.64A5.5 5.5 0 0 0 4.5 8Z"
        clipRule="evenodd"
      />
    </svg>
  );
}

function ListItem({
  active,
  accent,
  name,
  meta,
  selected,
  builtin,
  busy,
  onOpen,
  onRemove,
}: {
  active: boolean;
  accent: string;
  name: string;
  meta: string;
  selected: boolean;
  builtin: boolean;
  busy: boolean;
  onOpen: () => void;
  onRemove: () => void;
}) {
  return (
    <div
      className={`group flex items-center gap-1 rounded-lg border pr-1 transition-colors ${
        active
          ? "border-gold-500/60 bg-gold-500/10"
          : "border-surface-700 bg-surface-850/40 hover:border-surface-500 hover:bg-surface-800/70"
      }`}
    >
      <button
        type="button"
        disabled={busy}
        onClick={onOpen}
        className="flex min-w-0 flex-1 items-center gap-2.5 px-3 py-2 text-left disabled:opacity-50"
      >
        <span
          className="h-2.5 w-2.5 shrink-0 rounded-full"
          style={{ background: accent }}
        />
        <span className="min-w-0 flex-1">
          <span className="block truncate text-sm font-medium text-slate-200">
            {name}
          </span>
          <span className="block text-[11px] text-slate-500">{meta}</span>
        </span>
        {selected && <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-emerald-400" />}
      </button>

      <button
        type="button"
        disabled={busy}
        onClick={onRemove}
        title={builtin ? "Reset to default" : "Delete"}
        className={`shrink-0 rounded-md p-1.5 opacity-0 transition-all focus:opacity-100 group-hover:opacity-100 disabled:cursor-not-allowed ${
          builtin
            ? "text-slate-500 hover:text-gold-400"
            : "text-slate-500 hover:text-red-400"
        }`}
      >
        {builtin ? <ResetIcon /> : <TrashIcon />}
      </button>
    </div>
  );
}

export function Sidebar({
  state,
  target,
  busy,
  onOpen,
  onNewWorkflow,
  onNewMethod,
  onDelete,
  onReset,
  onHotkey,
  onShare,
  onReference,
}: Props) {
  return (
    <aside className="flex w-72 shrink-0 flex-col gap-4 overflow-y-auto border-r border-surface-800 bg-surface-900/50 p-4">
      <section>
        <div className="mb-2 flex items-center justify-between">
          <h2 className="text-xs font-semibold uppercase tracking-[0.18em] text-slate-500">
            Macros
          </h2>
          <span className="font-mono text-[11px] text-slate-600">
            {state.workflows.length}
          </span>
        </div>
        <div className="flex flex-col gap-1.5">
          {state.workflows.map((workflow) => (
            <ListItem
              key={workflow.id}
              active={target?.kind === "workflow" && target.id === workflow.id}
              accent={workflow.accent || "#d4a537"}
              name={workflow.name}
              meta={`${workflow.node_count} nodes${workflow.builtin ? " · built-in" : ""}`}
              selected={state.selected === workflow.id}
              builtin={workflow.builtin}
              busy={busy}
              onOpen={() => onOpen({ kind: "workflow", id: workflow.id })}
              onRemove={() =>
                workflow.builtin
                  ? onReset({ kind: "workflow", id: workflow.id })
                  : onDelete({ kind: "workflow", id: workflow.id })
              }
            />
          ))}
        </div>
        <div className="mt-2">
          <CreateInline placeholder="New macro" onCreate={onNewWorkflow} />
        </div>
      </section>

      <section>
        <div className="mb-2 flex items-center justify-between">
          <h2 className="text-xs font-semibold uppercase tracking-[0.18em] text-slate-500">
            Methods
          </h2>
          <Badge tone="slate">reusable</Badge>
        </div>
        <div className="flex flex-col gap-1.5">
          {state.methods.map((method) => (
            <ListItem
              key={method.id}
              active={target?.kind === "method" && target.id === method.id}
              accent="#f472b6"
              name={method.name}
              meta={`${method.node_count} nodes${method.builtin ? " · built-in" : ""}`}
              selected={false}
              builtin={method.builtin}
              busy={busy}
              onOpen={() => onOpen({ kind: "method", id: method.id })}
              onRemove={() =>
                method.builtin
                  ? onReset({ kind: "method", id: method.id })
                  : onDelete({ kind: "method", id: method.id })
              }
            />
          ))}
        </div>
        <div className="mt-2">
          <CreateInline placeholder="New method" onCreate={onNewMethod} />
        </div>
      </section>

      <section className="mt-1 flex flex-col gap-3">
        <HotkeyPanel hotkey={state.hotkey} disabled={busy} onChange={onHotkey} />

        <button
          type="button"
          onClick={onShare}
          className="btn-ghost w-full justify-center py-2 text-xs"
        >
          <svg viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
            <path d="M10 2a4 4 0 0 0-3.9 4.9A5 5 0 0 0 10 16a5 5 0 0 0 3.9-9.1A4 4 0 0 0 10 2Zm1 5V5.4a4 4 0 0 0 .9.1 2 2 0 1 1-1.9-2 4 4 0 0 0-1 1.6A4 4 0 0 0 11 7ZM9 13v-2H7.5a4 4 0 0 0 1 1.7L9 13Z" />
          </svg>
          Share / Import macro
        </button>

        <button
          type="button"
          onClick={onReference}
          className="btn-ghost w-full justify-center py-2 text-xs"
        >
          <svg viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
            <path
              fillRule="evenodd"
              d="M10 2a8 8 0 1 0 0 16 8 8 0 0 0 0-16ZM9 8a1 1 0 1 1 2 0c0 .44-.28.74-.63 1.06l-.12.11c-.35.32-.75.68-1.02 1.19A.75.75 0 0 0 10 11.5a.75.75 0 0 0 .71-.53c.13-.24.32-.44.65-.75l.12-.1C11.9 9.7 12.5 9.1 12.5 8a2.5 2.5 0 0 0-5 0 .75.75 0 0 0 1.5 0ZM10 15a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z"
              clipRule="evenodd"
            />
          </svg>
          Node reference
        </button>
      </section>
    </aside>
  );
}
