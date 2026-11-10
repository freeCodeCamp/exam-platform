import { Flex, HStack, IconButton } from "@chakra-ui/react";
import type { IconType } from "react-icons";

import { FccWordmark } from "@/components/fcc-wordmark";
import { ROUTES } from "@/utils/routes";

export interface NavItem {
  href: string;
  label: string;
  icon: IconType;
}

export function SiteHeader({ nav = [], path }: { nav?: NavItem[]; path?: string }) {
  return (
    <Flex as="header" borderBottomWidth="1px" bg="bg">
      <Flex
        w="full"
        maxW="1200px"
        mx="auto"
        px="6"
        h="14"
        align="center"
        justify="space-between"
        gap="4"
      >
        <Flex asChild color="fg" flexShrink="0">
          <a href={ROUTES.home} aria-label="Exam Platform home">
            <FccWordmark height="24" aria-hidden />
          </a>
        </Flex>
        {nav.length > 0 && (
          <HStack as="nav" aria-label="Main" gap="1">
            {nav.map(({ href, label, icon: Icon }) => {
              const active = path !== undefined && isActive(href, path);
              return (
                <IconButton
                  key={href}
                  asChild
                  variant="ghost"
                  size="lg"
                  bg={active ? "bg.muted" : undefined}
                  color={active ? "brand.fg" : "fg.muted"}
                >
                  <a
                    href={href}
                    aria-label={label}
                    title={label}
                    aria-current={active ? "page" : undefined}
                  >
                    <Icon size="22" aria-hidden />
                  </a>
                </IconButton>
              );
            })}
          </HStack>
        )}
      </Flex>
    </Flex>
  );
}

export function isActive(href: string, path: string): boolean {
  return href === path || (href !== ROUTES.home && path.startsWith(`${href}/`));
}
