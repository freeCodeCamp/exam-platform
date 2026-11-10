import { Flex, HStack, IconButton, Text, VisuallyHidden } from "@chakra-ui/react";
import type { IconType } from "react-icons";

export interface ToggleOption<T extends string> {
  value: T;
  label: string;
  icon: IconType;
}

export function ToggleGroup<T extends string>({
  legend,
  value,
  options,
  onChange,
}: {
  legend: string;
  value: T;
  options: ToggleOption<T>[];
  onChange: (value: T) => void;
}) {
  return (
    <HStack as="fieldset" flex="1 1 200px" justify="space-between" gap="3">
      <VisuallyHidden as="legend">{legend}</VisuallyHidden>
      <Text aria-hidden fontSize="15px" fontWeight="semibold" color="fg.muted">
        {legend}
      </Text>
      <Flex gap="1" p="1" bg="bg.muted" borderWidth="1px" rounded="10px">
        {options.map(({ value: option, label, icon: Icon }) => {
          const pressed = option === value;
          return (
            <IconButton
              key={option}
              size="lg"
              rounded="7px"
              variant={pressed ? "solid" : "ghost"}
              colorPalette={pressed ? "brand" : undefined}
              color={pressed ? undefined : "fg"}
              aria-pressed={pressed}
              aria-label={label}
              title={label}
              onClick={() => onChange(option)}
            >
              <Icon size="20" aria-hidden />
            </IconButton>
          );
        })}
      </Flex>
    </HStack>
  );
}
