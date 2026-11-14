import { navigate } from "astro:transitions/client";

export function redirect(path: string): void {
  void navigate(path, { history: "replace" });
}
