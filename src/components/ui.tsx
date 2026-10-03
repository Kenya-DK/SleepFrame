import type { ReactNode } from "react";

interface PanelProps {
  title: string;
  subtitle?: string;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
}

export function Panel({
  title,
  subtitle,
  actions,
  children,
  className = "",
}: PanelProps) {
  return (
    <section className={`card ${className}`}>
      <header className="flex items-start justify-between gap-4 border-b border-surface-700 px-5 py-3">
        <div>
          <h2 className="text-xs font-semibold uppercase tracking-[0.18em] text-gold-400">
            {title}
          </h2>
          {subtitle && (
            <p className="mt-1 max-w-xl text-xs leading-relaxed text-slate-400">
              {subtitle}
            </p>
          )}
        </div>
        {actions && <div className="flex shrink-0 items-center gap-2">{actions}</div>}
      </header>
      <div className="p-5">{children}</div>
    </section>
  );
}

interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
  disabled?: boolean;
}

export function Switch({ checked, onChange, label, disabled }: SwitchProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border transition-colors disabled:cursor-not-allowed disabled:opacity-40 ${
        checked
          ? "border-gold-500 bg-gold-500/80"
          : "border-surface-600 bg-surface-800"
      }`}
      title={label}
    >
      <span
        className={`inline-block h-4 w-4 transform rounded-full bg-surface-950 shadow transition-transform ${
          checked ? "translate-x-6" : "translate-x-1"
        }`}
      />
    </button>
  );
}

export function Badge({ children, tone = "slate" }: { children: ReactNode; tone?: "slate" | "green" | "gold" | "red" }) {
  const tones: Record<string, string> = {
    slate: "border-surface-600 bg-surface-800 text-slate-300",
    green: "border-emerald-500/40 bg-emerald-500/10 text-emerald-300",
    gold: "border-gold-500/50 bg-gold-500/10 text-gold-300",
    red: "border-red-500/40 bg-red-500/10 text-red-300",
  };
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-[11px] font-semibold uppercase tracking-wider ${tones[tone]}`}
    >
      {children}
    </span>
  );
}
