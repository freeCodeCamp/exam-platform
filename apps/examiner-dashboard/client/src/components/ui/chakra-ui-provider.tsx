import { ChakraProvider, createSystem, defaultConfig, defineConfig } from "@chakra-ui/react";
import { ColorModeProvider, type ColorModeProviderProps } from "./color-mode";

// Square corners on all components. Components use the l1-l3 radii.
const config = defineConfig({
  theme: {
    semanticTokens: {
      radii: {
        l1: { value: "{radii.none}" },
        l2: { value: "{radii.none}" },
        l3: { value: "{radii.none}" },
      },
    },
  },
});

const system = createSystem(defaultConfig, config);

export function ChakraUiProvider(props: ColorModeProviderProps) {
  return (
    <ChakraProvider value={system}>
      <ColorModeProvider {...props} />
    </ChakraProvider>
  );
}
