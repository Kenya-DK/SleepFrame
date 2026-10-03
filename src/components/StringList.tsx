import { useState } from "react";

export interface StringListProps {
  /** Controlled list of strings. */
  value: string[];
  /** Called with the next list whenever it changes. */
  onChange: (value: string[]) => void;
  placeholder?: string;
  emptyText?: string;
  disabled?: boolean;
  /** Extra classes for the scroll container. */
  listClassName?: string;
  className?: string;
}

/**
 * Reusable, controlled editor for a list of strings. Used by node parameters
 * of kind `"list"` but has no dependency on the editor.
 */
export function StringList({
  value,
  onChange,
  placeholder = "Type a value and press Enter",
  emptyText = "No items yet.",
  disabled = false,
  listClassName = "max-h-56",
  className = "",
}: StringListProps) {
  const [draft, setDraft] = useState("");

  const add = () => {
    const text = draft.trim();
    if (!text) {
      return;
    }
    onChange([...value, text]);
    setDraft("");
  };

  const update = (index: number, text: string) => {
    onChange(value.map((item, i) => (i === index ? text : item)));
  };

  const remove = (index: number) => {
    onChange(value.filter((_, i) => i !== index));
  };

  return (
    <div className={`flex flex-col ${className}`}>
      <div className={`flex flex-col gap-1.5 overflow-y-auto pr-1 ${listClassName}`}>
        {value.map((item, index) => (
          <div
            key={index}
            className="flex items-center gap-2 rounded-lg border border-surface-700 bg-surface-850/60 px-2 py-1.5"
          >
            <span className="w-4 shrink-0 text-center font-mono text-[10px] text-slate-500">
              {index + 1}
            </span>
            <input
              value={item}
              disabled={disabled}
              onChange={(event) => update(index, event.target.value)}
              placeholder={placeholder}
              className="min-w-0 flex-1 border-none bg-transparent text-xs text-slate-200 outline-none placeholder:text-slate-600 disabled:cursor-not-allowed"
            />
            <button
              type="button"
              disabled={disabled}
              onClick={() => remove(index)}
              className="shrink-0 text-slate-500 transition-colors hover:text-red-400 disabled:cursor-not-allowed"
              title="Remove"
            >
              <svg viewBox="0 0 20 20" fill="currentColor" className="h-3.5 w-3.5">
                <path d="M6.3 5A1 1 0 0 0 5 6.3L8.6 10 5 13.7A1 1 0 1 0 6.3 15L10 11.4 13.7 15a1 1 0 0 0 1.3-1.3L11.4 10 15 6.3A1 1 0 0 0 13.7 5L10 8.6 6.3 5Z" />
              </svg>
            </button>
          </div>
        ))}

        {value.length === 0 && (
          <p className="rounded-lg border border-dashed border-surface-700 px-3 py-4 text-center text-[11px] text-slate-500">
            {emptyText}
          </p>
        )}
      </div>

      <div className="mt-2 flex gap-1.5">
        <input
          value={draft}
          disabled={disabled}
          onChange={(event) => setDraft(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              add();
            }
          }}
          placeholder={placeholder}
          className="input px-2 py-1.5 text-xs"
        />
        <button
          type="button"
          disabled={disabled || draft.trim().length === 0}
          onClick={add}
          className="btn-ghost px-3 py-1.5 text-xs"
        >
          Add
        </button>
      </div>
    </div>
  );
}
