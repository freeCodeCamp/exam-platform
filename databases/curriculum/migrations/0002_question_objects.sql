-- Files (images, audio) questions use. The bytes live in object storage, in the curriculum bucket,
-- content-addressed: `object_key` is `sha256/<sha256>.<ext>`, never a URL, so the provider, bucket
-- host or CDN can change without rewriting rows. Same bytes, same key: objects are never
-- overwritten with different content, and never deleted while any commit still references them.
CREATE TABLE objects (
  sha256 CHAR(64) PRIMARY KEY,
  object_key VARCHAR(255) NOT NULL UNIQUE,
  -- Sniffed from the bytes by eDd, never taken from the uploader.
  content_type VARCHAR(127) NOT NULL,
  byte_size INT NOT NULL,
  -- As first uploaded. For examiners to recognise the file; never part of the key.
  file_name VARCHAR(255) NOT NULL,
  created_at DATETIME(6) NOT NULL,
  CONSTRAINT object_sha256_length CHECK (CHAR_LENGTH(sha256) = 64),
  CONSTRAINT object_object_key CHECK (object_key LIKE 'sha256/%'),
  CONSTRAINT object_content_type CHECK (content_type <> ''),
  CONSTRAINT object_byte_size CHECK (byte_size > 0)
);

-- The objects a question shows, in order. Versioned with the question, so a snapshot names exactly
-- the objects it was served with.
CREATE TABLE question_objects (
  question_id VARCHAR(36) NOT NULL,
  position INT NOT NULL,
  object_sha256 CHAR(64) NOT NULL,
  -- Images: alt text. Audio: transcript, where the question allows one. Empty when none.
  alt_text TEXT NOT NULL,
  PRIMARY KEY (question_id, position),
  CONSTRAINT question_object_question FOREIGN KEY (question_id) REFERENCES questions (id),
  CONSTRAINT question_object_object FOREIGN KEY (object_sha256) REFERENCES objects (sha256),
  CONSTRAINT question_object_position CHECK (position >= 0)
);
