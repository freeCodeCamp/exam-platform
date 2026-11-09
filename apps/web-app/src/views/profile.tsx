import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { buildPath, currentRouteParams, ROUTES } from "@/utils/routes";

function ProfilePage() {
  const params = currentRouteParams(ROUTES.profile);

  return (
    <PageShell title="Profile" path={buildPath(ROUTES.profile, params)}>
      {null}
    </PageShell>
  );
}

export function Profile() {
  return (
    <AppRoot>
      <ProtectedRoute>
        <ProfilePage />
      </ProtectedRoute>
    </AppRoot>
  );
}
