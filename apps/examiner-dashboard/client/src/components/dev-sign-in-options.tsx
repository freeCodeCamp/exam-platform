import { Box, Button, Field, Heading, Input, Stack, Text } from "@chakra-ui/react";
import { useMutation } from "@tanstack/react-query";
import { useState, type SubmitEventHandler } from "react";

import { postDevLogin } from "@/utils/fetch";

// Logs in as any @freecodecamp.org user without Google. Dev builds only.
export function DevSignInOptions() {
  const [name, setName] = useState("Camperbot");
  const [email, setEmail] = useState("camperbot@freecodecamp.org");

  const devLogin = useMutation({
    mutationFn: postDevLogin,
    retry: false,
    onSuccess: () => window.location.assign("/"),
  });

  const onSubmit: SubmitEventHandler<HTMLFormElement> = (e) => {
    e.preventDefault();
    devLogin.mutate({ name, email });
  };

  return (
    <form onSubmit={onSubmit}>
      <Stack gap="3" pt="6" borderTopWidth="1px">
        <Box>
          <Heading as="h2" size="sm">
            Dev login
          </Heading>
          <Text textStyle="xs" color="fg.muted">
            Requires the eDd server to run with MOCK_AUTH=true.
          </Text>
        </Box>
        <Field.Root required>
          <Field.Label>Name</Field.Label>
          <Input value={name} onChange={(e) => setName(e.target.value)} />
        </Field.Root>
        <Field.Root required invalid={devLogin.isError}>
          <Field.Label>Email</Field.Label>
          <Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} />
          <Field.ErrorText>{devLogin.error?.message}</Field.ErrorText>
        </Field.Root>
        <Button type="submit" variant="subtle" loading={devLogin.isPending}>
          Log in as dev user
        </Button>
      </Stack>
    </form>
  );
}
