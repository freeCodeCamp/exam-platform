import {
  Alert,
  Box,
  Button,
  Center,
  Field,
  Flex,
  Heading,
  HStack,
  Spinner,
  Stack,
  Text,
} from "@chakra-ui/react";
import { useMutation } from "@tanstack/react-query";
import { useEffect, useState, type SubmitEventHandler } from "react";
import { LuLogIn, LuMail, LuUserRound } from "react-icons/lu";

import { AppRoot } from "@/components/app-root";
import { IconField } from "@/components/icon-field";
import { SiteHeader } from "@/components/site-header";
import { useAuth } from "@/contexts/auth";
import { postDevLogin } from "@/utils/fetch";
import { redirect } from "@/utils/navigate";
import { ROUTES } from "@/utils/routes";

function LoginForm() {
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");

  const login = useMutation({
    mutationFn: postDevLogin,
    retry: false,
    onSuccess: () => redirect(ROUTES.home),
  });

  const onSubmit: SubmitEventHandler<HTMLFormElement> = (e) => {
    e.preventDefault();
    login.mutate({ name, email });
  };

  return (
    <Stack asChild gap="4">
      <form onSubmit={onSubmit}>
        <Field.Root required>
          <IconField
            id="name"
            label="Name"
            icon={LuUserRound}
            autoComplete="name"
            placeholder="Name"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </Field.Root>
        <Field.Root required invalid={login.isError}>
          <IconField
            id="email"
            label="Email"
            icon={LuMail}
            type="email"
            autoComplete="email"
            placeholder="you@example.com"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
          />
          <Field.ErrorText>{login.error?.message}</Field.ErrorText>
        </Field.Root>
        <Button
          type="submit"
          colorPalette="brand"
          h="52px"
          mt="2"
          fontWeight="bold"
          fontSize="17px"
          loading={login.isPending}
        >
          <LuLogIn aria-hidden />
          Sign in
        </Button>
      </form>
    </Stack>
  );
}

function LoginCard() {
  const { user, isLoading, error } = useAuth();

  useEffect(() => {
    if (user) {
      redirect(ROUTES.home);
    }
  }, [user]);

  return (
    <Flex direction="column" minH="100dvh" bg="bg" color="fg">
      <SiteHeader />
      <Center as="main" flex="1" px="5" py="12">
        <Stack
          gap="4"
          w="full"
          maxW="400px"
          px="8"
          py="9"
          bg="bg.panel"
          borderWidth="1px"
          rounded="l3"
        >
          <HStack gap="3" mb="2">
            <Center boxSize="11" rounded="full" bg="bg.muted" color="brand.fg">
              <LuUserRound size="22" aria-hidden />
            </Center>
            <Heading as="h1" fontSize="26px">
              Sign in
            </Heading>
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
              <Spinner color="brand.solid" />
              <Text>{user ? "Redirecting..." : "Checking session..."}</Text>
            </HStack>
          ) : (
            <Box>
              <LoginForm />
            </Box>
          )}
        </Stack>
      </Center>
    </Flex>
  );
}

export function Login() {
  return (
    <AppRoot>
      <LoginCard />
    </AppRoot>
  );
}
