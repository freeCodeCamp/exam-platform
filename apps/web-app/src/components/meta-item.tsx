import { HStack, VisuallyHidden } from "@chakra-ui/react";
import type { ReactNode } from "react";
import type { IconType } from "react-icons";

export function MetaItem({
  icon: Icon,
  label,
  children,
}: {
  icon: IconType;
  label: string;
  children: ReactNode;
}) {
  return (
    <HStack as="span" gap="1.5" title={label}>
      <Icon size="18" aria-hidden />
      <VisuallyHidden>{label}:</VisuallyHidden>
      {children}
    </HStack>
  );
}
