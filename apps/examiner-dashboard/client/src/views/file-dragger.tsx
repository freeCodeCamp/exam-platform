import {
  Alert,
  Box,
  Button,
  Card,
  Center,
  Clipboard,
  FileUpload,
  Heading,
  HStack,
  IconButton,
  Image,
  SimpleGrid,
  Spinner,
  Text,
  useFileUpload,
  VStack,
} from "@chakra-ui/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { LuFile, LuUpload } from "react-icons/lu";

import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { getObjects, type RecordedObject, uploadObjects } from "@/utils/fetch";
import {
  ACCEPT,
  formatBytes,
  KIND_NAMES,
  MAX_FILE_BYTES,
  MAX_FILES_PER_REQUEST,
} from "@/utils/objects";

const OBJECTS_QUERY_KEY = ["objects"];
// Presigned URLs last 15 minutes: refetch before previews break. In a background tab, the interval
// pauses and focus refetches once the list is stale.
const OBJECTS_REFRESH_MS = 10 * 60 * 1000;

function Uploader() {
  const queryClient = useQueryClient();
  const fileUpload = useFileUpload({
    maxFiles: MAX_FILES_PER_REQUEST,
    maxFileSize: MAX_FILE_BYTES,
    accept: ACCEPT,
  });
  const upload = useMutation({
    mutationFn: uploadObjects,
    onSuccess: () => {
      fileUpload.clearFiles();
    },
    // After a failure too: files before the failed one were stored.
    onSettled: async () => {
      await queryClient.invalidateQueries({ queryKey: OBJECTS_QUERY_KEY });
    },
  });

  const pending = fileUpload.acceptedFiles;

  return (
    <VStack align="stretch" gap="3">
      <FileUpload.RootProvider value={fileUpload} alignItems="stretch">
        <FileUpload.HiddenInput />
        <FileUpload.Dropzone w="full" minH="40">
          <Box color="colorPalette.fg">
            <LuUpload size="28" />
          </Box>
          <FileUpload.DropzoneContent>
            <Text fontWeight="medium">Drag files here, or click to choose</Text>
            <Text color="fg.muted" textStyle="sm">
              {KIND_NAMES}. Up to {MAX_FILES_PER_REQUEST} files, {formatBytes(MAX_FILE_BYTES)} each.
            </Text>
          </FileUpload.DropzoneContent>
        </FileUpload.Dropzone>
        <FileUpload.List showSize clearable />
      </FileUpload.RootProvider>

      {fileUpload.rejectedFiles.map(({ file, errors }) => (
        <Text key={file.name} color="fg.error" textStyle="sm">
          {file.name}: {errors.join(", ").toLowerCase().replaceAll("_", " ")}
        </Text>
      ))}

      {upload.error && (
        <Alert.Root status="error">
          <Alert.Indicator />
          <Alert.Title>Upload failed: {upload.error.message}</Alert.Title>
        </Alert.Root>
      )}

      <HStack>
        <Button
          disabled={pending.length === 0}
          loading={upload.isPending}
          loadingText="Uploading..."
          onClick={() => upload.mutate(pending)}
        >
          <LuUpload />
          Upload {pending.length === 1 ? "1 file" : `${pending.length} files`}
        </Button>
        <Button
          variant="ghost"
          disabled={pending.length === 0 || upload.isPending}
          onClick={() => {
            fileUpload.clearFiles();
            fileUpload.clearRejectedFiles();
          }}
        >
          Clear
        </Button>
      </HStack>
    </VStack>
  );
}

function Preview({ file }: { file: RecordedObject }) {
  if (file.contentType.startsWith("image/")) {
    return <Image src={file.url} alt={file.fileName} maxH="full" maxW="full" fit="contain" />;
  }
  if (file.contentType.startsWith("audio/")) {
    return <audio controls src={file.url} aria-label={file.fileName} style={{ width: "90%" }} />;
  }
  return (
    <Box color="fg.muted">
      <LuFile size="32" />
    </Box>
  );
}

function ObjectCard({ file }: { file: RecordedObject }) {
  return (
    <Card.Root size="sm" overflow="hidden">
      <Center h="40" bg="bg.muted">
        <Preview file={file} />
      </Center>
      <Card.Body gap="1">
        <Text textStyle="sm" fontWeight="medium" truncate title={file.fileName}>
          {file.fileName}
        </Text>
        <HStack justify="space-between" gap="2">
          <Text fontFamily="mono" textStyle="xs" truncate title={file.sha256}>
            {file.sha256}
          </Text>
          <Clipboard.Root value={file.sha256}>
            <Clipboard.Trigger asChild>
              <IconButton size="xs" variant="ghost" aria-label={`Copy hash of ${file.fileName}`}>
                <Clipboard.Indicator />
              </IconButton>
            </Clipboard.Trigger>
          </Clipboard.Root>
        </HStack>
        <Text textStyle="xs" color="fg.muted">
          {formatBytes(file.byteSize)} · {new Date(file.createdAt).toLocaleString("en-GB")}
        </Text>
      </Card.Body>
    </Card.Root>
  );
}

function UploadedObjects() {
  const files = useQuery({
    queryKey: OBJECTS_QUERY_KEY,
    queryFn: getObjects,
    staleTime: OBJECTS_REFRESH_MS,
    refetchInterval: OBJECTS_REFRESH_MS,
  });

  return (
    <VStack align="stretch" gap="3">
      <Heading as="h2" size="md">
        Uploaded {files.data && `(${files.data.length})`}
      </Heading>
      {files.isPending && (
        <HStack color="fg.muted">
          <Spinner size="sm" />
          <Text>Loading files...</Text>
        </HStack>
      )}
      {files.error && (
        <Alert.Root status="error">
          <Alert.Indicator />
          <Alert.Title>Could not list files: {files.error.message}</Alert.Title>
        </Alert.Root>
      )}
      {files.data?.length === 0 && <Text color="fg.muted">No files yet.</Text>}
      <SimpleGrid columns={{ base: 1, md: 2, xl: 4 }} gap="4">
        {files.data?.map((file) => (
          <ObjectCard key={file.sha256} file={file} />
        ))}
      </SimpleGrid>
    </VStack>
  );
}

export function FileDragger() {
  return (
    <AppRoot>
      <ProtectedRoute>
        <PageShell
          title="File Dragger"
          description="Upload images and audio for questions to reference by hash."
          path="/file-dragger"
        >
          <VStack align="stretch" gap="8" p="6">
            <Uploader />
            <UploadedObjects />
          </VStack>
        </PageShell>
      </ProtectedRoute>
    </AppRoot>
  );
}
