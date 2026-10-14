import { createRootRoute, Link, Outlet } from "@tanstack/react-router";

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  return (
    <>
      <nav aria-label="Main navigation">
        <Link to="/" activeOptions={{ exact: true }}>
          Home
        </Link>{" "}
        <Link to="/exam">Take Practice Exam</Link>
      </nav>
      <main>
        <Outlet />
      </main>
    </>
  );
}
