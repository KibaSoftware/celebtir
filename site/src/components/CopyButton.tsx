"use client";

import { useState } from "react";
import styles from "./Evidence.module.css";

/** The older copy path, for browsers or frames that block the Clipboard API. */
function legacyCopy(value: string): boolean {
  if (typeof document.execCommand !== "function") return false;
  const area = document.createElement("textarea");
  area.value = value;
  area.setAttribute("readonly", "");
  area.style.position = "fixed";
  area.style.opacity = "0";
  document.body.appendChild(area);
  area.select();
  let ok = false;
  try {
    ok = document.execCommand("copy");
  } catch {
    ok = false;
  }
  area.remove();
  return ok;
}

/**
 * UI-5: copies a full address or signature and says so for a moment. If the
 * browser blocks copying, it says that instead; the full value stays in the
 * link's title.
 */
export function CopyButton({ value, currentPage = false }: { value: string; currentPage?: boolean }) {
  const [state, setState] = useState<"idle" | "copied" | "blocked">("idle");
  const flash = (s: "copied" | "blocked") => {
    setState(s);
    window.setTimeout(() => setState("idle"), 1500);
  };
  return (
    <button
      type="button"
      className={styles.copy}
      aria-label={`Copy ${value}`}
      title={`Copy ${value}`}
      onClick={async () => {
        const text = currentPage ? window.location.href : value;
        try {
          await navigator.clipboard.writeText(text);
          flash("copied");
        } catch {
          flash(legacyCopy(text) ? "copied" : "blocked");
        }
      }}
    >
      {state === "copied" ? "Copied" : state === "blocked" ? "Copy blocked" : "Copy"}
    </button>
  );
}
