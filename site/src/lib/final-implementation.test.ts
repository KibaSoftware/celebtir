import { describe,it,expect } from "vitest";
import { DEFAULT_VIEW,parseView,serializeView } from "./viewState";
import { pageChain } from "./scope";

describe("final research contract",()=>{
  it("restores token grouping, independent collapse and evidence selection",()=>{
    const view={...DEFAULT_VIEW,tokenGroup:"issuer" as const,tokenGroups:{Circle:true,Marinade:false},tokenDetail:"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v.supply_control"};
    expect(parseView(new URLSearchParams(serializeView(view)))).toEqual(view);
  });
  it("ignores invalid token detail and malformed group state",()=>{
    expect(parseView(new URLSearchParams("tdetail=bogus&tgroups=oops&tgroup=unknown"))).toEqual(DEFAULT_VIEW);
  });
  it("fixes native HyperEVM context without guessing from an address",()=>{
    expect(pageChain("/hyperevm/",{})).toBe("hyperliquid");
  });
});
