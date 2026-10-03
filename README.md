# SleepFrame

A desktop automation companion for Warframe, built as a **Tauri (Rust)** app
with a **React + Tailwind CSS** frontend.

Macros are built visually in an **n8n-style node editor**: you wire small,
reusable nodes together into a flow, define reusable **methods** and call them
from any macro. Macros can be shared as a copy-paste code.

## Documentation

A how-to wiki lives in [`docs/`](./docs/index.html) and is ready to publish with
GitHub Pages (Settings → Pages → Branch `main`, folder `/docs`).

## Features

- **Node-based macro editor** — drag nodes on a canvas, connect their ports, and
  configure each one in the side panel. Connections are drawn as curved links.
- **Reusable methods** — named sub-flows that macros can call directly or run on
  their own background timer.
- **Rich node library** — start/end, key down/up/press, **hold key** (key down +
  wait + key up), **shortcut (key combos)**, type text, send messages (with an
  editable string list), click, double click, move mouse, focus Warframe, wait
  (with countdown + notification), condition (true/false routing), repeat
  (optionally every X time), call method, method timer, notification.
- **Read text (OCR)** — capture a screen region and read its text with a
  **bundled, pure-Rust OCR engine**. No Tesseract or other install needed, and
  the detection/recognition models are embedded in the binary. Picking a region
  opens a **Snipping-Tool style fullscreen overlay** — just drag a box over the
  screen (over the game too); Esc cancels. The recognised text is shown on the
  node and can be typed automatically.
- **Shortcut capture** — key fields have a record button: press the combination
  you want (e.g. `Ctrl+Shift+A`, `Alt+F4`, `5`) and it is captured for you.
- **Live execution feedback** — the node currently running lights up on the
  canvas; status, tray icon and notifications track the run.
- **Share macros** — export a macro (and the methods it uses) as a share code,
  then paste it into another install to import it.
- **Global hotkey** — start/stop the selected macro from anywhere (default `F3`).
- **System tray** — the app keeps running in the tray; closing the window hides
  it instead of quitting.
- **Built-in presets** — `Trading` and `The Index` ship as editable examples.

## Requirements

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- WebView2 (preinstalled on Windows 10/11)
- Windows

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

Installers and binaries are written to `src-tauri/target/release/bundle`.

## Using the editor

1. Pick a macro in the sidebar (or create one), then click **Run** / **Start**.
2. **+ Add node** opens the palette; nodes are grouped into Triggers, Inputs,
   Timing, Flow, Methods and Output.
3. Click an output port on the right of a node, then click another node to
   connect them. Click the small **×** on a link to remove it. An output can
   have several links — they all run together as parallel branches.
4. Select nodes by clicking them; **Ctrl/Shift-click** adds to the selection and
   dragging on empty canvas **rubber-band selects**. Drag any selected node to
   move the whole group. **Ctrl+D** duplicates the selection (keeping internal
   links), **Delete** removes it and **Esc** clears it.
   Zoom with the **−/%/+** control (bottom-right) or **Ctrl/⌘ + scroll**; click
   the percentage to reset to 100%. **Hold the middle mouse button and drag** to
   pan the canvas.
5. Select a node to edit its parameters; select nothing to edit the macro's
   name, description and accent colour. The **Send messages** node holds its own
   editable list of strings and cycles through them.
6. For key fields, click the **record** button and press the combination you
   want (e.g. `Ctrl+Space`, `Shift+F1`); you can also type it manually. Use the
   **Shortcut** node to fire a whole combination at once.
7. Create **methods** in the Methods section and reference them with the
   *Call method* or *Method timer* nodes. Built-ins can be duplicated or reset.
8. **Share / Import macro** exports a share code (including referenced methods)
   or imports one from a code or raw JSON.

## Project layout

```
src/                     React + Tailwind frontend
  components/
    FlowEditor.tsx       n8n-style canvas, palette and config panel
    StringList.tsx       reusable controlled list-of-strings editor
    DurationField.tsx    reusable duration input (number + ms / s / min)
    Sidebar.tsx          macro/method lists, hotkey, tools
    ShareDialog.tsx      export / import share codes
    ui.tsx               shared Panel / Badge / Switch primitives
  api.ts                 Typed wrappers around the Tauri commands
  types.ts               Shared DTOs
src-tauri/               Rust backend
  src/lib.rs             App setup, tray icon, window handling
  src/workflow.rs        Node/workflow model, node registry, built-ins
  src/executor.rs        Node graph execution engine
  src/runtime.rs         Tray/status/notifications, hotkey, start/stop
  src/state.rs           App state and on-disk storage / share bundles
  src/commands.rs        Tauri commands exposed to the frontend
  src/input.rs           Keyboard/mouse simulation (enigo)
  src/win.rs             Windows process/window interop
  src/log_processor.rs   EE.log tailer
  src/ocr.rs             Screen-region OCR (bundled pure-Rust engine)
  models/                Embedded OCR models (.onnx, no install needed)
```

## Notes

- Macros and methods are stored in `%APPDATA%\com.sleepframe.app\settings`.
  Built-ins are generated in code and can be reset; your edits shadow them.
- SleepFrame does **not** require administrator. It forces the game window to
  the foreground (by attaching its input thread) so simulated input reaches it.
- If your game or launcher runs **elevated**, Windows blocks synthetic input
  from a normal app. In that case run the game without admin, or optionally
  launch SleepFrame as administrator.
- SleepFrame only simulates the inputs you configure; use it responsibly and in
  line with Warframe's terms of service.

## Disclaimer

By accessing SleepFrame, you agree that:

Any use of SleepFrame is at your own risk.
You acknowledge that SleepFrame is not liable for any consequences arising from
your use of SleepFrame.
