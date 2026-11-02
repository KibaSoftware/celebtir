import type { Metadata } from "next";
import { ArchivedList } from "@/components/page/ArchivedList";
import styles from "@/components/page/Page.module.css";
import { loadHistory } from "@/lib/data";
import type { ArchivedFile } from "@/lib/types";

export const metadata: Metadata = {
  title: "Archived",
  description: "Protocols that shut down or left Celebtir's coverage, with their last known ratings and the reason.",
};

// ADR 017: listed in registry/archived.yaml, shown as last rated.
export default function ArchivedPage() {
  const archived = loadHistory<ArchivedFile>("archived")?.protocols ?? [];
  return (
    <>
      <h1>Archived</h1>
      <p className={styles.lede}>
        Protocols that shut down or left coverage. Each shows its ratings on the last day it was rated, which may no longer
        match the chain.
      </p>
      <ArchivedList archived={archived} />
    </>
  );
}
