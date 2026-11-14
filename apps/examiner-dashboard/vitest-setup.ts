import "@testing-library/jest-dom/vitest";
import { afterEach, vi } from "vitest";
import { queryClient } from "./client/src/utils/query-client.js";

afterEach(() => {
  queryClient.clear();
});

Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});
