import { LimitedItems } from "@/components/LimitedItems";
import { Address, EvidenceList } from "@/components/Evidence";
import { formatAmount, formatUsd } from "@/lib/format";
import { type Hip3DexFacts, type Hip3Snapshot } from "@/lib/types";
import styles from "./Page.module.css";


function HyperCore({ address }: { address: string }) {
  return <Address address={address} href={`https://app.hyperliquid.xyz/explorer/address/${address}`} />;
}

/** Hyperliquid's HIP-3 rules, which state the stake requirement. */
export const HIP3_RULES = "https://hyperliquid.gitbook.io/hyperliquid-docs/hyperliquid-improvement-proposals-hips/hip-3-builder-deployed-perpetuals";

/**
 * F11: the HYPE the deployer address itself has staked, as read. The HIP-3
 * rules ask for a stake but do not say which address must hold it, so a
 * lower figure here is not presented as a shortfall.
 */
function stakeText(d: Hip3DexFacts): string {
  const s = d.stake.value;
  if (s.kind === "unavailable") return `Unavailable (${s.reason})`;
  const parts = [`${formatAmount(Number(s.delegated))} HYPE staked by the deployer address`];
  if (s.unstaking) parts.push("unstaking in progress");
  return parts.join("; ");
}

/** HUI-3: one section per HIP-3 DEX; facts only, no ratings or Stage in v0.1. */
export function Hip3Dex({ name, dex, sameDeployer = [] }: { name: string; dex: Hip3DexFacts; sameDeployer?: string[] }) {
  const caps = Object.entries(dex.oi_caps.value);
  const subs = Object.entries(dex.sub_deployers.value);
  const setOracle = dex.sub_deployers.value.setOracle ?? [];
  return (
    <section id={name} aria-labelledby={`hip3-${name}`} className={`${styles.card} ${styles.hip3}`} data-hip3={name}>
      <h2 id={`hip3-${name}`}>
        {dex.full_name} <span className={styles.muted}>({name})</span>
      </h2>
      <p className={styles.muted}>Unscored control facts. Celebtir has no adopted HIP-3 rating methodology; protocol factors and Stages do not apply to this directory.</p>
      {sameDeployer.length ? (
        <p className={styles.muted}>
          A distinct DEX from the same deployer as {sameDeployer.map((n, i) => <span key={n}>{i ? ", " : ""}<a href={`#${n}`}>{n}</a></span>)}: each is listed separately by <code>perpDexs</code>.
        </p>
      ) : null}
      <p>{caps.length} recorded markets · {subs.length} delegated action types. {dex.oracle_updater.value ? "An oracle updater is recorded." : setOracle.length ? "Oracle updates are delegated." : "The deployer address sets prices; no updater delegation is recorded."}</p>
      <details><summary>Deployer, permissions and stake</summary>
      <dl className={styles.facts}>
        <dt>Deployer</dt>
        <dd>
          <HyperCore address={dex.deployer.value} />
        </dd>
        <dt>Oracle updater</dt>
        <dd>
          {dex.oracle_updater.value ? (
            <HyperCore address={dex.oracle_updater.value} />
          ) : setOracle.length ? (
            <>
              No oracle updater set; <code>setOracle</code> delegated to:{" "}
              {setOracle.map((a, i) => (
                <span key={a}>
                  {i ? ", " : ""}
                  <HyperCore address={a} />
                </span>
              ))}
            </>
          ) : (
            <>No oracle updater set and no <code>setOracle</code> delegation recorded: the deployer address sets prices</>
          )}
        </dd>
        <dt>Delegated actions</dt>
        <dd>
          {subs.length ? (
            <ul>
              {subs.map(([action, addrs]) => (
                <li key={action}>
                  <code>{action}</code>:{" "}
                  {addrs.map((a, i) => (
                    <span key={a}>
                      {i ? ", " : ""}
                      <HyperCore address={a} />
                    </span>
                  ))}
                </li>
              ))}
            </ul>
          ) : (
            "None"
          )}
        </dd>
        <dt>Fee recipient</dt>
        <dd>{dex.fee_recipient.value ? <HyperCore address={dex.fee_recipient.value} /> : "Not set"}</dd>
        <dt>HYPE staked by the deployer address</dt>
        <dd>
          {stakeText(dex)}.{" "}
          <span className={styles.muted}>
            The <a href={HIP3_RULES}>HIP-3 rules</a> require 500K HYPE staked, maintained for at least 183 days after deployment and expected to decrease over time; they do not say which address must hold it.
          </span>
        </dd>
      </dl></details>
      <details>
        <summary>
          Open-interest caps ({caps.length} {caps.length === 1 ? "market" : "markets"})
        </summary>
        {caps.length ? (
          <table className={styles.table}>
            <caption className="sr-only">Open-interest caps of {name}</caption>
            <thead>
              <tr>
                <th scope="col">Market</th>
                <th scope="col">Cap (USD)</th>
              </tr>
            </thead>
            <LimitedItems table colSpan={2} label="markets">
              {caps.map(([asset, cap]) => (
                <tr key={asset}>
                  <td>{asset}</td>
                  <td>{formatUsd(Number(cap))}</td>
                </tr>
              ))}
            </LimitedItems>
          </table>
        ) : (
          <p>No markets listed.</p>
        )}
      </details>
      <details>
        <summary>Evidence</summary>
        <EvidenceList evidence={[...dex.deployer.evidence, ...dex.stake.evidence]} />
      </details>
    </section>
  );
}

export function Hip3List({ hip3 }: { hip3: Hip3Snapshot | undefined }) {
  if (!hip3) return <p data-empty="">HIP-3 DEXs are not in this snapshot yet.</p>;
  if (hip3.kind === "unavailable") return <p data-empty="">HIP-3 data is unavailable for this snapshot: {hip3.reason}.</p>;
  const names = Object.keys(hip3.dexs).sort();
  if (!names.length) return <p data-empty="">The info API listed no HIP-3 DEXs.</p>;
  return (
    <>
      <nav aria-label="HIP-3 DEXs">
        <ul className={styles.inlineList}>
          {names.map((n) => (
            <li key={n}>
              <a href={`#${n}`}>{hip3.dexs[n]!.full_name}</a>
            </li>
          ))}
        </ul>
      </nav>
      {names.map((n) => (
        <Hip3Dex
          key={n}
          name={n}
          dex={hip3.dexs[n]!}
          sameDeployer={names.filter(m => m !== n && hip3.dexs[m]!.deployer.value.toLowerCase() === hip3.dexs[n]!.deployer.value.toLowerCase())}
        />
      ))}
    </>
  );
}
