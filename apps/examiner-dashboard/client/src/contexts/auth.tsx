import { useQuery } from "@tanstack/react-query";
import { createContext, useContext, useMemo, useSyncExternalStore, type ReactNode } from "react";

import { getSessionUser, type SessionUser } from "@/utils/fetch";

interface Auth {
  user: SessionUser | null;
  isLoading: boolean;
  error: Error | null;
}

function subscribeToHydration() {
  // noop because there are no changes to subscribe to. React will automatically call
  // getHydratedSnapshot after hydrating and does not need notifiy.
  return () => {};
}

function getHydratedSnapshot() {
  return true;
}

function getServerSnapshot() {
  return false;
}

export const AuthContext = createContext<Auth | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  // Before (on the server) and during hydration, this is false which ensures that, regardless of what
  // useQuery returns, the 'value' is always what it was during SSR.
  const hydrated = useSyncExternalStore(
    subscribeToHydration,
    getHydratedSnapshot,
    getServerSnapshot,
  );

  const { data, isPending, error } = useQuery({
    queryKey: ["session"],
    queryFn: getSessionUser,
    retry: false,
    staleTime: Infinity,
  });

  const value = useMemo(
    () => ({
      user: hydrated ? (data ?? null) : null,
      isLoading: !hydrated || isPending,
      error: hydrated ? error : null,
    }),
    [data, hydrated, isPending, error],
  );

  return <AuthContext value={value}>{children}</AuthContext>;
}

export function useAuth(): Auth {
  const auth = useContext(AuthContext);
  if (!auth) {
    throw new Error("useAuth must be used within AuthProvider");
  }
  return auth;
}
