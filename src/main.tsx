import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";

import App from "./App";
import { RegionOverlay } from "./components/RegionOverlay";
import "./index.css";

function isOverlayWindow(): boolean {
  try {
    return getCurrentWindow().label === "region-overlay";
  } catch {
    return new URLSearchParams(window.location.search).has("overlay");
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  isOverlayWindow() ? <RegionOverlay /> : <App />,
);
