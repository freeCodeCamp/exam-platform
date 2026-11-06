// Upload policy for question objects. Mirrors server/routes/objects.rs (`KINDS` and the limits), so
// the dropzone rejects what the server would. The server sniffs the bytes and rejects anything else.

export const MAX_FILE_BYTES = 10 * 1024 * 1024;
export const MAX_FILES_PER_REQUEST = 5;

interface ObjectKind {
  name: string;
  // Browsers report some types under other names (`audio/x-m4a`, `audio/x-wav`), or none at all, so
  // extensions match too.
  contentTypes: string[];
  extensions: string[];
}

export const OBJECT_KINDS: ObjectKind[] = [
  { name: "PNG", contentTypes: ["image/png"], extensions: [".png"] },
  { name: "JPEG", contentTypes: ["image/jpeg"], extensions: [".jpg", ".jpeg"] },
  { name: "GIF", contentTypes: ["image/gif"], extensions: [".gif"] },
  { name: "WebP", contentTypes: ["image/webp"], extensions: [".webp"] },
  { name: "MP3", contentTypes: ["audio/mpeg", "audio/mp3"], extensions: [".mp3"] },
  {
    name: "WAV",
    contentTypes: ["audio/wav", "audio/x-wav", "audio/wave", "audio/vnd.wave"],
    extensions: [".wav"],
  },
  { name: "Ogg", contentTypes: ["audio/ogg"], extensions: [".ogg", ".oga"] },
  { name: "FLAC", contentTypes: ["audio/flac", "audio/x-flac"], extensions: [".flac"] },
  { name: "M4A", contentTypes: ["audio/mp4", "audio/x-m4a"], extensions: [".m4a"] },
];

// For the file picker and dropzone: a file matching any entry is accepted.
export const ACCEPT = OBJECT_KINDS.flatMap((kind) => [...kind.contentTypes, ...kind.extensions]);

// "PNG, JPEG, ... or M4A".
export const KIND_NAMES = new Intl.ListFormat("en", { type: "disjunction" }).format(
  OBJECT_KINDS.map((kind) => kind.name),
);
