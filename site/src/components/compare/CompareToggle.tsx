"use client";

import { useCompareTray } from "@/lib/compareTray";
import styles from "./Compare.module.css";

/** UI-4: tick a row in any table to add it to the compare tray. */
export function CompareToggle({ id, name }: { id: string; name: string }) {
  const tray = useCompareTray();
  const checked = tray.list.includes(id);
  return (
    <input
      type="checkbox"
      className={styles.toggle}
      aria-label={`Compare ${name}`}
      title={!checked && tray.full ? "The compare tray holds 4 protocols" : `Compare ${name}`}
      checked={checked}
      disabled={!checked && tray.full}
      onChange={() => tray.toggle(id)}
    />
  );
}
