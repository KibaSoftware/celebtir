import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Hip3List } from "./Hip3List";
import { loadSnapshot } from "@/lib/data";
import { buildGraph } from "@/lib/graph";
import { actorsFor } from "@/lib/permissions";
import { accountUrl } from "@/lib/solscan";
import type { Controller, Evidence, Hip3DexFacts, Hip3Snapshot, Snapshot } from "@/lib/types";

afterEach(cleanup);

const DEPLOYER = "0x88806a71D74ad0a510b350545C9aE490912F0888";
const UPDATER = "0x94757f8DCb4bf73b850195660e959d1105CfedD5";
const PROXY = "0x393D0B87Ed38fc779FD9611144aE649BA6082109";
const SAFE = "0x18A82c968b992D28D4D812920eB7b4305306f8F1";
const TIMELOCK = "0x52908400098527886E0F7030069857D2E4169EE7";

const ev = (address: string, field: string, chain = "hyper_core"): Evidence => ({
  chain,
  address,
  slot: 49_145_767,
  field,
  explorer_url: accountUrl(address, chain),
});
const fact = <T,>(value: T) => ({ value, evidence: [ev(DEPLOYER, "info.perpDexs[xyz]")] });

const dex: Hip3DexFacts = {
  full_name: "XYZ",
  deployer: fact(DEPLOYER),
  oracle_updater: fact(null),
  sub_deployers: fact({ setOracle: [UPDATER] }),
  fee_recipient: fact(null),
  oi_caps: fact({ "xyz:AAPL": "200000000.0", "xyz:GOLD": "750000000.0" }),
  funding_multipliers: fact({}),
  stake: fact({
    kind: "read" as const,
    delegated: "501610.91703892",
    undelegated: "0.0",
    pending_withdrawal: "0.0",
    required: "500000",
    stake_below_required: false,
    unstaking: false,
  }),
};

describe("hui_2 explorer links per chain", () => {
  it("hui_2 HyperEVM goes to HyperEVMScan, HyperCore to the Hyperliquid explorer, Solana to Solscan", () => {
    expect(accountUrl(PROXY)).toBe(`https://hyperevmscan.io/address/${PROXY}`);
    expect(accountUrl(PROXY, "hyper_evm")).toBe(`https://hyperevmscan.io/address/${PROXY}`);
    expect(accountUrl(DEPLOYER, "hyper_core")).toBe(`https://app.hyperliquid.xyz/explorer/address/${DEPLOYER}`);
    expect(accountUrl("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD")).toBe(
      "https://solscan.io/account/KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD",
    );
  });
});

describe("hui_3 HIP-3 DEX page", () => {
  const listed: Hip3Snapshot = { kind: "listed", dexs: { xyz: dex } };

  it("hui_3 shows deployer, oracle control, stake and caps, not rated and without a Stage", () => {
    render(<Hip3List hip3={listed} />);
    const section = screen.getByRole("region", { name: /XYZ/ });
    const s = within(section);
    expect(s.getByText(/Unscored control facts/)).toBeTruthy();
    expect(section.querySelector('[data-rating]')).toBeNull();
    expect(section.textContent).not.toMatch(/Stage [0-2]/);
    // F11: oracle delegation and stake are stated as read, without a shortfall claim.
    expect(section.textContent).toMatch(/No oracle updater set; setOracle delegated to:|No oracle updater set and no setOracle delegation recorded/);
    expect(s.getAllByText("setOracle").length).toBeGreaterThan(0);
    expect(s.getByText(/501.61K HYPE staked by the deployer address/)).toBeTruthy();
    expect(section.textContent).not.toMatch(/below the requirement|shortfall/);
    expect(s.getByText("xyz:GOLD")).toBeTruthy();
    const deployer = s.getAllByRole("link").find((a) => a.getAttribute("title") === DEPLOYER)!;
    expect(deployer.getAttribute("href")).toBe(`https://app.hyperliquid.xyz/explorer/address/${DEPLOYER}`);
  });

  it("hui_3 missing or failed HIP-3 data says so", () => {
    render(<Hip3List hip3={undefined} />);
    expect(screen.getByText(/not in this snapshot/)).toBeTruthy();
    cleanup();
    render(<Hip3List hip3={{ kind: "unavailable", reason: "perpDexs: HTTP status 500" }} />);
    expect(screen.getByText(/unavailable for this snapshot/)).toBeTruthy();
  });
});

describe("hui_4 Safe behind a timelock in the control graph", () => {
  const safe: Controller = {
    kind: "multisig",
    multisig_kind: "safe",
    address: SAFE,
    threshold: 4,
    voters: 8,
    time_lock_secs: 0,
    config_authority: null,
    signers: [],
  };
  const timelock: Controller = { kind: "timelock", address: TIMELOCK, min_delay_secs: 172_800, proposer: safe, admin: null };

  function snapshot(): Snapshot {
    const s = structuredClone(loadSnapshot());
    const base = Object.keys(s.scores.protocols)[0]!;
    s.facts.protocols["evm-test"] = {
      ...s.facts.protocols[base]!,
      chain: "hyper_evm",
      name: "EVM Test",
      sources: [],
      programs: {
        [PROXY]: {
          role: "core",
          loader: null,
          upgrade_control: { value: timelock, evidence: [ev(PROXY, "eip1967.admin", "hyper_evm")] },
          code_transparency: s.facts.protocols[base]!.programs[Object.keys(s.facts.protocols[base]!.programs)[0]!]!.code_transparency,
        },
      },
      admin: {},
      tokens: {},
    };
    s.scores.protocols["evm-test"] = { ...s.scores.protocols[base]!, programs: {}, admin: {}, tokens: {} };
    return s;
  }

  it("hui_4 the proposer Safe is a parent of the timelock, with the delay on the edge", () => {
    const s = snapshot();
    const g = buildGraph(s, "evm-test");
    const tl = g.nodes.find((n) => n.id === `node-${TIMELOCK}`)!;
    const sf = g.nodes.find((n) => n.id === `node-${SAFE}`)!;
    expect(tl.kind).toBe("timelock");
    expect(sf.kind).toBe("multisig");
    expect(tl.facts.find((f) => f.label === "Minimum delay")!.value).toBe("2 days delay");
    const upgrade = g.edges.find((e) => e.source === tl.id)!;
    expect(upgrade.target).toBe(`node-program-${PROXY}`);
    expect(upgrade.label).toBe("can upgrade · 2 days delay");
    const propose = g.edges.find((e) => e.source === sf.id && e.target === tl.id)!;
    expect(propose.label).toMatch(/^can propose through/);
    expect(sf.column).toBeLessThan(tl.column);
  });

  it("hui_4 an outside admin is shown and removes the delay", () => {
    const s = snapshot();
    const key: Controller = { kind: "single_key", key: DEPLOYER };
    s.facts.protocols["evm-test"]!.programs[PROXY]!.upgrade_control.value = { ...timelock, admin: key };
    const { actors } = actorsFor(s.facts.protocols["evm-test"]!, s.facts.tokens);
    const admin = actors.find((a) => a.id === DEPLOYER)!;
    expect(admin.grants.map((g) => g.power)).toEqual(["reconfigure"]);
    const g = buildGraph(s, "evm-test");
    const upgrade = g.edges.find((e) => e.source === `node-${TIMELOCK}`)!;
    expect(upgrade.label).toBe("can upgrade · no delay");
  });
});
