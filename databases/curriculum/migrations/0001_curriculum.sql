CREATE TABLE exams (
  id VARCHAR(36) PRIMARY KEY,
  slug VARCHAR(128) NOT NULL UNIQUE,
  title VARCHAR(255) NOT NULL DEFAULT '',
  instructions MEDIUMTEXT NOT NULL,
  -- TODO: non_exhaustive enum on SDK
  -- disabled: new exam attempts cannot be taken
  -- enabled: exam attempts are startable
  -- archived: exam is not expected to ever be taken again, but content needs to be kept around for provenance
  status ENUM('disabled', 'enabled', 'archived')
);

-- Split from `exams` so adding an option changes only this table's schema. Historical snapshots
-- are read under their own schema, so readers then handle extra generations of policy alone.
-- One row per exam, so one column per option. Nothing is nullable: every commit carries a
-- complete policy, and no reader substitutes defaults.
CREATE TABLE exam_policy (
  exam_id VARCHAR(36) PRIMARY KEY,
  duration_seconds INT NOT NULL,
  CONSTRAINT policy_exam FOREIGN KEY (exam_id) REFERENCES exams (id),
  CONSTRAINT policy_duration_positive CHECK (duration_seconds > 0)
);

CREATE TABLE questions (
  id VARCHAR(36) PRIMARY KEY,
  exam_id VARCHAR(36) NOT NULL,
  position INT NOT NULL,
  -- TODO: remember to use non_exhaustive enum on SDK
  kind ENUM('multiple_choice') NOT NULL,
  prompt MEDIUMTEXT NOT NULL,
  -- TODO: Enum?
  archived BOOLEAN NOT NULL DEFAULT FALSE,
  UNIQUE KEY question_order (exam_id, position),
  CONSTRAINT question_exam FOREIGN KEY (exam_id) REFERENCES exams (id),
  CONSTRAINT question_kind_valid CHECK (kind <> ''),
  CONSTRAINT question_position CHECK (position >= 0),
  CONSTRAINT question_archived_boolean CHECK (archived IN (0, 1))
);

-- Per-branch selection of the immutable commit each exam is served from. The snapshot may precede
-- the branch tip, and must stay an ancestor of it. No FK: the exam may exist only in the snapshot.
CREATE TABLE manifest (
  exam_id VARCHAR(36) PRIMARY KEY,
  snapshot_commit CHAR(32) NOT NULL,
  deprecated BOOLEAN NOT NULL DEFAULT FALSE,
  CONSTRAINT manifest_snapshot_hash CHECK (CHAR_LENGTH(snapshot_commit) = 32),
  CONSTRAINT manifest_deprecated_boolean CHECK (deprecated IN (0, 1))
);
