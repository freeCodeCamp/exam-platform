import { Center, Heading, HStack, Stack, Text } from "@chakra-ui/react";
import type { ReactNode } from "react";

import { initials } from "@/utils/initials";

export function UserHeading({ name, children }: { name: string; children?: ReactNode }) {
  return (
    <HStack gap="4">
      <Center
        aria-hidden
        boxSize="14"
        flexShrink={0}
        rounded="full"
        bg="bg.muted"
        fontFamily="heading"
        fontWeight="bold"
        fontSize="xl"
      >
        {initials(name)}
      </Center>
      <Stack gap="1">
        <Heading as="h1" fontSize="30px">
          {name}
        </Heading>
        {children && (
          <Text as="div" color="fg.muted" fontSize="15px">
            {children}
          </Text>
        )}
      </Stack>
    </HStack>
  );
}
