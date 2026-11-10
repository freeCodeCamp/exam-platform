CREATE TABLE aapi.sessions (
  id_hash bytea PRIMARY KEY,
  candidate_id uuid NOT NULL REFERENCES aapi.candidates (id) ON DELETE CASCADE,
  expires_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX sessions_candidate_id_idx ON aapi.sessions (candidate_id);
CREATE INDEX sessions_expires_at_idx ON aapi.sessions (expires_at);

GRANT SELECT, INSERT, DELETE ON aapi.sessions TO aapi;
