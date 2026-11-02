"use client";

import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { useCompareTray, TRAY_MAX } from "@/lib/compareTray";
import styles from "./Compare.module.css";

/**
 * UI-4: the compare tray, on every page while it holds anything. It lives in
 * localStorage, so it persists across pages until cleared.
 */
export function CompareTray({ names }: { names: Record<string, string> }) {
  const tray = useCompareTray();
  const ids = tray.list.filter((id) => names[id]);
  if (!ids.length) return null;
  return (
    <aside className={styles.tray} aria-label="Compare tray" data-shortcut="tray">
      <span className={styles.trayTitle}>
        Compare ({ids.length} of {TRAY_MAX})
      </span>
      <ul className={styles.chips}>
        {ids.map((id) => (
          <li key={id} className={styles.chip}>
            {names[id]}
            <button type="button" className={styles.remove} aria-label={`Remove ${names[id]} from the compare tray`} onClick={() => tray.remove(id)}>
              ×
            </button>
          </li>
        ))}
      </ul>
      {ids.length >= 2 ? (
        <Link className={styles.go} href={`/compare/?p=${ids.join(",")}`}>
          Compare {ids.length}
        </Link>
      ) : (
        <span className={styles.hint}>Add one more to compare</span>
      )}
      <button type="button" className={styles.clear} onClick={tray.clear}>
        Clear
      </button>
    </aside>
  );
}
