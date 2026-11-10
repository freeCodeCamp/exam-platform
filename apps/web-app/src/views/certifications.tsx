import { Button, Flex, Heading, IconButton, SimpleGrid, Stack } from "@chakra-ui/react";
import { LuAward, LuClipboardList, LuLink, LuPlus } from "react-icons/lu";

import { AppRoot } from "@/components/app-root";
import { CertificationCard } from "@/components/certification-card";
import { EmptyState } from "@/components/empty-state";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { UserHeading } from "@/components/user-heading";
import { useAuth } from "@/contexts/auth";
import { buildPath, currentRouteParams, ROUTES } from "@/utils/routes";
import type { Certification } from "@/utils/types";

function CertificationsPage({ certifications }: { certifications: Certification[] }) {
  const { user } = useAuth();
  const { userId } = currentRouteParams(ROUTES.certifications);
  const own = user?.id === userId;

  return (
    <PageShell path={buildPath(ROUTES.certifications, { userId })}>
      <Stack gap="8">
        <Flex wrap="wrap" align="center" justify="space-between" gap="4">
          {own && user ? (
            <UserHeading name={user.name} />
          ) : (
            <Heading as="h1" fontSize="30px">
              Certifications
            </Heading>
          )}
          <IconButton
            variant="outline"
            size="xl"
            borderWidth="2px"
            color="fg"
            aria-label="Copy profile link"
            title="Copy profile link"
            onClick={() => void navigator.clipboard.writeText(window.location.href)}
          >
            <LuLink size="20" aria-hidden />
          </IconButton>
        </Flex>

        {certifications.length === 0 ? (
          <EmptyState icon={LuAward} title="No certifications">
            {own && (
              <Button asChild colorPalette="brand" fontWeight="bold">
                <a href={ROUTES.exams}>
                  <LuClipboardList aria-hidden />
                  Exams
                </a>
              </Button>
            )}
          </EmptyState>
        ) : (
          <SimpleGrid as="ul" listStyleType="none" minChildWidth="360px" gap="5">
            {certifications.map((certification) => (
              <li key={certification.id}>
                <CertificationCard userId={userId} certification={certification} />
              </li>
            ))}
            {own && (
              <Flex as="li">
                <Flex
                  asChild
                  flex="1"
                  align="center"
                  justify="center"
                  minH="168px"
                  borderWidth="2px"
                  borderStyle="dashed"
                  rounded="l3"
                  color="fg.muted"
                >
                  <a href={ROUTES.exams} aria-label="Find more exams" title="Find more exams">
                    <LuPlus size="28" aria-hidden />
                  </a>
                </Flex>
              </Flex>
            )}
          </SimpleGrid>
        )}
      </Stack>
    </PageShell>
  );
}

export function Certifications({ certifications = [] }: { certifications?: Certification[] }) {
  return (
    <AppRoot>
      <ProtectedRoute>
        <CertificationsPage certifications={certifications} />
      </ProtectedRoute>
    </AppRoot>
  );
}
