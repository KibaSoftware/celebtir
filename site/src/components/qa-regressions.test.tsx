import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { ReasonText, GroupedReason } from "./ReasonText";
import { LimitedItems } from "./LimitedItems";
import ApiPage from "@/app/api/page";
import { existsSync } from "node:fs";
import path from "node:path";

afterEach(cleanup);
it("ui_12 qa reasons render code and group duplicate program evidence", () => {
  const address = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";
  render(<><ReasonText text="`authority` can change parameters" /><GroupedReason text={`${address}: same reason; ${address}: same reason`} /></>);
  expect(document.querySelector("code")?.textContent).toBe("authority");
  expect(screen.getByText(/2 programs:/)).toBeTruthy();
  expect(document.querySelector("details")?.open).toBe(false);
});
it("ui_23 qa long graph lists reveal all evidence on demand", () => {
  render(<LimitedItems label="test evidence">{Array.from({ length: 29 }, (_, i) => <li key={i}>Entry {i}</li>)}</LimitedItems>);
  expect(screen.getByText("Entry 28").hidden).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: "Show all 29 test evidence" }));
  expect(screen.getByText("Entry 28").hidden).toBe(false);
  fireEvent.click(screen.getByRole("button", { name: "Show fewer test evidence" }));
  expect(screen.getByText("Entry 28").hidden).toBe(true);
});
it("api_1 qa every documented schema links a published file", () => {
  render(<ApiPage />);
  const links = [...document.querySelectorAll<HTMLAnchorElement>('a[href^="/data/v2/schemas/"]')];
  expect(links.map(a => a.textContent)).toEqual(["summary.schema.json", "protocol.schema.json", "token.schema.json", "events.schema.json"]);
  for (const link of links) expect(existsSync(path.join(process.cwd(), "public", link.getAttribute("href")!))).toBe(true);
});

it("sco_9 ui_12 legacy delay text does not claim withdrawal is impossible", () => {
  render(<ReasonText seen={new Set()} text="Funds can be stolen if a harmful upgrade is approved, because it takes effect after 24 h, before users can withdraw." />);
  expect(document.body.textContent).toContain("An approved upgrade takes effect after 24 h.");
  expect(document.body.textContent).not.toContain("stolen");
  expect(document.body.textContent).not.toContain("before users can withdraw");
});
