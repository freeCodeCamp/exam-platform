import { EmptyState as ChakraEmptyState, VStack } from "@chakra-ui/react";
import type { ReactNode } from "react";
import type { IconType } from "react-icons";

export function EmptyState({
  icon: Icon,
  title,
  children,
}: {
  icon: IconType;
  title: string;
  children?: ReactNode;
}) {
  return (
    <ChakraEmptyState.Root borderWidth="2px" borderStyle="dashed" rounded="l3" py="12">
      <ChakraEmptyState.Content>
        <ChakraEmptyState.Indicator color="fg.muted">
          <Icon aria-hidden />
        </ChakraEmptyState.Indicator>
        <VStack textAlign="center">
          <ChakraEmptyState.Title fontSize="lg">{title}</ChakraEmptyState.Title>
        </VStack>
        {children}
      </ChakraEmptyState.Content>
    </ChakraEmptyState.Root>
  );
}
