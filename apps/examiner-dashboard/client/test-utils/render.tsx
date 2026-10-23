import { ChakraUiProvider } from "@/components/ui/chakra-ui-provider";
import { render as rtlRender } from "@testing-library/react";

export function render(ui: React.ReactNode) {
  return rtlRender(<>{ui}</>, {
    wrapper: (props: React.PropsWithChildren) => (
      <ChakraUiProvider>{props.children}</ChakraUiProvider>
    ),
  });
}
