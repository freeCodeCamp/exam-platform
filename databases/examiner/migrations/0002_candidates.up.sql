-- aAPI's candidates. eDd reads all candidate data (docs/auth.md), and writes none.
GRANT USAGE ON SCHEMA aapi TO aapi, edd;

CREATE TABLE aapi.candidates (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  email text NOT NULL,
  name text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

-- Emails match case-insensitively: query with `lower(email) = lower($1)`.
CREATE UNIQUE INDEX candidates_email_key ON aapi.candidates (lower(email));

GRANT SELECT, INSERT, UPDATE, DELETE ON aapi.candidates TO aapi;
GRANT SELECT ON aapi.candidates TO edd;
