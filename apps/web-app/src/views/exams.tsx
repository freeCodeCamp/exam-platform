import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { ROUTES } from "@/utils/routes";

export function Exams() {
  return (
    <AppRoot>
      <ProtectedRoute>
        <PageShell title="Exams" path={ROUTES.exams}>
          {null}
        </PageShell>
      </ProtectedRoute>
    </AppRoot>
  );
}
