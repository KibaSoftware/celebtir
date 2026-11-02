import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { DATA_STATES, DataState } from "./DataState";
import { FreshnessBar } from "./FreshnessBar";
import { RatingBadge } from "./RatingBadge";

afterEach(cleanup);

const info = {
  date: "2026-10-17",
  finishedAt: "2026-10-17T02:20:00Z",
  slot: 458_020_325,
  methodology: "0.1",
  demo: false,
};

describe("ui_2 freshness bar", () => {
  it("ui_2 shows the snapshot date, slot and methodology version", () => {
    render(<FreshnessBar info={info} now={Date.parse("2026-10-17T09:00:00Z")} />);
    expect(screen.getByText("2026-10-17")).toBeTruthy();
    expect(screen.getByText("Solana slot 458,020,325")).toBeTruthy();
    expect(screen.getByText("methodology v0.1")).toBeTruthy();
    expect(document.querySelector("[data-stale]")).toBeNull();
  });

  it("ui_2 a 2-day-old snapshot shows the warning", () => {
    render(<FreshnessBar info={info} now={Date.parse("2026-10-19T03:00:00Z")} />);
    expect(screen.getByRole("status").textContent).toContain("2 days old");
  });

  it("ui_2 the warning starts after 36 hours", () => {
    render(<FreshnessBar info={info} now={Date.parse("2026-10-18T14:19:00Z")} />);
    expect(document.querySelector("[data-stale]")).toBeNull();
    cleanup();
    render(<FreshnessBar info={info} now={Date.parse("2026-10-18T14:21:00Z")} />);
    expect(document.querySelector("[data-stale]")).not.toBeNull();
  });

  it("ui_2 demo data is labelled", () => {
    render(<FreshnessBar info={{ ...info, demo: true }} now={Date.parse("2026-10-17T09:00:00Z")} />);
    expect(screen.getByText(/Demo data/)).toBeTruthy();
  });
});

describe("ui_7 data states", () => {
  it("ui_7 the four states render with one consistent wording", () => {
    const { container } = render(
      <div>
        <DataState kind="unknown" />
        <DataState kind="unrated" />
        <DataState kind="not_applicable" />
        <DataState kind="unavailable" />
      </div>,
    );
    expect(container.innerHTML).toMatchSnapshot();
    for (const s of Object.values(DATA_STATES)) {
      expect(screen.getByText(s.label)).toBeTruthy();
    }
  });

  it("ui_7 rating badges use the same words for Unknown and Not applicable", () => {
    render(
      <div>
        <RatingBadge rating="unknown" />
        <RatingBadge rating="not_applicable" />
      </div>,
    );
    expect(screen.getByText(DATA_STATES.unknown.label)).toBeTruthy();
    expect(screen.getByText(DATA_STATES.not_applicable.label)).toBeTruthy();
  });
});
