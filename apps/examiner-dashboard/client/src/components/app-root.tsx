import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { useState, type ReactNode } from "react";
import { ChakraUiProvider } from "@/components/ui/chakra-ui-provider";

// Each page is its own React island, so every view wraps itself in this.
export function AppRoot({ children }: { children: ReactNode }) {
  const [queryClient] = useState(() => new QueryClient());
  return (
    <ChakraUiProvider>
      <QueryClientProvider client={queryClient}>
        {children}
        <ReactQueryDevtools />
      </QueryClientProvider>
    </ChakraUiProvider>
  );
}
