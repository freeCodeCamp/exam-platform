import { Container, Flex, Heading, HStack, Link } from "@chakra-ui/react";
import type { ReactNode } from "react";
import { ColorModeButton } from "@/components/ui/color-mode";

export function PageShell({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <Container maxW="5xl" py="6">
      <Flex as="header" justify="space-between" align="center" mb="6">
        <HStack gap="4">
          <Link href="/">Home</Link>
        </HStack>
        <ColorModeButton />
      </Flex>
      <Heading as="h1" size="2xl" mb="4">
        {title}
      </Heading>
      <main>{children}</main>
    </Container>
  );
}
