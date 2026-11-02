import { vi } from "vitest";

// jsdom has no layout engine or ResizeObserver. Let the real React Flow
// component mount in DOM tests; geometry and resizing remain browser tests.
vi.stubGlobal("ResizeObserver", class {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
});
