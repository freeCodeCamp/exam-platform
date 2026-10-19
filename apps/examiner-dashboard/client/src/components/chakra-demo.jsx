import { Button, HStack } from "@chakra-ui/react";
import { Provider } from "@/components/ui/provider";

export const Demo = () => {
  return (
    <Provider>
      <HStack>
        <Button>Click me</Button>
        <Button>Click me</Button>
      </HStack>
    </Provider>
  );
};
