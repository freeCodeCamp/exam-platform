import { useQuery } from "@tanstack/react-query";
import { createContext, useContext, useMemo, type ReactNode } from "react";

import { getSessionUser, type SessionUser } from "@/utils/fetch";

interface Auth {
  user: SessionUser | null;
  isLoading: boolean;
  error: Error | null;
}

export const AuthContext = createContext<Auth | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const { data, isPending, error } = useQuery({
    queryKey: ["session"],
    queryFn: getSessionUser,
    retry: false,
    staleTime: Infinity,
  });

  const value = useMemo(
    () => ({ user: data ?? null, isLoading: isPending, error }),
    [data, isPending, error],
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
