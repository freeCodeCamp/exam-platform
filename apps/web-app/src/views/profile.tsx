import {
  Box,
  Button,
  Center,
  Flex,
  Heading,
  HStack,
  IconButton,
  Stack,
  Text,
  VisuallyHidden,
} from "@chakra-ui/react";
import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import {
  LuCake,
  LuCheck,
  LuGlobe,
  LuIdCard,
  LuLock,
  LuLogOut,
  LuMapPin,
  LuMoon,
  LuSun,
  LuUpload,
  LuUserRound,
  LuUserX,
} from "react-icons/lu";

import { AppRoot } from "@/components/app-root";
import { EmptyState } from "@/components/empty-state";
import { IconField } from "@/components/icon-field";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { ToggleGroup } from "@/components/toggle-group";
import { useColorMode, type ColorMode } from "@/components/ui/color-mode";
import { useAuth } from "@/contexts/auth";
import { deleteSession, type SessionUser } from "@/utils/fetch";
import { initials } from "@/utils/initials";
import { redirect } from "@/utils/navigate";
import { buildPath, currentRouteParams, ROUTES } from "@/utils/routes";

type Visibility = "public" | "private";

function ProfileForm({ user }: { user: SessionUser }) {
  const { colorMode, setColorMode } = useColorMode();
  const [name, setName] = useState(user.name);
  const [age, setAge] = useState("");
  const [location, setLocation] = useState("");
  const [document, setDocument] = useState<string | null>(null);
  const [visibility, setVisibility] = useState<Visibility>("public");

  return (
    <Stack asChild gap="4" p="7" bg="bg.panel" borderWidth="1px" rounded="l3">
      <form onSubmit={(e) => e.preventDefault()}>
        <IconField
          id="name"
          label="Name"
          icon={LuUserRound}
          autoComplete="name"
          placeholder="Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <Flex wrap="wrap" gap="4">
          <Box flex="1 1 140px">
            <IconField
              id="age"
              label="Age"
              icon={LuCake}
              type="number"
              min={13}
              placeholder="Age"
              value={age}
              onChange={(e) => setAge(e.target.value)}
            />
          </Box>
          <Box flex="3 1 280px">
            <IconField
              id="location"
              label="Location"
              icon={LuMapPin}
              autoComplete="country-name"
              placeholder="Location"
              value={location}
              onChange={(e) => setLocation(e.target.value)}
            />
          </Box>
        </Flex>

        <HStack gap="3" minH="52px" py="1" pe="1" bg="bg.muted" borderWidth="1px" rounded="l2">
          <Center w="12" flexShrink={0} color="fg.muted" title="Identity document">
            <LuIdCard size="20" aria-hidden />
          </Center>
          <Text flex="1" minW="0" truncate color={document ? "fg" : "fg.muted"}>
            {document ?? "Identity document"}
          </Text>
          <Center
            asChild
            boxSize="11"
            flexShrink={0}
            rounded="l1"
            bg="bg.emphasized"
            cursor="pointer"
            title="Upload identity document"
          >
            <label htmlFor="id-doc">
              <LuUpload size="20" aria-hidden />
              <VisuallyHidden>Upload identity document</VisuallyHidden>
            </label>
          </Center>
          <VisuallyHidden asChild>
            <input
              id="id-doc"
              type="file"
              accept="image/*,.pdf"
              onChange={(e) => setDocument(e.target.files?.[0]?.name ?? null)}
            />
          </VisuallyHidden>
        </HStack>

        <Flex wrap="wrap" gap="4" pt="2">
          <ToggleGroup<ColorMode>
            legend="Theme"
            value={colorMode}
            onChange={setColorMode}
            options={[
              { value: "light", label: "Light theme", icon: LuSun },
              { value: "dark", label: "Dark theme", icon: LuMoon },
            ]}
          />
          <ToggleGroup<Visibility>
            legend="Profile"
            value={visibility}
            onChange={setVisibility}
            options={[
              { value: "public", label: "Public profile", icon: LuGlobe },
              { value: "private", label: "Private profile", icon: LuLock },
            ]}
          />
        </Flex>

        <Flex justify="flex-end" pt="2">
          <Button
            type="submit"
            colorPalette="brand"
            h="52px"
            px="7"
            fontWeight="bold"
            fontSize="17px"
            disabled
          >
            <LuCheck aria-hidden />
            Save
          </Button>
        </Flex>
      </form>
    </Stack>
  );
}

function ProfilePage() {
  const { user } = useAuth();
  const { userId } = currentRouteParams(ROUTES.profile);
  const logout = useMutation({
    mutationFn: deleteSession,
    onSuccess: () => redirect(ROUTES.login),
  });

  return (
    <PageShell path={buildPath(ROUTES.profile, { userId })} maxW="760px">
      {user && user.id === userId ? (
        <Stack gap="6">
          <Flex align="center" justify="space-between" gap="4">
            <HStack gap="4">
              <Center
                aria-hidden
                boxSize="72px"
                rounded="full"
                bg="bg.muted"
                borderWidth="1px"
                fontFamily="heading"
                fontWeight="bold"
                fontSize="2xl"
              >
                {initials(user.name)}
              </Center>
              <Heading as="h1" fontSize="30px">
                Profile
              </Heading>
            </HStack>
            <IconButton
              variant="outline"
              size="xl"
              borderWidth="2px"
              color="fg"
              aria-label="Sign out"
              title="Sign out"
              loading={logout.isPending}
              onClick={() => logout.mutate()}
            >
              <LuLogOut size="20" aria-hidden />
            </IconButton>
          </Flex>
          <ProfileForm user={user} />
        </Stack>
      ) : (
        <EmptyState icon={LuUserX} title="Profile not found" />
      )}
    </PageShell>
  );
}

export function Profile() {
  return (
    <AppRoot>
      <ProtectedRoute>
        <ProfilePage />
      </ProtectedRoute>
    </AppRoot>
  );
}
