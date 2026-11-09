import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { buildPath, currentRouteParams, ROUTES } from "@/utils/routes";

function CertificationPage() {
  const params = currentRouteParams(ROUTES.certification);

  return (
    <PageShell title="Certification" path={buildPath(ROUTES.certification, params)}>
      {null}
    </PageShell>
  );
}

export function Certification() {
  return (
    <AppRoot>
      <ProtectedRoute>
        <CertificationPage />
      </ProtectedRoute>
    </AppRoot>
  );
}
