import { QueryClient } from "@tanstack/react-query";

// This needs to be a singleton so that all pages share a cache.
export const queryClient = new QueryClient();
