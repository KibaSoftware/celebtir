import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { RatingBadge } from "./RatingBadge";

afterEach(cleanup);

const ALL = ["low", "medium", "high", "unknown", "not_applicable"] as const;

describe("ui_24 RatingBadge", () => {
  it("ui_24 every rating shows a text label and an icon, never color alone", () => {
    const labels = { low: "Low", medium: "Medium", high: "High", unknown: "Unknown", not_applicable: "Not applicable" };
    for (const rating of ALL) {
      const { container, unmount } = render(<RatingBadge rating={rating} />);
      expect(screen.getByText(labels[rating])).toBeTruthy();
      expect(container.querySelector("svg[aria-hidden='true']")).toBeTruthy();
      unmount();
    }
  });

  it("ui_24 the compact badge keeps its label for screen readers", () => {
    render(<RatingBadge rating="high" compact />);
    expect(screen.getByText("High")).toBeTruthy();
  });

  it("ui_24 each rating has a distinct icon shape", () => {
    const shapes = new Set(
      ALL.map((r) => {
        const { container, unmount } = render(<RatingBadge rating={r} />);
        const html = container.querySelector("svg")!.innerHTML;
        unmount();
        return html.replace(/#[0-9a-f]{6}/gi, "");
      }),
    );
    expect(shapes.size).toBe(ALL.length);
  });
});
