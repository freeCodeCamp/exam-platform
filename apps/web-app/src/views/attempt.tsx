import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { buildPath, currentRouteParams, ROUTES } from "@/utils/routes";

function AttemptPage() {
  const params = currentRouteParams(ROUTES.attempt);

  return (
    <PageShell title="Attempt" path={buildPath(ROUTES.attempt, params)}>
      {null}
    </PageShell>
  );
}

export function Attempt() {
  return (
    <AppRoot>
      <ProtectedRoute>
        <AttemptPage />
      </ProtectedRoute>
    </AppRoot>
  );
}
