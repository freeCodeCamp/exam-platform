import { Box, Button, Center, Flex, Heading, HStack, Text } from "@chakra-ui/react";
import type { ReactNode } from "react";
import type { IconType } from "react-icons";
import {
  LuAward,
  LuClipboardList,
  LuGraduationCap,
  LuHouse,
  LuLogIn,
  LuUser,
} from "react-icons/lu";

import { ColorModeButton } from "@/components/ui/color-mode";
import { useAuth } from "@/contexts/auth";
import type { SessionUser } from "@/utils/fetch";
import { buildPath, ROUTES } from "@/utils/routes";

interface NavItem {
  href: string;
  label: string;
  icon: IconType;
}

function navItems(user: SessionUser | null): NavItem[] {
  const home = { href: ROUTES.home, label: "Home", icon: LuHouse };
  if (!user) {
    return [home, { href: ROUTES.login, label: "Log in", icon: LuLogIn }];
  }

  const userId = user.id;
  return [
    home,
    { href: ROUTES.exams, label: "Exams", icon: LuClipboardList },
    {
      href: buildPath(ROUTES.certifications, { userId }),
      label: "Certifications",
      icon: LuAward,
    },
    { href: buildPath(ROUTES.profile, { userId }), label: "Profile", icon: LuUser },
  ];
}

function isActive(href: string, path: string): boolean {
  return href === path || (href !== ROUTES.home && path.startsWith(`${href}/`));
}

export function PageShell({
  title,
  description,
  path,
  children,
}: {
  title: string;
  description?: string;
  path: string;
  children: ReactNode;
}) {
  const { user } = useAuth();

  return (
    <Flex h="100dvh" colorPalette="teal">
      <Flex
        as="nav"
        direction="column"
        w="60"
        flexShrink={0}
        bg="bg.subtle"
        borderEndWidth="1px"
        p="3"
        gap="1"
      >
        <HStack px="2" py="2" mb="4" gap="3">
          <Center boxSize="9" bg="colorPalette.solid" color="colorPalette.contrast">
            <LuGraduationCap />
          </Center>
          <Text fontWeight="bold" lineHeight="short">
            Exam Platform
          </Text>
        </HStack>
        {navItems(user).map(({ href, label, icon: Icon }) => {
          const active = isActive(href, path);
          return (
            <Button
              key={href}
              asChild
              variant={active ? "subtle" : "ghost"}
              color={active ? "colorPalette.fg" : "fg.muted"}
              justifyContent="flex-start"
              fontWeight={active ? "semibold" : "medium"}
            >
              <a href={href} aria-current={active ? "page" : undefined}>
                <Icon />
                {label}
              </a>
            </Button>
          );
        })}
        <HStack mt="auto" pt="3" borderTopWidth="1px" justify="space-between">
          <Text textStyle="sm" color="fg.muted" px="2">
            Theme
          </Text>
          <ColorModeButton />
        </HStack>
      </Flex>
      <Flex as="main" direction="column" flex="1" minW="0">
        <Box px="6" py="4" borderBottomWidth="1px">
          <Heading as="h1" size="2xl">
            {title}
          </Heading>
          {description && (
            <Text color="fg.muted" textStyle="sm">
              {description}
            </Text>
          )}
        </Box>
        <Box flex="1" minH="0" overflow="auto">
          {children}
        </Box>
      </Flex>
    </Flex>
  );
}
