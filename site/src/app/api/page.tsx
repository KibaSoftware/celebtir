import type { Metadata } from "next";
import ref from "@/components/reference/Reference.module.css";
import styles from "@/components/page/Page.module.css";

export const metadata: Metadata = {
  title: "API",
  description: "Celebtir's data as static JSON: summary, protocols, tokens and events, with JSON Schemas, under CC BY 4.0.",
};

const FILES = [
  ["/data/v2/summary.json", "Every protocol's chain and assessment, and for assessed protocols its Stage, six ratings and value class; controller and TVL for all", "summary.schema.json"],
  ["/data/v2/protocols/<id>.json", "One protocol's facts, scores and history", "protocol.schema.json"],
  ["/data/v2/tokens/<mint>.json", "One token's facts, four ratings and supply history", "token.schema.json"],
  ["/data/v2/events.json", "The change log, newest first", "events.schema.json"],
  ["/feeds/all.atom", "The change log as an Atom feed; /feeds/<id>.atom per protocol", "Atom 1.0"],
] as const;

// The JSON documentation (site map: /api). The files themselves are
// published by the static API step.
export default function ApiPage() {
  return (
    <>
      <h1>API</h1>
      <p className={styles.lede}>
        The data behind every page is published as static JSON. There is no key, no rate limit and no server: fetch the files directly, from
        any website.
      </p>
      <div className={styles.scroll}>
        <table className={styles.table}>
          <thead>
            <tr>
              <th scope="col">Path</th>
              <th scope="col">Contents</th>
              <th scope="col">Schema</th>
            </tr>
          </thead>
          <tbody>
            {FILES.map(([p, what, schema]) => (
              <tr key={p}>
                <th scope="row">
                  {p.includes("<") ? <code className={ref.code}>{p}</code> : <a href={p}><code className={ref.code}>{p}</code></a>}
                </th>
                <td>{what}</td>
                <td className={styles.muted}>{schema.endsWith(".json") ? <a href={`/data/v2/schemas/${schema}`}>{schema}</a> : schema}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className={ref.prose}>
        <h2 className={styles.h2}>Rules</h2>
        <ul>
          <li>Schemas are published next to the data, under /data/v2/schemas/, and every file names the schema it follows.</li>
          <li>Version 2 replaced version 1 before launch, so no version 1 files are published. From now on, breaking changes go to a new version path, and the previous version stays for 90 days with a deprecation note in each file.</li>
          <li>Every protocol row carries <code className={ref.code}>chain</code> and <code className={ref.code}>assessment</code>. An <code className={ref.code}>unassessed</code> protocol (HyperEVM under methodology v0.3) has its facts, controller summary and TVL, but <code className={ref.code}>stage</code>, <code className={ref.code}>ratings</code> and <code className={ref.code}>control_class</code> are null.</li>
          <li>Activity <code className={ref.code}>change_7d_bp</code> is the 7-day total against the previous 7 days, null when either week has a missing day or a day below 5% of the series median (listed in <code className={ref.code}>gap_days</code>).</li>
          <li>Files are served with Access-Control-Allow-Origin: *, so a browser on any origin can read them.</li>
          <li>Output is deterministic: keys are sorted and arrays are ordered by stable IDs, so a diff between two days shows only real changes.</li>
          <li>Timestamps appear only in run metadata. Facts carry the slot they were read at.</li>
        </ul>
        <h2 className={styles.h2}>Licence</h2>
        <p>
          Data is published under CC BY 4.0, and every file has a <code className={ref.code}>licence</code> field saying so. Attribute it as
          &ldquo;Data: Celebtir (CC BY 4.0)&rdquo; with a link to this site.
        </p>
        <h2 className={styles.h2}>Example</h2>
        <pre className={ref.pre} tabIndex={0} aria-label="Example: fetch the summary">{`const res = await fetch("/data/v2/summary.json");
const summary = await res.json();`}</pre>
      </div>
    </>
  );
}
