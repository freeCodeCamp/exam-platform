import {
  Alert,
  Badge,
  Box,
  Button,
  Card,
  Center,
  Dialog,
  HStack,
  IconButton,
  Image,
  Input,
  List,
  Portal,
  SimpleGrid,
  Spinner,
  Text,
  VStack,
} from "@chakra-ui/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { LuFile, LuPencil, LuTrash2 } from "react-icons/lu";

import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import {
  deleteObject,
  getStoreEntries,
  renameObject,
  type StoreEntry,
  type StoreEntryStatus,
} from "@/utils/fetch";
import { formatBytes } from "@/utils/objects";

const STORE_QUERY_KEY = ["object-store"];
// File Dragger lists eDb's objects: renames and deletes change it too.
const OBJECTS_QUERY_KEY = ["objects"];
// Presigned URLs last 15 minutes: refetch before previews break.
const STORE_REFRESH_MS = 10 * 60 * 1000;

const STATUS_BADGES: Record<StoreEntryStatus, { label: string; palette: string } | null> = {
  recorded: null,
  unrecorded: { label: "Unrecorded", palette: "orange" },
  missing: { label: "Bytes missing", palette: "red" },
};

function useInvalidateObjects() {
  const queryClient = useQueryClient();
  return async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: STORE_QUERY_KEY }),
      queryClient.invalidateQueries({ queryKey: OBJECTS_QUERY_KEY }),
    ]);
  };
}

function displayName(entry: StoreEntry): string {
  return entry.record?.fileName ?? entry.key;
}

function Preview({ entry }: { entry: StoreEntry }) {
  const { url } = entry;
  const contentType = entry.record?.contentType;
  if (url && contentType?.startsWith("image/")) {
    return <Image src={url} alt={displayName(entry)} maxH="full" maxW="full" fit="contain" />;
  }
  if (url && contentType?.startsWith("audio/")) {
    return <audio controls src={url} aria-label={displayName(entry)} style={{ width: "90%" }} />;
  }
  return (
    <VStack color="fg.muted" gap="1">
      <LuFile size="32" />
      {url && (
        <Text asChild textStyle="xs">
          <a href={url} target="_blank" rel="noreferrer">
            Open
          </a>
        </Text>
      )}
    </VStack>
  );
}

function FileName({ entry }: { entry: StoreEntry }) {
  const invalidate = useInvalidateObjects();
  const [draft, setDraft] = useState<string | null>(null);
  const rename = useMutation({
    mutationFn: (fileName: string) => renameObject(entry.key, fileName),
    onSuccess: () => setDraft(null),
    onSettled: invalidate,
  });

  if (draft === null) {
    return (
      <HStack justify="space-between" gap="2">
        <Text textStyle="sm" fontWeight="medium" truncate title={displayName(entry)}>
          {entry.record?.fileName ?? "No record"}
        </Text>
        {entry.status !== "unrecorded" && (
          <IconButton
            size="xs"
            variant="ghost"
            aria-label={`Rename ${displayName(entry)}`}
            onClick={() => setDraft(entry.record?.fileName ?? "")}
          >
            <LuPencil />
          </IconButton>
        )}
      </HStack>
    );
  }

  return (
    <VStack
      as="form"
      align="stretch"
      gap="2"
      onSubmit={(event) => {
        event.preventDefault();
        rename.mutate(draft);
      }}
    >
      <Input
        size="sm"
        aria-label="File name"
        value={draft}
        maxLength={255}
        autoFocus
        onChange={(event) => setDraft(event.target.value)}
      />
      {rename.error && (
        <Text color="fg.error" textStyle="xs">
          Rename failed: {rename.error.message}
        </Text>
      )}
      <HStack>
        <Button size="xs" type="submit" disabled={draft.trim() === ""} loading={rename.isPending}>
          Save
        </Button>
        <Button
          size="xs"
          variant="ghost"
          disabled={rename.isPending}
          onClick={() => {
            rename.reset();
            setDraft(null);
          }}
        >
          Cancel
        </Button>
      </HStack>
    </VStack>
  );
}

function DeleteButton({ entry }: { entry: StoreEntry }) {
  const invalidate = useInvalidateObjects();
  const [open, setOpen] = useState(false);
  const remove = useMutation({
    mutationFn: () => deleteObject(entry.key),
    onSuccess: () => setOpen(false),
    onSettled: invalidate,
  });
  const name = displayName(entry);

  return (
    <Dialog.Root
      role="alertdialog"
      open={open}
      onOpenChange={(details) => {
        if (!remove.isPending) {
          remove.reset();
          setOpen(details.open);
        }
      }}
    >
      <Dialog.Trigger asChild>
        <IconButton size="xs" variant="ghost" colorPalette="red" aria-label={`Delete ${name}`}>
          <LuTrash2 />
        </IconButton>
      </Dialog.Trigger>
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content>
            <Dialog.Header>
              <Dialog.Title>Delete {name}?</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <List.Root gap="2" textStyle="sm">
                <List.Item>
                  Permanently deletes {entry.status === "missing" ? "the record" : "the file"}{" "}
                  <Text as="span" fontFamily="mono" textStyle="xs" wordBreak="break-all">
                    {entry.key}
                  </Text>
                  {entry.status === "recorded" && " and its record"}. This cannot be undone.
                </List.Item>
                <List.Item>
                  Any committed content that references it keeps the reference but loses the file:
                  candidates and examiners served that content get a broken image or audio. Nothing
                  checks for references first.
                </List.Item>
                <List.Item>
                  Uploading the same file again restores it under the same key and hash, which
                  repairs those references.
                </List.Item>
              </List.Root>
              {remove.error && (
                <Alert.Root status="error" mt="4">
                  <Alert.Indicator />
                  <Alert.Title>Delete failed: {remove.error.message}</Alert.Title>
                </Alert.Root>
              )}
            </Dialog.Body>
            <Dialog.Footer>
              <Dialog.ActionTrigger asChild>
                <Button variant="outline" disabled={remove.isPending}>
                  Cancel
                </Button>
              </Dialog.ActionTrigger>
              <Button
                colorPalette="red"
                loading={remove.isPending}
                loadingText="Deleting..."
                onClick={() => remove.mutate()}
              >
                Delete permanently
              </Button>
            </Dialog.Footer>
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  );
}

function EntryCard({ entry }: { entry: StoreEntry }) {
  const badge = STATUS_BADGES[entry.status];
  const at = entry.lastModified ?? entry.record?.createdAt;

  return (
    <Card.Root size="sm" overflow="hidden">
      <Center h="40" bg="bg.muted">
        <Preview entry={entry} />
      </Center>
      <Card.Body gap="1">
        <FileName entry={entry} />
        <Text fontFamily="mono" textStyle="xs" truncate title={entry.key}>
          {entry.key}
        </Text>
        <HStack justify="space-between" gap="2">
          <Text textStyle="xs" color="fg.muted">
            {formatBytes(entry.byteSize)}
            {at && ` · ${new Date(at).toLocaleString("en-GB")}`}
          </Text>
          <HStack gap="1">
            {badge && (
              <Badge size="sm" colorPalette={badge.palette}>
                {badge.label}
              </Badge>
            )}
            <DeleteButton entry={entry} />
          </HStack>
        </HStack>
      </Card.Body>
    </Card.Root>
  );
}

function StoreEntries() {
  const entries = useQuery({
    queryKey: STORE_QUERY_KEY,
    queryFn: getStoreEntries,
    staleTime: STORE_REFRESH_MS,
    refetchInterval: STORE_REFRESH_MS,
  });

  return (
    <VStack align="stretch" gap="3">
      {entries.isPending && (
        <HStack color="fg.muted">
          <Spinner size="sm" />
          <Text>Loading objects...</Text>
        </HStack>
      )}
      {entries.error && (
        <Alert.Root status="error">
          <Alert.Indicator />
          <Alert.Title>Could not list objects: {entries.error.message}</Alert.Title>
        </Alert.Root>
      )}
      {entries.data && (
        <Text color="fg.muted" textStyle="sm">
          {entries.data.length === 1 ? "1 object" : `${entries.data.length} objects`}
        </Text>
      )}
      {entries.data?.length === 0 && <Text color="fg.muted">The bucket is empty.</Text>}
      <SimpleGrid columns={{ base: 1, md: 2, xl: 4 }} gap="4">
        {entries.data?.map((entry) => (
          <EntryCard key={entry.key} entry={entry} />
        ))}
      </SimpleGrid>
    </VStack>
  );
}

export function ObjectStore() {
  return (
    <AppRoot>
      <ProtectedRoute>
        <PageShell
          title="Object Store"
          description="Every object in the curriculum bucket, with its upload record. Rename or delete them."
          path="/object-store"
        >
          <Box p="6">
            <StoreEntries />
          </Box>
        </PageShell>
      </ProtectedRoute>
    </AppRoot>
  );
}
