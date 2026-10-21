CREATE TABLE users (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  email text NOT NULL,
  name text NOT NULL,
  -- Google account `sub`. NULL for users created through dev login.
  google_id text UNIQUE,
  picture text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

-- Emails match case-insensitively: query with `lower(email) = lower($1)`.
CREATE UNIQUE INDEX users_email_key ON users (lower(email));

CREATE TABLE sessions (
  -- SHA-256 of the `sid` cookie value, so a leaked table holds no usable session ids.
  id_hash bytea PRIMARY KEY,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  expires_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX sessions_user_id_idx ON sessions (user_id);
CREATE INDEX sessions_expires_at_idx ON sessions (expires_at);
