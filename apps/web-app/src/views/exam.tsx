import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { buildPath, currentRouteParams, ROUTES } from "@/utils/routes";

function ExamPage() {
  const params = currentRouteParams(ROUTES.exam);

  return (
    <PageShell title="Exam" path={buildPath(ROUTES.exam, params)}>
      {null}
    </PageShell>
  );
}

export function Exam() {
  return (
    <AppRoot>
      <ProtectedRoute>
        <ExamPage />
      </ProtectedRoute>
    </AppRoot>
  );
}
