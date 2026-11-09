import { Center, HStack, Spinner, Text } from "@chakra-ui/react";
import { useEffect, type ReactNode } from "react";

import { useAuth } from "@/contexts/auth";
import { redirect } from "@/utils/navigate";
import { ROUTES } from "@/utils/routes";

export function ProtectedRoute({ children }: { children: ReactNode }) {
  const { user, isLoading } = useAuth();
  const isLoggedOut = !isLoading && !user;

  useEffect(() => {
    if (isLoggedOut) {
      redirect(ROUTES.login);
    }
  }, [isLoggedOut]);

  if (!user) {
    return (
      <Center h="100dvh" colorPalette="teal">
        <HStack gap="3" color="fg.muted">
          <Spinner color="colorPalette.solid" />
          <Text>{isLoggedOut ? "Redirecting to login..." : "Checking session..."}</Text>
        </HStack>
      </Center>
    );
  }

  return children;
}
