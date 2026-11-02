import styles from "./DataState.module.css";

/**
 * UI-7: the four "no ordinary value" states, with one visual and one wording
 * everywhere. Ratings show Unknown and Not applicable through RatingBadge,
 * which uses the same labels.
 */
export type DataStateKind = "unknown" | "unrated" | "not_applicable" | "unavailable";

export const DATA_STATES: Record<DataStateKind, { label: string; explain: string }> = {
  unknown: {
    label: "Unknown",
    explain: "Unknown: available evidence is insufficient to determine this factor.",
  },
  unrated: {
    label: "Unrated",
    explain: "Unrated: Celebtir does not have enough supported evidence or an applicable published methodology to assign a rating in this domain.",
  },
  not_applicable: {
    label: "Not applicable",
    explain: "Not applicable: this criterion does not apply to this assessed entity under the selected methodology.",
  },
  unavailable: {
    label: "Data unavailable",
    explain: "Data unavailable: the relevant source could not be retrieved or decoded for this observation.",
  },
};

export function DataState({ kind, detail }: { kind: DataStateKind; detail?: string }) {
  const s = DATA_STATES[kind];
  return (
    <span className={styles.state} data-state={kind} title={detail ? `${s.explain} ${detail}` : s.explain}>
      {s.label}
    </span>
  );
}
