import { VStack, RadioGroup } from "@chakra-ui/react";

interface Props {
  prompt: string;
  options: { value: string; id: string }[];
}

export const MultipleChoice = ({ prompt, options }: Props) => {
  return (
    <RadioGroup.Root>
      {prompt}

      <VStack>
        {options.map((opt) => (
          <RadioGroup.Item key={opt.id} w="100%" p="10px" value={opt.value}>
            <RadioGroup.ItemHiddenInput />
            <RadioGroup.ItemText> {opt.value} </RadioGroup.ItemText>

            <RadioGroup.ItemIndicator />
          </RadioGroup.Item>
        ))}
      </VStack>
    </RadioGroup.Root>
  );
};
