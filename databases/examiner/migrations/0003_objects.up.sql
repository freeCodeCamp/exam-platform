-- Files (images, audio) examiners upload for questions. The bytes live in the curriculum bucket at
-- `object_key`, content-addressed: `sha256/<sha256>.<ext>`. An upload is not curriculum content:
-- its row is to reach cDb's `objects` in the same Dolt commit as the content that first references
-- it. Not owned: any examiner may use, rename or delete any upload. Deleting one that a cDb
-- commit references breaks that commit's content: eDd asks the examiner to confirm, never checks.
CREATE TABLE edd.objects (
  sha256 text PRIMARY KEY,
  object_key text NOT NULL UNIQUE,
  -- Sniffed from the bytes by eDd, never taken from the uploader.
  content_type text NOT NULL,
  byte_size integer NOT NULL,
  -- As first uploaded. For examiners to recognise the file; never part of the key. At most cDb's
  -- `objects.file_name` (`VARCHAR(255)`).
  file_name text NOT NULL,
  -- First uploader. No cascade: deleting an examiner who uploaded must not lose the record.
  uploaded_by uuid NOT NULL REFERENCES edd.users (id),
  created_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT objects_sha256 CHECK (sha256 ~ '^[0-9a-f]{64}$'),
  CONSTRAINT objects_object_key CHECK (object_key LIKE 'sha256/' || sha256 || '.%'),
  CONSTRAINT objects_content_type CHECK (content_type <> ''),
  CONSTRAINT objects_byte_size CHECK (byte_size > 0),
  CONSTRAINT objects_file_name CHECK (char_length(file_name) BETWEEN 1 AND 255)
);

CREATE INDEX objects_created_at_idx ON edd.objects (created_at DESC);
CREATE INDEX objects_uploaded_by_idx ON edd.objects (uploaded_by);

GRANT SELECT, INSERT, DELETE ON edd.objects TO edd;
-- Everything else derives from the bytes.
GRANT UPDATE (file_name) ON edd.objects TO edd;
