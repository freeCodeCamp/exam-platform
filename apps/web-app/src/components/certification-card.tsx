import { Center, Flex, Heading, HStack, IconButton } from "@chakra-ui/react";
import { LuAward, LuCalendar, LuLink, LuTarget } from "react-icons/lu";

import { MetaItem } from "@/components/meta-item";
import { buildPath, ROUTES } from "@/utils/routes";
import type { Certification } from "@/utils/types";

export function certificationUrl(userId: string, certificationId: string): string {
  return new URL(
    buildPath(ROUTES.certification, { userId, certificationId }),
    window.location.origin,
  ).href;
}

export function CertificationCard({
  userId,
  certification,
}: {
  userId: string;
  certification: Certification;
}) {
  return (
    <Flex direction="column" gap="6" p="7" h="full" bg="bg.panel" borderWidth="1px" rounded="l3">
      <HStack align="flex-start" gap="4">
        <Center boxSize="14" flexShrink={0} rounded="full" bg="positive.subtle" color="positive.fg">
          <LuAward size="28" aria-hidden />
        </Center>
        <Heading
          as="h2"
          fontFamily="body"
          fontSize="xl"
          lineHeight="1.3"
          fontWeight="semibold"
          pt="1"
        >
          <a href={buildPath(ROUTES.certification, { userId, certificationId: certification.id })}>
            {certification.name}
          </a>
        </Heading>
      </HStack>
      <HStack wrap="wrap" justify="space-between" gap="3">
        <HStack wrap="wrap" gap="4.5" color="fg.muted" fontSize="15px">
          <MetaItem icon={LuCalendar} label="Issued">
            {certification.issuedAt}
          </MetaItem>
          <MetaItem icon={LuTarget} label="Score">
            {certification.scorePercent}%
          </MetaItem>
        </HStack>
        <IconButton
          variant="subtle"
          size="lg"
          bg="bg.muted"
          aria-label="Copy verification link"
          title="Copy verification link"
          onClick={() =>
            void navigator.clipboard.writeText(certificationUrl(userId, certification.id))
          }
        >
          <LuLink size="20" aria-hidden />
        </IconButton>
      </HStack>
    </Flex>
  );
}
