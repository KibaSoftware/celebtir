import { LimitedItems } from "@/components/LimitedItems";
import { authorityWording } from "@/lib/presentation";
import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { Address } from "@/components/Evidence";
import type { SharedController } from "@/lib/graph";
import styles from "./Graph.module.css";

/**
 * F2: a program-derived address is never a key. Resolved ones are listed
 * under the program that signs for them; unresolved ones apart; only
 * controllers Celebtir could not classify are "Unrecognized".
 */
const GROUPS: { title: string; note: string; match: (c: SharedController) => boolean }[] = [
  { title: "Keys, multisigs and timelocks", note: "Accounts that sign directly.", match: c => !["program_controlled", "pda", "unrecognized", "unavailable", "cannot_act"].includes(c.kind) },
  { title: "Program-derived addresses, by controlling program", note: "No private key exists; the listed program signs, and whoever can upgrade it controls the power.", match: c => c.kind === "program_controlled" },
  { title: "Program-derived addresses, controlling program not resolved", note: "No private key exists; which program signs for the address is not yet verified, so these powers are rated Unknown.", match: c => c.kind === "pda" },
  { title: "Unrecognized or unreadable", note: "Controllers Celebtir could not classify.", match: c => ["unrecognized", "unavailable", "cannot_act"].includes(c.kind) },
];

function Item({ c }: { c: SharedController }) {
  return (
    <article className={styles.sharedItem} aria-labelledby={`shared-${c.id}`}>
      <h3 id={`shared-${c.id}`} className={styles.sharedTitle}>
        {authorityWording(c.label)} {c.address ? <Address address={c.address} /> : null}
      </h3>
      <p>{c.protocols.length} affected protocols</p>
      <LimitedItems className={styles.edges} label="affected protocols">
        {c.protocols.map((p) => (
          <li key={p.id}>
            <Link href={`/protocols/${p.id}/#node-${p.actorId}`}>{p.name}</Link>: {p.descriptions.join(", ")}
          </li>
        ))}
      </LimitedItems>
    </article>
  );
}

/** UI-22: each key or multisig that controls more than one protocol, once. */
export function SharedControllers({ shared }: { shared: SharedController[] }) {
  if (!shared.length) {
    return (
      <p className={styles.muted} data-empty="">
        No key or multisig controls more than one protocol in this snapshot.
      </p>
    );
  }
  return (
    <>
      {GROUPS.map((g) => {
        const items = shared.filter(g.match);
        if (!items.length) return null;
        const byProgram = g.title.startsWith("Program-derived addresses, by")
          ? [...new Set(items.map((i) => i.program ?? ""))].sort().map((program) => [program, items.filter((i) => (i.program ?? "") === program)] as const)
          : [["", items] as const];
        return (
          <section key={g.title} aria-label={g.title}>
            <h2>
              {g.title} <span className={styles.muted}>({items.length})</span>
            </h2>
            <p className={styles.muted}>{g.note}</p>
            {byProgram.map(([program, list]) => (
              <div key={program || "all"}>
                {program ? (
                  <p>
                    Program <Address address={program} />
                  </p>
                ) : null}
                <LimitedItems className={styles.shared} label="shared controllers">
                  {list.map((c) => (
                    <li key={c.id} id={`controller-${c.id}`}><Item c={c} /></li>
                  ))}
                </LimitedItems>
              </div>
            ))}
          </section>
        );
      })}
    </>
  );
}
