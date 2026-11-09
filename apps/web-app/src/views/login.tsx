import { Alert, Center, Heading, HStack, Spinner, Stack, Text } from "@chakra-ui/react";
import { useEffect } from "react";
import { LuGraduationCap } from "react-icons/lu";

import { AppRoot } from "@/components/app-root";
import { useAuth } from "@/contexts/auth";
import { redirect } from "@/utils/navigate";
import { ROUTES } from "@/utils/routes";

function LoginCard() {
  const { user, isLoading, error } = useAuth();

  useEffect(() => {
    if (user) {
      redirect(ROUTES.home);
    }
  }, [user]);

  return (
    <Center minH="100dvh" p="4" bg="bg.muted" colorPalette="teal">
      <Stack gap="6" w="full" maxW="sm" p="8" bg="bg" borderWidth="1px" shadow="sm">
        <HStack gap="3">
          <Center boxSize="9" bg="colorPalette.solid" color="colorPalette.contrast">
            <LuGraduationCap />
          </Center>
          <Text fontWeight="bold">Exam Platform</Text>
        </HStack>
        {error && (
          <Alert.Root status="warning">
            <Alert.Indicator />
            <Alert.Content>
              <Alert.Title>Could not check your session</Alert.Title>
              <Alert.Description>{error.message}</Alert.Description>
            </Alert.Content>
          </Alert.Root>
        )}
        {isLoading || user ? (
          <HStack gap="3" color="fg.muted">
            <Spinner color="colorPalette.solid" />
            <Text>{user ? "Redirecting..." : "Checking session..."}</Text>
          </HStack>
        ) : (
          <Heading as="h1" size="xl">
            Log in
          </Heading>
        )}
      </Stack>
    </Center>
  );
}

export function Login() {
  return (
    <AppRoot>
      <LoginCard />
    </AppRoot>
  );
}
