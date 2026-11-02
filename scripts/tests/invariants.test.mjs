import test from "node:test";
import assert from "node:assert/strict";
import { run, controllerClass } from "../../celebtir-invariants.mjs";

const program = "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy";
const mint = "So11111111111111111111111111111111111111112";
const review = { source: "https://example.org/review", reviewed_on: "2026-10-26", note: "No admin powers" };
function fixture() {
  const ratings = { admin_powers: "low", oracle_dependency: "not_applicable" };
  const row = { id: "p", chain: "solana", assessment: "assessed", stage: "stage_0", ratings, control_class: "immutable" };
  const p = { id: "p", chain: "solana", assessment: "assessed", score: {}, facts: {
    chain: "solana", category: "liquid-staking", tokens: { [mint]: [] }, admin: {}, oracles: {}, admin_review: review, oracle_review: review,
    programs: { [program]: { role: "core", upgrade_control: { value: { kind: "immutable" } }, code_transparency: { value: { kind: "public_source_unverified", repo: "https://github.com/solana-program/stake-pool" } } } },
  }, program_history: { [program]: { events: [{ kind: "upgrade", date: "2026-10-19", signature: "tx", slot: 42 }] } } };
  return {
    "/data/v2/summary.json": { api_version: "v2", methodology_version: "0.3", snapshot_date: "2026-10-26", assessed_chains: ["solana"], protocols: [row] },
    "/data/v2/protocols/p.json": p,
    [`/data/v2/tokens/${mint}.json`]: { facts: { mint_authority: { value: { kind: "immutable" } }, freeze_authority: { value: { kind: "immutable" } } } },
    "/data/v2/events.json": { events: [{ protocol: "p", source: "on_chain", kind: "program_upgraded", signature: "tx", slot: 42, date: "2026-10-26", occurred_on: "2026-10-19", detected_within_days: 7, evidence: [{ address: program, slot: 42, explorer_url: "https://solscan.io/tx/tx" }] }] },
  };
}
const check = (data, options) => run("http://fixture", async url => {
  const v = data[new URL(url).pathname];
  return new Response(typeof v === "string" ? v : JSON.stringify(v), { status: v === undefined ? 404 : 200 });
}, options);
const row = d => d["/data/v2/summary.json"].protocols[0];
const proto = d => d["/data/v2/protocols/p.json"];

test("sourced reviewed-none, exact transaction evidence and v2 coverage pass", async () => assert.deepEqual(await check(fixture()), []));

test("application updates preserve historical methodology but still enforce API assessment", async () => {
  for (const version of ["0.1", "0.2"]) {
    const d = fixture(); d["/data/v2/summary.json"].methodology_version=version;
    delete proto(d).facts.admin_review;
    const original = JSON.stringify(d);
    assert.ok((await check(d)).length > 0);
    assert.deepEqual(await check(d,{applicationUpdate:true}), []);
    assert.equal(JSON.stringify(d), original);
    const missing = structuredClone(d); delete missing[`/data/v2/tokens/${mint}.json`];
    assert.ok((await check(missing,{applicationUpdate:true})).some(f=>f.includes("F2 token missing")));
    row(d).chain="hyper_evm";
    assert.ok((await check(d,{applicationUpdate:true})).some(f=>f.includes("F5")));
  }
  const current=fixture(); delete proto(current).facts.admin_review;
  assert.ok((await check(current,{applicationUpdate:true})).some(f=>f.includes("F3")));
  current["/data/v2/summary.json"].methodology_version="9.9";
  assert.ok((await check(current,{applicationUpdate:true})).some(f=>f.includes("methodology")));
});

test("Sourcify matches require HyperEVM implementation evidence and a recognized match type", async () => {
  const d = fixture();
  row(d).chain = proto(d).chain = proto(d).facts.chain = "hyper_evm";
  d["/data/v2/summary.json"].assessed_chains = ["hyper_evm"];
  const address = "0x" + "a".repeat(40);
  const source = proto(d).facts.programs[program].code_transparency;
  source.value = {kind:"verified",repo:`https://repo.sourcify.dev/999/${address}`,commit:"sourcify:exact_match"};
  source.evidence = [{chain:"hyper_evm",address,field:"sourcify verified source"}];
  assert.deepEqual(await check(d), []);
  for (const mutate of [
    s => { s.evidence[0].address = "0x" + "b".repeat(40); },
    s => { s.value.commit = "sourcify:unverified"; },
    s => { s.evidence[0].chain = "solana"; },
  ]) {
    const bad = structuredClone(d);
    mutate(proto(bad).facts.programs[program].code_transparency);
    assert.ok((await check(bad)).some(f => f.includes("F1")));
  }
});
test("Etherscan fallback accepts only the collector's address-bound HyperEVM attestation", async () => {
  const d = fixture();
  row(d).chain = proto(d).chain = proto(d).facts.chain = "hyper_evm";
  d["/data/v2/summary.json"].assessed_chains = ["hyper_evm"];
  const implementation = "0x68e37dE8d93d3496ae143F2E900490f6280C57cD";
  const source = proto(d).facts.programs[program].code_transparency;
  // The source target may be the proxy implementation rather than the program key.
  source.value = {kind:"verified",repo:`https://hyperevmscan.io/address/${implementation}#code`,commit:"etherscan:etherscan"};
  source.evidence = [{chain:"hyper_evm",address:implementation.toLowerCase(),field:"etherscan verified source",slot:123}];
  const original = JSON.stringify(d);
  assert.deepEqual(await check(d), []);
  assert.equal(JSON.stringify(d), original);
  for (const mutate of [
    s => { s.evidence = []; },
    s => { s.evidence[0].address = "0x" + "b".repeat(40); },
    s => { s.evidence[0].chain = "solana"; },
    s => { s.evidence[0].field = "sourcify verified source"; },
    s => { s.value.kind = "public_source_unverified"; },
    s => { s.value.commit = "etherscan:unverified"; },
    s => { s.value.repo = s.value.repo.replace("hyperevmscan.io", "hyperevmscan.io.example.org"); },
    s => { s.value.repo += "?unbound=1"; },
    s => { s.value.repo = s.value.repo.replace("#code", ""); },
  ]) {
    const bad = structuredClone(d);
    mutate(proto(bad).facts.programs[program].code_transparency);
    assert.ok((await check(bad)).some(f => f.includes("F1")));
  }
  proto(d).facts.chain = "solana";
  assert.ok((await check(d)).some(f => f.includes("F1")));
});
for (const [name, mutate, marker] of [
  ["missing expected token", d => delete d[`/data/v2/tokens/${mint}.json`], "F2 token missing"],
  ["off-curve key", d => d[`/data/v2/tokens/${mint}.json`].facts.mint_authority.value = {kind:"single_key",key:"6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"}, "F2 PDA as key"],
  ["unresolved PDA rated High", d => { d[`/data/v2/tokens/${mint}.json`].facts.mint_authority.value = {kind:"pda"}; }, "F2 unresolved PDA"],
  ["missing admin review", d => delete proto(d).facts.admin_review, "F3 missing data"],
  ["missing oracle review", d => delete proto(d).facts.oracle_review, "F3 missing oracle"],
  ["org source", d => proto(d).facts.programs[program].code_transparency.value.repo = "https://github.com/solana-program", "F1 repository"],
  ["short commit", d => proto(d).facts.programs[program].code_transparency.value.commit = "abcdef", "F1 commit"],
  ["missing assessment", d => delete row(d).assessment, "F5 assessment"],
  ["wrong chain", d => row(d).chain = "hyper_evm", "F5 chain"],
  ["same-day unrelated event", d => d["/data/v2/events.json"].events[0].signature = "other", "F6 missing event"],
  ["wrong program event", d => d["/data/v2/events.json"].events[0].evidence[0].address = "other", "F6 missing event"],
  ["duplicate transaction event", d => d["/data/v2/events.json"].events.push(d["/data/v2/events.json"].events[0]), "F6 missing event"],
  ["dishonest detection latency", d => d["/data/v2/events.json"].events[0].detected_within_days = 1, "F6 missing event"],
  ["short delay called no delay", d => { proto(d).facts.programs[program].upgrade_control.value = {kind:"multisig",time_lock_secs:43200}; row(d).control_class="multisig_without_delay"; }, "F7 bucket"],
  ["draft reviewer", d => proto(d).facts.reviewed_by = "loop-draft", "H2 draft"],
]) test(name, async () => { const d=fixture(); mutate(d); assert.ok((await check(d)).some(f=>f.includes(marker))); });

test("shared program source cannot differ by protocol", async () => {
  const d=fixture(); d["/data/v2/summary.json"].protocols.push({...row(d),id:"q"});
  d["/data/v2/protocols/q.json"]=structuredClone(proto(d));
  d["/data/v2/protocols/q.json"].facts.programs[program].code_transparency.value.repo="https://github.com/other/repo";
  assert.ok((await check(d)).some(f=>f.includes("F1 program-keyed")));
});
test("calendar omissions and near-zero days invalidate weekly changes", async () => {
  for(const omission of [true,false]) {
    const d=fixture(); const days=Array.from({length:14},(_,i)=>({date:`2026-10-${String(i+1).padStart(2,"0")}`,usd:100}));
    if(omission)days.splice(4,1); else days[4].usd=1;
    proto(d).metrics={fees:{days,change_7d_bp:100}};
    assert.ok((await check(d)).some(f=>f.includes("F8")));
  }
});
test("unassessed API, history, badge, Markdown and llms lines cannot leak stages", async () => {
  const base=fixture(); Object.assign(row(base),{chain:"hyper_evm",assessment:"unassessed",stage:null,ratings:null,control_class:null});
  Object.assign(proto(base),{chain:"hyper_evm",assessment:"unassessed",score:null,history:[]});proto(base).facts.chain="hyper_evm";
  base["/protocols/p.md"]="Unassessed";base["/badges/p.svg"]="Unassessed";base["/llms.txt"]="[p](/protocols/p.md): Unassessed";
  base["/protocols/p/index.html"]='<title>p</title><meta name="description" content="Unassessed">';
  assert.deepEqual(await check(base),[]);
  for(const path of ["/protocols/p.md","/badges/p.svg","/llms.txt"]) {
    const d=structuredClone(base);d[path]+=" Stage 0";
    assert.ok((await check(d)).some(f=>f.includes("F5 unassessed export")));
  }
  const d=structuredClone(base);proto(d).history=[{stage:"stage_0"}];
  assert.ok((await check(d)).some(f=>f.includes("F5 unassessed leak")));
  const meta=structuredClone(base);meta["/protocols/p/index.html"]='<meta property="og:title" content="p Stage 0">';
  assert.ok((await check(meta)).some(f=>f.includes("F5 unassessed metadata")));
});
test("delay boundaries, PDA inheritance and config/admin bypasses", () => {
  const multi=secs=>({kind:"multisig",time_lock_secs:secs});
  assert.equal(controllerClass(multi(0)),"multisig_without_delay");
  assert.equal(controllerClass(multi(600)),"multisig_short_delay");
  assert.equal(controllerClass(multi(86400)),"multisig_with_delay");
  assert.equal(controllerClass({...multi(86400),config_authority:{kind:"single_key"}}),"single_key");
  assert.equal(controllerClass({kind:"program_controlled",upgrade:multi(43200)}),"multisig_short_delay");
  assert.equal(controllerClass({kind:"timelock",min_delay_secs:172800,proposer:multi(86400),admin:{kind:"single_key"}}),"single_key");
});
test("Squads source labels use decoded version", async()=>{
  const d=fixture();proto(d).facts.programs[program].upgrade_control.value={kind:"multisig",address:"m",multisig_kind:"squads_v3",time_lock_secs:0};
  row(d).control_class="multisig_without_delay";proto(d).facts.sources=[{kind:"multisig",address:"m",label:"Squads v4"}];
  assert.ok((await check(d)).some(f=>f.includes("F12")));
});
