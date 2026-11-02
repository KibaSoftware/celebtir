import { describe, expect, it } from "vitest";
import { describeController, delayText } from "./controller";
import { formatDuration, shortAddress } from "./format";
import { actorsFor } from "./permissions";
import type { Controller, Evidence, Fact, ProtocolFacts, TokenFacts } from "./types";

const ev = (field: string): Evidence[] => [
  { chain: "solana", address: "Ev1111111111111111111111111111111111111111", slot: 7, field, explorer_url: "https://solscan.io/account/Ev" },
];
const fact = <T,>(value: T, field = "f"): Fact<T> => ({ value, evidence: ev(field) });

const SQUADS: Controller = {
  kind: "multisig",
  multisig_kind: "squads_v4",
  address: "Msig111111111111111111111111111111111111111",
  threshold: 5,
  voters: 10,
  time_lock_secs: 86_400,
  config_authority: null,
};
const KEY: Controller = { kind: "single_key", key: "Key1111111111111111111111111111111111111111" };

function protocol(over: Partial<ProtocolFacts> = {}): ProtocolFacts {
  return {
    chain: "solana",
    category: "lending",
    last_reviewed: "2026-10-17",
    name: "Demo",
    website: "https://demo.example",
    reviewed_by: "someone",
    sources: [{ kind: "program", address: "Prog111111111111111111111111111111111111111", label: "Lending", url: "https://example.com" }],
    programs: {
      Prog111111111111111111111111111111111111111: {
        role: "core",
        loader: "BPFLoaderUpgradeab1e11111111111111111111111",
        upgrade_control: fact(SQUADS, "upgrade_authority"),
        code_transparency: fact({ kind: "no_public_source" } as const),
      },
    },
    admin: {
      "Acct111111111111111111111111111111111111111.owner": {
        account: "Acct111111111111111111111111111111111111111",
        field: "owner",
        power: "parameters",
        controller: fact(SQUADS),
      },
      "Acct111111111111111111111111111111111111111.pauser": {
        account: "Acct111111111111111111111111111111111111111",
        field: "pauser",
        power: "pause",
        controller: fact(KEY),
      },
    },
    oracles: {},
    dependencies: fact({ kind: "unavailable", reason: "no transactions" } as const),
    tokens: { Mint111111111111111111111111111111111111111: ["collateral"] },
    min_slot: 1,
    max_slot: 2,
    ...over,
  };
}

const USDC: TokenFacts = {
  symbol: "USDC",
  program: "spl_token",
  supply: 1,
  decimals: 6,
  mint_authority: fact(KEY, "mint_authority"),
  freeze_authority: fact(KEY, "freeze_authority"),
  extensions: fact({ default_frozen: false }),
  provenance: fact({ kind: "native" } as const),
};

describe("ui_13 permissions by actor", () => {
  it("ui_13 groups every power under the actor that holds it, with its delay", () => {
    const { actors } = actorsFor(protocol(), {});
    expect(actors.map((a) => a.id)).toEqual(["Key1111111111111111111111111111111111111111", "Msig111111111111111111111111111111111111111"]);
    const squads = actors[1]!;
    expect(squads.label).toBe("Squads 5 of 10 · 24 h delay");
    expect(squads.grants.map((g) => [g.power, g.subjectLabel, g.delaySecs])).toEqual([
      ["upgrade", "Lending", 86_400],
      ["parameters", "owner on Acct…1111", 86_400],
    ]);
    expect(actors[0]!.grants.map((g) => [g.power, g.delaySecs])).toEqual([["pause", 0]]);
  });

  it("ui_13 token authorities add freeze and mint powers for tokens the protocol uses", () => {
    const { actors } = actorsFor(protocol(), { Mint111111111111111111111111111111111111111: USDC });
    const key = actors.find((a) => a.id === KEY.key)!;
    expect(key.grants.map((g) => `${g.power}:${g.subjectLabel}`)).toEqual(["pause:pauser on Acct…1111", "freeze:USDC", "mint:USDC"]);
  });

  it("ui_13 a config authority makes the multisig's delay zero and holds the power to change it", () => {
    const withConfig: Controller = { ...SQUADS, config_authority: KEY } as Controller;
    const p = protocol({ admin: {} });
    p.programs.Prog111111111111111111111111111111111111111!.upgrade_control = fact(withConfig);
    const { actors } = actorsFor(p, {});
    const squads = actors.find((a) => a.id === "Msig111111111111111111111111111111111111111")!;
    expect(squads.grants[0]!.delaySecs).toBe(0);
    const key = actors.find((a) => a.id === KEY.key)!;
    expect(key.grants.map((g) => g.power)).toEqual(["reconfigure"]);
  });

  it("ui_13 renounced powers are listed as held by no one; unreadable ones by an unknown actor", () => {
    const p = protocol({ admin: {} });
    p.programs.Prog111111111111111111111111111111111111111!.upgrade_control = fact({ kind: "immutable" } as const);
    p.admin = {
      "A.x": { account: "Acct111111111111111111111111111111111111111", field: "x", power: "funds", controller: fact({ kind: "unavailable", reason: "decode failed" } as const) },
    };
    const { actors, renounced } = actorsFor(p, {});
    expect(renounced.map((g) => g.power)).toEqual(["upgrade"]);
    expect(actors).toHaveLength(1);
    expect(actors[0]!.kind).toBe("unavailable");
    expect(actors[0]!.grants[0]!.delaySecs).toBeNull();
  });
});

describe("ui_12 shared wording", () => {
  it("ui_12 controllers, delays and addresses read the same everywhere", () => {
    expect(describeController(SQUADS)).toBe("Squads 5 of 10 · 24 h delay");
    expect(describeController({ ...SQUADS, time_lock_secs: 0 } as Controller)).toBe("Squads 5 of 10 · no delay");
    expect(describeController(KEY)).toBe("Single key");
    expect(describeController({ kind: "immutable" })).toBe("Immutable");
    expect(describeController({ kind: "unavailable", reason: "x" })).toBe("Unreadable");
    expect(delayText(null)).toBe("unknown delay");
    expect(delayText(0)).toBe("no delay");
    expect(formatDuration(86_400)).toBe("24 h");
    expect(formatDuration(7 * 86_400)).toBe("7 days");
    expect(formatDuration(1_800)).toBe("1800 s");
    expect(shortAddress("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD")).toBe("KLen…gmjD");
  });
});

describe("ui_19 nested multisigs", () => {
  it("ui_19 a signer that is another multisig is an actor that can sign for the outer one", () => {
    const inner: Controller = {
      kind: "multisig",
      multisig_kind: "squads_v4",
      address: "Inner11111111111111111111111111111111111111",
      threshold: 2,
      voters: 3,
      time_lock_secs: 0,
      config_authority: null,
    };
    const outer = { ...SQUADS, nested_signers: { InnerVault1111111111111111111111111111111111: inner } } as Controller;
    const p = protocol({ admin: {} });
    p.programs.Prog111111111111111111111111111111111111111!.upgrade_control = fact(outer);
    const { actors } = actorsFor(p, {});
    const nested = actors.find((a) => a.id === "Inner11111111111111111111111111111111111111")!;
    expect(nested.grants.map((g) => [g.power, g.subject, g.ref.kind])).toEqual([["sign", SQUADS.kind === "multisig" ? SQUADS.address : "", "multisig"]]);
  });
});
