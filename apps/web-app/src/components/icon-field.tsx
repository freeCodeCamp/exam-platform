import { Input, InputGroup, VisuallyHidden, type InputProps } from "@chakra-ui/react";
import type { ReactNode } from "react";
import type { IconType } from "react-icons";

export function IconField({
  id,
  label,
  icon: Icon,
  endElement,
  ...inputProps
}: InputProps & { id: string; label: string; icon: IconType; endElement?: ReactNode }) {
  return (
    <InputGroup
      startElement={
        <label htmlFor={id} title={label}>
          <Icon size="20" aria-hidden />
          <VisuallyHidden>{label}</VisuallyHidden>
        </label>
      }
      startElementProps={{ color: "fg.muted", pointerEvents: "auto", w: "12" }}
      endElement={endElement}
    >
      <Input
        id={id}
        h="52px"
        ps="12"
        bg="bg.muted"
        borderColor="border"
        fontSize="md"
        _placeholder={{ color: "fg.muted" }}
        {...inputProps}
      />
    </InputGroup>
  );
}
