import { ChakraProvider, createSystem, defaultConfig, defineConfig } from "@chakra-ui/react";
import { ColorModeProvider, type ColorModeProviderProps } from "./color-mode";

const config = defineConfig({
  globalCss: {
    body: { bg: "bg", color: "fg", fontFamily: "body" },
    a: { color: "link" },
    "a:focus-visible, button:focus-visible, input:focus-visible, label:focus-within": {
      outline: "3px solid {colors.focus}",
      outlineOffset: "2px",
    },
  },
  theme: {
    tokens: {
      fonts: {
        heading: { value: "'Space Grotesk', Inter, system-ui, sans-serif" },
        body: { value: "Inter, system-ui, sans-serif" },
      },
    },
    semanticTokens: {
      radii: {
        l1: { value: "6px" },
        l2: { value: "8px" },
        l3: { value: "12px" },
      },
      colors: {
        bg: {
          DEFAULT: { value: { _light: "#f5f6f7", _dark: "#0a0a23" } },
          panel: { value: { _light: "#ffffff", _dark: "#1b1b32" } },
          muted: { value: { _light: "#ebebee", _dark: "#2a2a40" } },
          emphasized: { value: { _light: "#dfdfe2", _dark: "#3b3b4f" } },
        },
        fg: {
          DEFAULT: { value: { _light: "#0a0a23", _dark: "#f5f6f7" } },
          muted: { value: { _light: "#3b3b4f", _dark: "#d0d0d5" } },
        },
        border: {
          DEFAULT: { value: { _light: "#d0d0d5", _dark: "#3b3b4f" } },
        },
        link: { value: { _light: "#002ead", _dark: "#99c9ff" } },
        focus: { value: "#198eee" },
        brand: {
          solid: { value: "#feac32" },
          contrast: { value: "#0a0a23" },
          fg: { value: { _light: "#4d3800", _dark: "#feac32" } },
          muted: { value: { _light: "#ffd28a", _dark: "#4d3800" } },
          subtle: { value: { _light: "#ffe7c2", _dark: "#2a2a40" } },
          emphasized: { value: "#ffbf5e" },
          focusRing: { value: "#198eee" },
        },
        positive: {
          fg: { value: { _light: "#00471b", _dark: "#acd157" } },
          subtle: { value: { _light: "#dcefb4", _dark: "#00471b" } },
        },
        notice: {
          fg: { value: { _light: "#002ead", _dark: "#99c9ff" } },
          subtle: { value: { _light: "#dbeafe", _dark: "#002ead" } },
        },
        lilac: {
          fg: { value: { _light: "#5a01a7", _dark: "#dbb8ff" } },
        },
      },
    },
  },
});

const system = createSystem(defaultConfig, config);

export function ChakraUiProvider(props: ColorModeProviderProps) {
  return (
    <ChakraProvider value={system}>
      <ColorModeProvider defaultTheme="dark" enableSystem={false} {...props} />
    </ChakraProvider>
  );
}
