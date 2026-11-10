import { Stack } from "@chakra-ui/react";
import { LuSearchX } from "react-icons/lu";

import { AppRoot } from "@/components/app-root";
import { BackLink } from "@/components/back-link";
import { CertificationCard } from "@/components/certification-card";
import { EmptyState } from "@/components/empty-state";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { buildPath, currentRouteParams, ROUTES } from "@/utils/routes";
import type { Certification as CertificationData } from "@/utils/types";

function CertificationPage({ certification }: { certification: CertificationData | null }) {
  const params = currentRouteParams(ROUTES.certification);

  return (
    <PageShell path={buildPath(ROUTES.certification, params)} maxW="760px" pt="8">
      <Stack gap="6">
        <BackLink
          href={buildPath(ROUTES.certifications, { userId: params.userId })}
          label="Back to certifications"
        />
        {certification ? (
          <CertificationCard userId={params.userId} certification={certification} />
        ) : (
          <EmptyState icon={LuSearchX} title="Certification not found" />
        )}
      </Stack>
    </PageShell>
  );
}

export function Certification({
  certification = null,
}: {
  certification?: CertificationData | null;
}) {
  return (
    <AppRoot>
      <ProtectedRoute>
        <CertificationPage certification={certification} />
      </ProtectedRoute>
    </AppRoot>
  );
}
