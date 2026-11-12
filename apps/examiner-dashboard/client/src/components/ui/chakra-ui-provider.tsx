import { ChakraProvider, createSystem, defaultConfig, defineConfig } from "@chakra-ui/react";
import createCache from "@emotion/cache";
import { CacheProvider } from "@emotion/react";
import { useState } from "react";
import { ColorModeProvider, type ColorModeProviderProps } from "./color-mode.js";

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
  const [cache] = useState(() => createCache({ key: "chakra" }));

  return (
    <CacheProvider value={cache}>
      <ChakraProvider value={system}>
        <ColorModeProvider {...props} />
      </ChakraProvider>
    </CacheProvider>
  );
}
