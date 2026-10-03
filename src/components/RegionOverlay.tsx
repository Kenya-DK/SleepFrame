import { useEffect, useState } from "react";

import * as api from "../api";

interface Point {
  x: number;
  y: number;
}

/**
 * Fullscreen transparent overlay for Snipping-Tool style region selection.
 * Rendered in the `region-overlay` window (see main.tsx).
 */
export function RegionOverlay() {
  const [start, setStart] = useState<Point | null>(null);
  const [current, setCurrent] = useState<Point | null>(null);

  useEffect(() => {
    document.documentElement.style.background = "transparent";
    document.body.style.background = "transparent";
    document.body.style.overflow = "hidden";

    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        void api.cancelRegionSelect();
      }
    };

    // The overlay window is shown/hidden, so reset any stale selection when it
    // becomes visible again.
    const reset = () => {
      setStart(null);
      setCurrent(null);
    };

    window.addEventListener("keydown", onKey);
    window.addEventListener("focus", reset);
    document.addEventListener("visibilitychange", reset);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("focus", reset);
      document.removeEventListener("visibilitychange", reset);
    };
  }, []);

  const rect =
    start && current
      ? {
          left: Math.min(start.x, current.x),
          top: Math.min(start.y, current.y),
          width: Math.abs(current.x - start.x),
          height: Math.abs(current.y - start.y),
        }
      : null;

  const finish = () => {
    if (rect && rect.width > 4 && rect.height > 4) {
      void api.submitRegion(rect.left, rect.top, rect.width, rect.height);
    } else {
      setStart(null);
      setCurrent(null);
    }
  };

  return (
    <div
      className="fixed inset-0 cursor-crosshair select-none"
      style={{ background: "rgba(6,7,11,0.35)" }}
      onPointerDown={(event) => {
        if (event.button !== 0) return;
        setStart({ x: event.clientX, y: event.clientY });
        setCurrent({ x: event.clientX, y: event.clientY });
      }}
      onPointerMove={(event) => {
        if (!start) return;
        setCurrent({ x: event.clientX, y: event.clientY });
      }}
      onPointerUp={finish}
    >
      {rect ? (
        <>
          <div
            className="absolute border border-gold-400"
            style={{
              left: rect.left,
              top: rect.top,
              width: rect.width,
              height: rect.height,
              boxShadow: "0 0 0 9999px rgba(6,7,11,0.45)",
              background: "transparent",
            }}
          />
          <div
            className="absolute rounded bg-surface-950/90 px-2 py-0.5 font-mono text-[11px] text-gold-300"
            style={{
              left: rect.left,
              top: Math.max(0, rect.top - 24),
            }}
          >
            {rect.width} × {rect.height}
          </div>
        </>
      ) : (
        <div className="pointer-events-none absolute left-1/2 top-6 -translate-x-1/2 rounded-lg border border-surface-600 bg-surface-950/90 px-4 py-2 text-xs text-slate-200">
          Drag to select a region · Esc to cancel
        </div>
      )}
    </div>
  );
}
