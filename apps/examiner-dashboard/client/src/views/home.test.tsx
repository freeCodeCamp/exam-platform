/* @vitest-environment jsdom */
import { render } from "@testing-library/react";
import { expect, test } from "vitest";

import { Home } from "./home.tsx";

test("trivial view test. Delete after creating real ones", async () => {
  const screen = render(<Home />);

  expect(screen.getByText(/Author/)).toBeDefined();
});
