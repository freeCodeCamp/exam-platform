// Full-page navigation, replacing the current history entry. Own module so tests can mock it:
export function redirect(path: string): void {
  window.location.replace(path);
}
