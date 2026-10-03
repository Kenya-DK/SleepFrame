import { useEffect, useState } from "react";

import * as api from "../api";
import type { Workflow } from "../types";

interface Props {
  open: boolean;
  workflowId: string | null;
  onClose: () => void;
  onImported: (workflow: Workflow) => void;
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    const area = document.createElement("textarea");
    area.value = text;
    document.body.appendChild(area);
    area.select();
    document.execCommand("copy");
    document.body.removeChild(area);
  }
}

export function ShareDialog({ open, workflowId, onClose, onImported }: Props) {
  const [code, setCode] = useState("");
  const [importCode, setImportCode] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!open || !workflowId) {
      return;
    }
    setStatus(null);
    void api
      .exportWorkflow(workflowId)
      .then(setCode)
      .catch((error) => setStatus(String(error)));
  }, [open, workflowId]);

  if (!open) {
    return null;
  }

  const doCopy = async () => {
    if (!code) return;
    await copyText(code);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1500);
  };

  const doImport = async () => {
    if (!importCode.trim()) return;
    setBusy(true);
    try {
      const workflow = await api.importWorkflow(importCode.trim());
      setImportCode("");
      setStatus(`Imported "${workflow.name}"`);
      onImported(workflow);
    } catch (error) {
      setStatus(String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6">
      <div className="card w-full max-w-2xl">
        <header className="flex items-center justify-between border-b border-surface-700 px-5 py-3">
          <h2 className="text-xs font-semibold uppercase tracking-[0.18em] text-gold-400">
            Share macro
          </h2>
          <button type="button" onClick={onClose} className="btn-icon" title="Close">
            <svg viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
              <path d="M6.3 5A1 1 0 0 0 5 6.3L8.6 10 5 13.7A1 1 0 1 0 6.3 15L10 11.4 13.7 15a1 1 0 0 0 1.3-1.3L11.4 10 15 6.3A1 1 0 0 0 13.7 5L10 8.6 6.3 5Z" />
            </svg>
          </button>
        </header>

        <div className="space-y-6 p-5">
          <section>
            <div className="mb-2 flex items-center justify-between">
              <h3 className="label">Export</h3>
              <button
                type="button"
                onClick={doCopy}
                disabled={!code}
                className="btn-ghost px-3 py-1.5 text-xs"
              >
                {copied ? "Copied!" : "Copy share code"}
              </button>
            </div>
            <textarea
              readOnly
              value={code}
              placeholder="Select a macro to export…"
              className="input h-28 resize-none font-mono text-[11px]"
            />
            <p className="mt-1.5 text-[11px] text-slate-500">
              Includes every reusable method the macro references.
            </p>
          </section>

          <section>
            <h3 className="label mb-2">Import</h3>
            <textarea
              value={importCode}
              onChange={(event) => setImportCode(event.target.value)}
              placeholder="Paste a share code or raw JSON here…"
              className="input h-28 resize-none font-mono text-[11px]"
            />
            <div className="mt-2 flex items-center justify-between">
              {status && <span className="text-xs text-slate-400">{status}</span>}
              <button
                type="button"
                onClick={doImport}
                disabled={busy || !importCode.trim()}
                className="btn-primary ml-auto"
              >
                Import macro
              </button>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
