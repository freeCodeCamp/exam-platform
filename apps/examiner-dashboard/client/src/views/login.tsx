import {
  Alert,
  Button,
  Center,
  CloseButton,
  Heading,
  HStack,
  Spinner,
  Stack,
  Text,
} from "@chakra-ui/react";
import { useEffect, useState } from "react";
import { FcGoogle } from "react-icons/fc";
import { LuGraduationCap } from "react-icons/lu";

import { AppRoot } from "@/components/app-root";
import { DevSignInOptions } from "@/components/dev-sign-in-options";
import { useAuth } from "@/contexts/auth";
import { GOOGLE_LOGIN_PATH } from "@/utils/fetch";
import { redirect } from "@/utils/navigate";

// Codes the server's Google callback puts in `/login?error=`.
const loginErrors = {
  access_denied: "Google sign-in was cancelled.",
  invalid_state: "Login expired or was started in another browser. Try again.",
  unverified_email: "Your Google account has no verified email address.",
  forbidden_domain: "Only freeCodeCamp Google Workspace accounts may log in.",
  login_failed: "Login failed. Try again.",
} as const;

function isLoginErrorCode(code: string): code is keyof typeof loginErrors {
  return Object.hasOwn(loginErrors, code);
}

// Unknown codes get the generic message, so a crafted link cannot show arbitrary text.
export function loginErrorMessage(code: string): string {
  return loginErrors[isLoginErrorCode(code) ? code : "login_failed"];
}

function LoginCard() {
  const { user, isLoading, error: sessionError } = useAuth();
  const [error, setError] = useState<string | null>(null);

  // After hydration: the page is prerendered, so there is no URL at build time.
  useEffect(() => {
    const url = new URL(window.location.href);
    const code = url.searchParams.get("error");
    if (code === null) return;

    // oxlint-disable-next-line react/set-state-in-effect -- URL is external, unknown until hydration.
    setError(loginErrorMessage(code));
    // So a reload does not show it again.
    url.searchParams.delete("error");
    window.history.replaceState(null, "", url);
  }, []);

  useEffect(() => {
    if (user) {
      redirect("/");
    }
  }, [user]);

  return (
    <Center minH="100dvh" p="4" bg="bg.muted" colorPalette="teal">
      <Stack gap="6" w="full" maxW="sm" p="8" bg="bg" borderWidth="1px" shadow="sm">
        <HStack gap="3">
          <Center boxSize="9" bg="colorPalette.solid" color="colorPalette.contrast">
            <LuGraduationCap />
          </Center>
          <Text fontWeight="bold">Examiner Dashboard</Text>
        </HStack>
        {error && (
          <Alert.Root status="error">
            <Alert.Indicator />
            <Alert.Content>
              <Alert.Title>{error}</Alert.Title>
            </Alert.Content>
            <CloseButton size="sm" aria-label="Dismiss" onClick={() => setError(null)} />
          </Alert.Root>
        )}
        {sessionError && (
          <Alert.Root status="warning">
            <Alert.Indicator />
            <Alert.Content>
              <Alert.Title>Could not check your session</Alert.Title>
              <Alert.Description>{sessionError.message}</Alert.Description>
            </Alert.Content>
          </Alert.Root>
        )}
        {isLoading || user ? (
          <HStack gap="3" color="fg.muted">
            <Spinner color="colorPalette.solid" />
            <Text>{user ? "Redirecting..." : "Checking session..."}</Text>
          </HStack>
        ) : (
          <>
            <Stack gap="1">
              <Heading as="h1" size="xl">
                Log in
              </Heading>
              <Text textStyle="sm" color="fg.muted">
                Use your @freecodecamp.org Google account.
              </Text>
            </Stack>
            <Button asChild size="lg" variant="outline">
              <a href={GOOGLE_LOGIN_PATH}>
                <FcGoogle />
                Continue with Google
              </a>
            </Button>
            {import.meta.env.DEV && <DevSignInOptions />}
          </>
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
