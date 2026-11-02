import { afterEach, expect, it } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { LimitedItems } from "./LimitedItems";
import { HashFocus } from "./protocol/HashFocus";

afterEach(() => { cleanup(); history.replaceState(null, "", "/"); });

it("previews ten ordered records, retains all records, and restores the preview", () => {
  render(<LimitedItems ordered label="events">{Array.from({length: 13}, (_, i) => <li key={i}>Event {i}</li>)}</LimitedItems>);
  const list = screen.getByRole("list", {name: "events"});
  expect(list.tagName).toBe("OL");
  expect(within(list).getAllByRole("listitem")).toHaveLength(11); // ten records + disclosure
  expect(list.querySelectorAll("li")).toHaveLength(14);
  fireEvent.click(screen.getByRole("button", {name: "Show all 13 events"}));
  expect(screen.getByText("Showing 13 of 13 events")).toBeTruthy();
  expect(within(list).getAllByRole("listitem")).toHaveLength(14);
  fireEvent.click(screen.getByRole("button", {name: "Show fewer events"}));
  expect(within(list).getAllByRole("listitem")).toHaveLength(11);
});

it("reveals nested hidden records and closed disclosures on initial hash and history navigation", async () => {
  history.replaceState(null, "", "/#deep-target");
  render(<><HashFocus prefix="" /><LimitedItems label="outer">{Array.from({length: 12}, (_, i) => <li key={i}>{i === 11 ? <details><summary>Record details</summary><LimitedItems label="inner">{Array.from({length: 12}, (_, j) => <li key={j}><span id={j === 11 ? "deep-target" : undefined}>Nested {j}</span></li>)}</LimitedItems></details> : `Record ${i}`}</li>)}</LimitedItems></>);
  const target = document.getElementById("deep-target")!;
  await waitFor(() => expect(document.activeElement).toBe(target));
  expect(target.closest("[hidden]")).toBeNull();
  expect(target.closest("details")?.open).toBe(true);
  fireEvent.click(screen.getByRole("button", {name: "Show fewer inner"}));
  expect(target.closest("[hidden]")).not.toBeNull();
  window.dispatchEvent(new PopStateEvent("popstate"));
  await waitFor(() => expect(target.closest("[hidden]")).toBeNull());
});

it("spans the actual table columns and tolerates malformed fragments", () => {
  history.replaceState(null, "", "/#%broken");
  render(<table><LimitedItems table colSpan={4} label="permissions">{Array.from({length: 12}, (_, i) => <tr key={i}><td>Permission {i}</td><td>A</td><td>B</td><td>C</td></tr>)}</LimitedItems></table>);
  expect(screen.getByRole("button", {name: "Show all 12 permissions"}).closest("td")?.colSpan).toBe(4);
});
