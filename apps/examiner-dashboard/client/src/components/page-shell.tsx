import { Box, Button, Center, Flex, Heading, HStack, Text } from "@chakra-ui/react";
import type { ReactNode } from "react";
import { LuBlocks, LuGraduationCap, LuHouse } from "react-icons/lu";
import { ColorModeButton } from "@/components/ui/color-mode";

const navItems = [
  { href: "/", label: "Home", icon: LuHouse },
  { href: "/question-builder", label: "Question Builder", icon: LuBlocks },
];

export function PageShell({
  title,
  description,
  path,
  children,
}: {
  title: string;
  description?: string;
  // Path of the current page, used to highlight its nav item.
  path: string;
  children: ReactNode;
}) {
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
            Examiner Dashboard
          </Text>
        </HStack>
        {navItems.map(({ href, label, icon: Icon }) => {
          const active = href === path;
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
