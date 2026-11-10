import { Center, Flex, Heading, HStack } from "@chakra-ui/react";
import type { IconType } from "react-icons";
import {
  LuArrowRight,
  LuAward,
  LuCalendarCheck,
  LuCircleHelp,
  LuClock,
  LuLock,
  LuTarget,
} from "react-icons/lu";

import { MetaItem } from "@/components/meta-item";
import { buildPath, ROUTES } from "@/utils/routes";
import type { ExamStatus, ExamSummary } from "@/utils/types";

const STATUS_BADGES: Partial<
  Record<ExamStatus, { label: string; icon: IconType; bg: string; color: string }>
> = {
  passed: { label: "Passed", icon: LuAward, bg: "positive.subtle", color: "positive.fg" },
  registered: {
    label: "Registered",
    icon: LuCalendarCheck,
    bg: "notice.subtle",
    color: "notice.fg",
  },
  locked: { label: "Prerequisites needed", icon: LuLock, bg: "bg.muted", color: "fg.muted" },
};

export function StatusBadge({ status }: { status: ExamStatus }) {
  const badge = STATUS_BADGES[status];
  if (!badge) return null;
  const { label, icon: Icon, bg, color } = badge;

  return (
    <Center
      role="img"
      aria-label={label}
      title={label}
      boxSize="9"
      flexShrink={0}
      rounded="full"
      bg={bg}
      color={color}
    >
      <Icon size="20" aria-hidden />
    </Center>
  );
}

export function ExamCard({ exam }: { exam: ExamSummary }) {
  return (
    <Flex
      asChild
      direction="column"
      justify="space-between"
      gap="7"
      p="6"
      h="full"
      bg="bg.panel"
      borderWidth="1px"
      rounded="l3"
      color="fg"
      textDecoration="none"
      _hover={{ borderColor: "brand.solid" }}
    >
      <a href={buildPath(ROUTES.exam, { examId: exam.id })}>
        <HStack align="flex-start" justify="space-between" gap="4">
          <Heading
            as="h2"
            fontFamily="body"
            fontSize="19px"
            lineHeight="1.35"
            fontWeight="semibold"
          >
            {exam.name}
          </Heading>
          <StatusBadge status={exam.status} />
        </HStack>
        <HStack justify="space-between" gap="3">
          <HStack wrap="wrap" gap="4.5" color="fg.muted" fontSize="15px">
            <MetaItem icon={LuClock} label="Duration">
              {exam.durationMinutes} min
            </MetaItem>
            <MetaItem icon={LuCircleHelp} label="Questions">
              {exam.questionCount}
            </MetaItem>
            <MetaItem icon={LuTarget} label="Pass mark">
              {exam.passPercent}%
            </MetaItem>
          </HStack>
          <Flex color="brand.fg">
            <LuArrowRight size="22" aria-hidden />
          </Flex>
        </HStack>
      </a>
    </Flex>
  );
}
