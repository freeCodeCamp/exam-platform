import { Box, Button, Center, Heading, HStack, SimpleGrid, Stack, Text } from "@chakra-ui/react";
import type { IconType } from "react-icons";
import {
  LuAward,
  LuCalendarCheck,
  LuDownload,
  LuIdCard,
  LuMonitor,
  LuPlug,
  LuSearchX,
} from "react-icons/lu";

import { AppRoot } from "@/components/app-root";
import { BackLink } from "@/components/back-link";
import { EmptyState } from "@/components/empty-state";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { buildPath, currentRouteParams, ROUTES } from "@/utils/routes";
import type { Attempt as AttemptData } from "@/utils/types";

type StepState = "done" | "current" | "pending";

const STEP_COLORS: Record<StepState, { color: string; borderColor: string }> = {
  done: { color: "positive.fg", borderColor: "border" },
  current: { color: "brand.fg", borderColor: "brand.solid" },
  pending: { color: "fg.muted", borderColor: "border" },
};

function Step({
  icon: Icon,
  label,
  title,
  state,
}: {
  icon: IconType;
  label: string;
  title: string;
  state: StepState;
}) {
  return (
    <Stack
      as="li"
      title={title}
      aria-current={state === "current" ? "step" : undefined}
      align="center"
      gap="2"
      px="2"
      py="4"
      bg="bg.panel"
      borderWidth="1px"
      rounded="10px"
      {...STEP_COLORS[state]}
    >
      <Icon size="24" aria-hidden />
      <Text fontSize="sm" fontWeight="semibold">
        {label}
      </Text>
    </Stack>
  );
}

function AttemptDetails({ attempt }: { attempt: AttemptData }) {
  const connected = attempt.desktopConnected;

  return (
    <Stack gap="7">
      <HStack gap="4">
        <BackLink href={buildPath(ROUTES.exam, { examId: attempt.examId })} label="Back to exam" />
        <Heading as="h1" fontSize="32px" lineHeight="1.15">
          {attempt.examName}
        </Heading>
      </HStack>

      <SimpleGrid as="ol" aria-label="Progress" listStyleType="none" columns={4} gap="2">
        <Step
          icon={LuIdCard}
          label="Verified"
          title="Identity verified"
          state={attempt.identityVerified ? "done" : "current"}
        />
        <Step icon={LuCalendarCheck} label="Registered" title="Registered" state="done" />
        <Step
          icon={LuMonitor}
          label="Connected"
          title="Desktop app"
          state={connected ? "done" : attempt.identityVerified ? "current" : "pending"}
        />
        <Step icon={LuAward} label="Exam" title="Exam" state="pending" />
      </SimpleGrid>

      <Stack
        as="section"
        aria-label="Desktop app"
        align="center"
        gap="6"
        px="6"
        py="12"
        bg="bg.panel"
        borderWidth="1px"
        rounded="14px"
        textAlign="center"
      >
        <Box position="relative">
          <Center boxSize="24" rounded="full" bg="bg.muted">
            <LuMonitor size="44" aria-hidden />
          </Center>
          <Box
            position="absolute"
            right="1.5"
            bottom="1.5"
            boxSize="4.5"
            rounded="full"
            borderWidth="3px"
            borderColor="bg.panel"
            bg={connected ? "positive.fg" : "fg.muted"}
          />
        </Box>
        <Text
          role="status"
          px="3.5"
          py="1.5"
          rounded="full"
          bg={connected ? "positive.subtle" : "bg.muted"}
          color={connected ? "positive.fg" : "fg.muted"}
          fontSize="15px"
          fontWeight="semibold"
        >
          {connected ? "Connected" : "Not connected"}
        </Text>
        {!connected && (
          <Button colorPalette="brand" h="60px" px="8" fontWeight="bold" fontSize="lg" disabled>
            <LuPlug aria-hidden />
            Connect Desktop App
          </Button>
        )}
        <HStack asChild gap="2" minH="11" fontSize="15px">
          <a href="#download">
            <LuDownload size="20" aria-hidden />
            Get the app
          </a>
        </HStack>
      </Stack>
    </Stack>
  );
}

function AttemptPage({ attempt }: { attempt: AttemptData | null }) {
  const params = currentRouteParams(ROUTES.attempt);

  return (
    <PageShell path={buildPath(ROUTES.attempt, params)} maxW="880px" pt="8">
      {attempt ? (
        <AttemptDetails attempt={attempt} />
      ) : (
        <Stack gap="6">
          <BackLink href={ROUTES.exams} label="Back to exams" />
          <EmptyState icon={LuSearchX} title="Attempt not found" />
        </Stack>
      )}
    </PageShell>
  );
}

export function Attempt({ attempt = null }: { attempt?: AttemptData | null }) {
  return (
    <AppRoot>
      <ProtectedRoute>
        <AttemptPage attempt={attempt} />
      </ProtectedRoute>
    </AppRoot>
  );
}
