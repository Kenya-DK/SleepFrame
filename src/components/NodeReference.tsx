import { useMemo, useState } from "react";

import type { NodeSpec, ParamSpec } from "../types";

const CATEGORY_LABELS: Record<string, string> = {
  trigger: "Triggers",
  input: "Inputs",
  timing: "Timing",
  flow: "Flow",
  method: "Methods",
  output: "Output",
};

const CATEGORY_ORDER = ["trigger", "input", "timing", "flow", "method", "output"];

function paramSummary(param: ParamSpec): string {
  switch (param.kind) {
    case "select":
      return param.options.filter(Boolean).join(" / ");
    case "checkbox":
      return "on / off";
    case "number":
      return "number";
    case "duration":
      return "time";
    case "shortcut":
      return "key / shortcut";
    case "nodes":
      return "node picker";
    case "list":
      return "list";
    default:
      return "text";
  }
}

export function NodeReference({
  specs,
  onClose,
}: {
  specs: NodeSpec[];
  onClose: () => void;
}) {
  const [query, setQuery] = useState("");

  const groups = useMemo(() => {
    const term = query.trim().toLowerCase();
    const matches = (spec: NodeSpec) =>
      !term ||
      spec.label.toLowerCase().includes(term) ||
      spec.kind.includes(term) ||
      spec.description.toLowerCase().includes(term) ||
      spec.help.toLowerCase().includes(term);

    return CATEGORY_ORDER.map((category) => ({
      category,
      items: specs.filter(
        (spec) => spec.category === category && matches(spec),
      ),
    })).filter((group) => group.items.length > 0);
  }, [specs, query]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6"
      onClick={onClose}
    >
      <div
        className="card flex max-h-[85vh] w-full max-w-3xl flex-col"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="flex items-center justify-between gap-4 border-b border-surface-700 px-5 py-3">
          <h2 className="text-xs font-semibold uppercase tracking-[0.18em] text-gold-400">
            Node reference
          </h2>
          <button type="button" onClick={onClose} className="btn-icon" title="Close">
            <svg viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
              <path d="M6.3 5A1 1 0 0 0 5 6.3L8.6 10 5 13.7A1 1 0 1 0 6.3 15L10 11.4 13.7 15a1 1 0 0 0 1.3-1.3L11.4 10 15 6.3A1 1 0 0 0 13.7 5L10 8.6 6.3 5Z" />
            </svg>
          </button>
        </header>

        <div className="border-b border-surface-700 px-5 py-3">
          <input
            autoFocus
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search what a node does…"
            className="input"
          />
        </div>

        <div className="flex-1 overflow-y-auto px-5 py-4">
          {groups.length === 0 && (
            <p className="py-8 text-center text-sm text-slate-500">
              No nodes match “{query}”.
            </p>
          )}

          {groups.map((group) => (
            <section key={group.category} className="mb-6">
              <h3 className="mb-2 text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-500">
                {CATEGORY_LABELS[group.category] ?? group.category}
              </h3>
              <div className="flex flex-col gap-2">
                {group.items.map((spec) => (
                  <article
                    key={spec.kind}
                    className="rounded-xl border border-surface-700 bg-surface-850/60 p-4"
                  >
                    <div className="flex items-center gap-2.5">
                      <span
                        className="h-2.5 w-2.5 shrink-0 rounded-full"
                        style={{ background: spec.color }}
                      />
                      <h4 className="text-sm font-semibold text-slate-100">
                        {spec.label}
                      </h4>
                      <code className="rounded bg-surface-900 px-1.5 py-0.5 text-[10px] text-slate-500">
                        {spec.kind}
                      </code>
                    </div>

                    <p className="mt-2 text-xs text-slate-400">{spec.description}</p>
                    {spec.help && (
                      <p className="mt-1.5 text-xs leading-relaxed text-slate-500">
                        {spec.help}
                      </p>
                    )}

                    <div className="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1 text-[11px] text-slate-500">
                      <span>
                        <span className="text-slate-400">Outputs:</span>{" "}
                        {spec.outputs.length ? spec.outputs.join(", ") : "—"}
                      </span>
                    </div>

                    {spec.params.length > 0 && (
                      <ul className="mt-2 space-y-1">
                        {spec.params.map((param) => (
                          <li
                            key={param.key}
                            className="flex flex-wrap items-baseline gap-x-2 text-[11px]"
                          >
                            <span className="font-medium text-slate-300">
                              {param.label}
                            </span>
                            <span className="text-slate-600">
                              {paramSummary(param)}
                            </span>
                            {param.show_if_key && (
                              <span className="text-slate-600">
                                (only for {param.show_if_key} = {param.show_if_value})
                              </span>
                            )}
                          </li>
                        ))}
                      </ul>
                    )}
                  </article>
                ))}
              </div>
            </section>
          ))}
        </div>
      </div>
    </div>
  );
}
