import { Box } from "@chakra-ui/react";
import type { ReactNode } from "react";
import { LuAward, LuClipboardList, LuLogIn, LuUserRound } from "react-icons/lu";

import { SiteHeader, type NavItem } from "@/components/site-header";
import { useAuth } from "@/contexts/auth";
import type { SessionUser } from "@/utils/fetch";
import { buildPath, ROUTES } from "@/utils/routes";

function navItems(user: SessionUser | null): NavItem[] {
  if (!user) {
    return [{ href: ROUTES.login, label: "Sign in", icon: LuLogIn }];
  }

  const userId = user.id;
  return [
    { href: ROUTES.exams, label: "Exams", icon: LuClipboardList },
    {
      href: buildPath(ROUTES.certifications, { userId }),
      label: "Certifications",
      icon: LuAward,
    },
    { href: buildPath(ROUTES.profile, { userId }), label: "Profile", icon: LuUserRound },
  ];
}

export function PageShell({
  path,
  maxW = "1200px",
  pt = "12",
  children,
}: {
  path: string;
  maxW?: string;
  pt?: string;
  children: ReactNode;
}) {
  const { user } = useAuth();

  return (
    <Box minH="100dvh" bg="bg" color="fg">
      <SiteHeader nav={navItems(user)} path={path} />
      <Box as="main" maxW={maxW} mx="auto" px="6" pt={pt} pb="16">
        {children}
      </Box>
    </Box>
  );
}
