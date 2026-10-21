import { VStack, RadioGroup } from "@chakra-ui/react";

interface Props {
  prompt: string;
  options: string[];
}

export const MultipleChoice = ({ prompt, options }: Props) => (
  <RadioGroup.Root>
    {prompt}

    <VStack>
      {options.map((opt) => (
        <RadioGroup.Item key={opt} w="100%" p="10px" value={opt}>
          <RadioGroup.ItemHiddenInput />
          <RadioGroup.ItemText> {opt} </RadioGroup.ItemText>

          <RadioGroup.ItemIndicator />
        </RadioGroup.Item>
      ))}
    </VStack>
  </RadioGroup.Root>
);
