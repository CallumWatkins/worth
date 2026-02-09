CREATE TABLE institutions (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE);

CREATE TABLE account_types (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE);

CREATE TABLE accounts (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  institution_id INTEGER NOT NULL REFERENCES institutions (id) ON DELETE CASCADE,
  type_id INTEGER NOT NULL REFERENCES account_types (id) ON DELETE CASCADE,
  currency_code TEXT NOT NULL,
  normal_balance_sign INTEGER NOT NULL CHECK (normal_balance_sign IN (1, -1)),
  opened_date TEXT,
  closed_date TEXT,
  created_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now')),
  UNIQUE (institution_id, name)
);

CREATE TABLE account_balance_snapshots (
  id INTEGER PRIMARY KEY,
  account_id INTEGER NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
  balance_date TEXT NOT NULL,
  balance_minor INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now')),
  UNIQUE (account_id, balance_date)
);

CREATE INDEX idx_balance_account_date ON account_balance_snapshots (account_id, balance_date);

CREATE INDEX idx_balance_date_account ON account_balance_snapshots (balance_date, account_id);

CREATE INDEX idx_accounts_institution ON accounts (institution_id);

INSERT INTO
  account_types (name)
VALUES
  ('current'),
  ('savings'),
  ('credit_card'),
  ('isa'),
  ('investment'),
  ('pension'),
  ('cash'),
  ('loan');
