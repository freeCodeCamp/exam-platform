import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { buildPath, currentRouteParams, ROUTES } from "@/utils/routes";

function CertificationsPage() {
  const params = currentRouteParams(ROUTES.certifications);

  return (
    <PageShell title="Certifications" path={buildPath(ROUTES.certifications, params)}>
      {null}
    </PageShell>
  );
}

export function Certifications() {
  return (
    <AppRoot>
      <ProtectedRoute>
        <CertificationsPage />
      </ProtectedRoute>
    </AppRoot>
  );
}
