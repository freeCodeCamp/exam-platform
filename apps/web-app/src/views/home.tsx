import { Button, Center, Flex, Heading, HStack, IconButton, Stack, Text } from "@chakra-ui/react";
import type { IconType } from "react-icons";
import {
  LuAward,
  LuCalendarPlus,
  LuClipboardList,
  LuIdCard,
  LuLogIn,
  LuMonitor,
} from "react-icons/lu";

import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { useAuth } from "@/contexts/auth";
import { ROUTES } from "@/utils/routes";

const STEPS: { label: string; icon: IconType; color: string }[] = [
  { label: "Verify", icon: LuIdCard, color: "notice.fg" },
  { label: "Register", icon: LuCalendarPlus, color: "lilac.fg" },
  { label: "Sit exam", icon: LuMonitor, color: "brand.fg" },
  { label: "Get certified", icon: LuAward, color: "positive.fg" },
];

function HomePage() {
  const { user } = useAuth();

  return (
    <PageShell path={ROUTES.home} pt="88px">
      <Stack gap="20">
        <Stack gap="6" maxW="680px">
          <Heading
            as="h1"
            fontSize={{ base: "44px", md: "60px" }}
            lineHeight="1.05"
            letterSpacing="-0.02em"
          >
            Exam Platform
          </Heading>
          <HStack gap="3" wrap="wrap">
            <Button asChild colorPalette="brand" h="52px" px="6" fontWeight="bold" fontSize="17px">
              <a href={ROUTES.exams}>
                <LuClipboardList aria-hidden />
                Exams
              </a>
            </Button>
            {!user && (
              <IconButton asChild variant="outline" boxSize="52px" borderWidth="2px" color="fg">
                <a href={ROUTES.login} aria-label="Sign in" title="Sign in">
                  <LuLogIn size="22" aria-hidden />
                </a>
              </IconButton>
            )}
          </HStack>
        </Stack>

        <Flex as="section" aria-label="How it works" wrap="wrap" gap="4">
          {STEPS.map(({ label, icon: Icon, color }, i) => (
            <Stack
              key={label}
              flex="1 1 220px"
              gap="4"
              p="7"
              bg="bg.panel"
              borderWidth="1px"
              rounded="l3"
            >
              <Center boxSize="52px" rounded="full" bg="bg.muted" color={color}>
                <Icon size="26" aria-hidden />
              </Center>
              <HStack align="baseline" gap="2.5">
                <Text fontFamily="heading" fontSize="sm" color="fg.muted">
                  {String(i + 1).padStart(2, "0")}
                </Text>
                <Text fontSize="lg" fontWeight="semibold">
                  {label}
                </Text>
              </HStack>
            </Stack>
          ))}
        </Flex>
      </Stack>
    </PageShell>
  );
}

export function Home() {
  return (
    <AppRoot>
      <HomePage />
    </AppRoot>
  );
}
