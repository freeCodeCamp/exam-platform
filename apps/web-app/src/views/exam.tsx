import { Box, Button, Flex, Heading, HStack, SimpleGrid, Stack, Text } from "@chakra-ui/react";
import type { IconType } from "react-icons";
import {
  LuArrowRight,
  LuAward,
  LuCalendarCheck,
  LuCalendarPlus,
  LuCircle,
  LuCircleCheck,
  LuCircleHelp,
  LuClock,
  LuIdCard,
  LuLock,
  LuMonitor,
  LuRotateCcw,
  LuSearchX,
  LuTarget,
} from "react-icons/lu";

import { AppRoot } from "@/components/app-root";
import { BackLink } from "@/components/back-link";
import { EmptyState } from "@/components/empty-state";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { buildPath, currentRouteParams, ROUTES } from "@/utils/routes";
import type { ExamDetail } from "@/utils/types";

function StatTile({ icon: Icon, label, value }: { icon: IconType; label: string; value: string }) {
  return (
    <Stack as="li" title={label} gap="2.5" p="4.5" bg="bg.panel" borderWidth="1px" rounded="10px">
      <Box color="link">
        <Icon size="22" role="img" aria-label={label} />
      </Box>
      <Text fontSize="22px" fontWeight="bold">
        {value}
      </Text>
    </Stack>
  );
}

function ExamActions({ exam }: { exam: ExamDetail }) {
  const primary = { colorPalette: "brand", h: "14", fontWeight: "bold", fontSize: "17px" };

  switch (exam.status) {
    case "open":
      return (
        <Button {...primary} w="full" disabled>
          <LuCalendarPlus aria-hidden />
          Register for exam
        </Button>
      );
    case "registered":
      return (
        <>
          <HStack gap="2.5" h="11" color="positive.fg" fontWeight="semibold">
            <LuCalendarCheck size="22" aria-hidden />
            Registered
          </HStack>
          {exam.attemptId && (
            <Button asChild {...primary}>
              <a href={buildPath(ROUTES.attempt, { attemptId: exam.attemptId })}>
                Attempt
                <LuArrowRight aria-hidden />
              </a>
            </Button>
          )}
        </>
      );
    case "passed":
      return (
        <HStack gap="2.5" h="11" color="positive.fg" fontWeight="semibold">
          <LuAward size="22" aria-hidden />
          Passed
        </HStack>
      );
    case "locked":
      return (
        <HStack gap="2.5" h="11" color="fg.muted" fontWeight="semibold">
          <LuLock size="22" aria-hidden />
          Prerequisites needed
        </HStack>
      );
  }
}

function ExamDetails({ exam }: { exam: ExamDetail }) {
  return (
    <Flex wrap="wrap" align="flex-start" gap="8">
      <Stack flex="999 1 560px" minW="0" gap="7">
        <Stack gap="3">
          <Heading as="h1" fontSize="40px" lineHeight="1.1">
            {exam.name}
          </Heading>
          <Text fontSize="lg" lineHeight="1.5" color="fg.muted" maxW="620px">
            {exam.description}
          </Text>
        </Stack>

        <SimpleGrid
          as="ul"
          aria-label="Exam details"
          listStyleType="none"
          columns={{ base: 2, md: 4 }}
          gap="3"
        >
          <StatTile icon={LuClock} label="Duration" value={`${exam.durationMinutes} min`} />
          <StatTile icon={LuCircleHelp} label="Questions" value={String(exam.questionCount)} />
          <StatTile icon={LuTarget} label="Pass mark" value={`${exam.passPercent}%`} />
          <StatTile icon={LuRotateCcw} label="Retake after" value={`${exam.retakeHours} h`} />
        </SimpleGrid>

        {exam.prerequisites.length > 0 && (
          <Stack gap="3">
            <Heading
              as="h2"
              fontFamily="body"
              fontSize="15px"
              fontWeight="semibold"
              textTransform="uppercase"
              letterSpacing="0.06em"
              color="fg.muted"
            >
              Requires
            </Heading>
            <Stack as="ul" listStyleType="none" gap="2">
              {exam.prerequisites.map((prerequisite) => {
                const KindIcon = prerequisite.kind === "identity" ? LuIdCard : LuAward;
                return (
                  <HStack
                    as="li"
                    key={prerequisite.name}
                    gap="3"
                    px="4"
                    py="3.5"
                    bg="bg.panel"
                    borderWidth="1px"
                    rounded="10px"
                  >
                    {prerequisite.met ? (
                      <Box color="positive.fg">
                        <LuCircleCheck size="22" role="img" aria-label="Done" />
                      </Box>
                    ) : (
                      <Box color="fg.muted">
                        <LuCircle size="22" role="img" aria-label="Not done" />
                      </Box>
                    )}
                    <Box color="fg.muted">
                      <KindIcon size="20" aria-hidden />
                    </Box>
                    <Text>{prerequisite.name}</Text>
                  </HStack>
                );
              })}
            </Stack>
          </Stack>
        )}
      </Stack>

      <Stack as="aside" flex="1 1 300px" gap="4" p="6" bg="bg.panel" borderWidth="1px" rounded="l3">
        <ExamActions exam={exam} />
        <HStack gap="2.5" color="fg.muted" fontSize="15px">
          <LuMonitor size="20" aria-hidden />
          Desktop app required
        </HStack>
      </Stack>
    </Flex>
  );
}

function ExamPage({ exam }: { exam: ExamDetail | null }) {
  const params = currentRouteParams(ROUTES.exam);

  return (
    <PageShell path={buildPath(ROUTES.exam, params)} pt="8">
      <Stack gap="6">
        <BackLink href={ROUTES.exams} label="Back to exams" />
        {exam ? (
          <ExamDetails exam={exam} />
        ) : (
          <EmptyState icon={LuSearchX} title="Exam not found" />
        )}
      </Stack>
    </PageShell>
  );
}

export function Exam({ exam = null }: { exam?: ExamDetail | null }) {
  return (
    <AppRoot>
      <ProtectedRoute>
        <ExamPage exam={exam} />
      </ProtectedRoute>
    </AppRoot>
  );
}
