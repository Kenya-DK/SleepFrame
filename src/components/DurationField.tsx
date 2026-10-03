import { useEffect, useState } from "react";

type Unit = "ms" | "s" | "min";

const UNITS: Record<Unit, number> = {
  ms: 1,
  s: 1000,
  min: 60_000,
};

function bestUnit(milliseconds: number): Unit {
  const value = Math.abs(milliseconds);
  if (value > 0 && value % 60_000 === 0) return "min";
  if (value > 0 && value % 1000 === 0) return "s";
  return "ms";
}

function format(amount: number): string {
  if (!Number.isFinite(amount)) return "0";
  return String(Math.round(amount * 1000) / 1000);
}

export interface DurationFieldProps {
  /** Duration in milliseconds. */
  value: number;
  onChange: (milliseconds: number) => void;
  disabled?: boolean;
}

/**
 * Reusable duration input: a number plus a ms / s / min unit, so timings don't
 * have to be typed in raw milliseconds. The stored value is always milliseconds.
 */
export function DurationField({ value, onChange, disabled }: DurationFieldProps) {
  const milliseconds = Number(value) || 0;
  const [unit, setUnit] = useState<Unit>(() => bestUnit(milliseconds));
  const [text, setText] = useState(() =>
    format(milliseconds / UNITS[bestUnit(milliseconds)]),
  );
  const [focused, setFocused] = useState(false);

  useEffect(() => {
    if (focused) {
      return;
    }
    const next = Number(value) || 0;
    const nextUnit = bestUnit(next);
    setUnit(nextUnit);
    setText(format(next / UNITS[nextUnit]));
  }, [value, focused]);

  const commit = (raw: string) => {
    setText(raw);
    const parsed = Number(raw);
    if (!Number.isNaN(parsed) && parsed >= 0) {
      onChange(Math.round(parsed * UNITS[unit]));
    }
  };

  const changeUnit = (nextUnit: Unit) => {
    setUnit(nextUnit);
    setText(format(milliseconds / UNITS[nextUnit]));
  };

  return (
    <div className="flex gap-1.5">
      <input
        value={text}
        disabled={disabled}
        inputMode="decimal"
        onFocus={() => setFocused(true)}
        onBlur={() => {
          setFocused(false);
          setText(format(milliseconds / UNITS[unit]));
        }}
        onChange={(event) => commit(event.target.value)}
        className="input font-mono"
      />
      <select
        value={unit}
        disabled={disabled}
        onChange={(event) => changeUnit(event.target.value as Unit)}
        className="select w-24 shrink-0"
      >
        <option value="ms">ms</option>
        <option value="s">s</option>
        <option value="min">min</option>
      </select>
    </div>
  );
}
