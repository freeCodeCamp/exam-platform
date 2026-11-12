import { QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { type ReactNode } from "react";
import { ChakraUiProvider } from "@/components/ui/chakra-ui-provider";
import { AuthProvider } from "@/contexts/auth";
import { queryClient } from "@/utils/query-client.js";

// Each page is its own React island, so every view wraps itself in this.
export function AppRoot({ children }: { children: ReactNode }) {
  return (
    <ChakraUiProvider>
      <QueryClientProvider client={queryClient}>
        <AuthProvider>{children}</AuthProvider>
        <ReactQueryDevtools />
      </QueryClientProvider>
    </ChakraUiProvider>
  );
}
