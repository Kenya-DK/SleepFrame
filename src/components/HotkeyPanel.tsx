import { Panel } from "./ui";
import type { HotkeyConfig } from "../types";

const MODIFIERS = ["None", "Ctrl", "Shift", "Alt"];
const KEYS = [
  "F1",
  "F2",
  "F3",
  "F4",
  "F5",
  "F6",
  "F7",
  "F8",
  "F9",
  "F10",
  "F11",
  "F12",
];

interface Props {
  hotkey: HotkeyConfig;
  disabled?: boolean;
  onChange: (modifier: string, key: string) => void;
}

export function HotkeyPanel({ hotkey, disabled, onChange }: Props) {
  return (
    <Panel
      title="Global Hotkey"
      subtitle="Press this combination anywhere to start or stop the selected macro."
    >
      <div className="flex items-end gap-3">
        <div className="flex-1">
          <label className="label mb-2 block">Modifier</label>
          <select
            className="select"
            value={hotkey.modifier}
            disabled={disabled}
            onChange={(event) => onChange(event.target.value, hotkey.key)}
          >
            {MODIFIERS.map((modifier) => (
              <option key={modifier} value={modifier}>
                {modifier}
              </option>
            ))}
          </select>
        </div>
        <span className="pb-2.5 text-lg font-bold text-slate-500">+</span>
        <div className="flex-1">
          <label className="label mb-2 block">Key</label>
          <select
            className="select"
            value={hotkey.key}
            disabled={disabled}
            onChange={(event) => onChange(hotkey.modifier, event.target.value)}
          >
            {KEYS.map((key) => (
              <option key={key} value={key}>
                {key}
              </option>
            ))}
          </select>
        </div>
      </div>
    </Panel>
  );
}
