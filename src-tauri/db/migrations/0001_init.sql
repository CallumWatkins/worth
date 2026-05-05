CREATE TABLE institutions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE
);

CREATE TABLE account_types (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE);

CREATE TABLE accounts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
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
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  account_id INTEGER NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
  balance_date TEXT NOT NULL,
  balance_minor INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now')),
  UNIQUE (account_id, balance_date)
);

CREATE TABLE app_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  analytics_enabled INTEGER NOT NULL DEFAULT 1 CHECK (analytics_enabled IN (0, 1)),
  default_display_currency_code TEXT NOT NULL DEFAULT 'GBP',
  display_locale TEXT NOT NULL DEFAULT 'system',
  theme TEXT NOT NULL DEFAULT 'dark' CHECK (theme IN ('system', 'light', 'dark')),
  created_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_balance_account_date ON account_balance_snapshots (account_id, balance_date);

CREATE INDEX idx_balance_date_account ON account_balance_snapshots (balance_date, account_id);

CREATE INDEX idx_accounts_institution ON accounts (institution_id);

CREATE VIRTUAL TABLE search_fts USING fts5 (
  kind unindexed,
  entity_id unindexed,
  name,
  institution_name,
  account_type,
  tokenize = 'unicode61 remove_diacritics 2',
  prefix = '2 3 4 5 6 7 8'
);

CREATE TRIGGER institutions_ai AFTER INSERT ON institutions BEGIN
INSERT INTO
  search_fts (
    kind,
    entity_id,
    name,
    institution_name,
    account_type
  )
VALUES
  ('institution', new.id, new.name, '', '');

END;

CREATE TRIGGER institutions_au AFTER
UPDATE ON institutions BEGIN
DELETE FROM search_fts
WHERE
  kind = 'institution'
  AND entity_id = old.id;

INSERT INTO
  search_fts (
    kind,
    entity_id,
    name,
    institution_name,
    account_type
  )
VALUES
  ('institution', new.id, new.name, '', '');

DELETE FROM search_fts
WHERE
  kind = 'account'
  AND entity_id IN (
    SELECT
      id
    FROM
      accounts
    WHERE
      institution_id = new.id
  );

INSERT INTO
  search_fts (
    kind,
    entity_id,
    name,
    institution_name,
    account_type
  )
SELECT
  'account',
  a.id,
  a.name,
  i.name,
  t.name
FROM
  accounts AS a
  INNER JOIN institutions AS i ON i.id = a.institution_id
  INNER JOIN account_types AS t ON t.id = a.type_id
WHERE
  a.institution_id = new.id;

END;

CREATE TRIGGER institutions_ad AFTER DELETE ON institutions BEGIN
DELETE FROM search_fts
WHERE
  kind = 'institution'
  AND entity_id = old.id;

END;

CREATE TRIGGER accounts_ai AFTER INSERT ON accounts BEGIN
INSERT INTO
  search_fts (
    kind,
    entity_id,
    name,
    institution_name,
    account_type
  )
SELECT
  'account',
  a.id,
  a.name,
  i.name,
  t.name
FROM
  accounts AS a
  INNER JOIN institutions AS i ON i.id = a.institution_id
  INNER JOIN account_types AS t ON t.id = a.type_id
WHERE
  a.id = new.id;

END;

CREATE TRIGGER accounts_au AFTER
UPDATE ON accounts BEGIN
DELETE FROM search_fts
WHERE
  kind = 'account'
  AND entity_id = old.id;

INSERT INTO
  search_fts (
    kind,
    entity_id,
    name,
    institution_name,
    account_type
  )
SELECT
  'account',
  a.id,
  a.name,
  i.name,
  t.name
FROM
  accounts AS a
  INNER JOIN institutions AS i ON i.id = a.institution_id
  INNER JOIN account_types AS t ON t.id = a.type_id
WHERE
  a.id = new.id;

END;

CREATE TRIGGER accounts_ad AFTER DELETE ON accounts BEGIN
DELETE FROM search_fts
WHERE
  kind = 'account'
  AND entity_id = old.id;

END;

CREATE TRIGGER account_types_au AFTER
UPDATE ON account_types BEGIN
DELETE FROM search_fts
WHERE
  kind = 'account'
  AND entity_id IN (
    SELECT
      id
    FROM
      accounts
    WHERE
      type_id = new.id
  );

INSERT INTO
  search_fts (
    kind,
    entity_id,
    name,
    institution_name,
    account_type
  )
SELECT
  'account',
  a.id,
  a.name,
  i.name,
  t.name
FROM
  accounts AS a
  INNER JOIN institutions AS i ON i.id = a.institution_id
  INNER JOIN account_types AS t ON t.id = a.type_id
WHERE
  a.type_id = new.id;

END;

INSERT INTO
  search_fts (
    kind,
    entity_id,
    name,
    institution_name,
    account_type
  )
SELECT
  'institution',
  i.id,
  i.name,
  '',
  ''
FROM
  institutions AS i;

INSERT INTO
  search_fts (
    kind,
    entity_id,
    name,
    institution_name,
    account_type
  )
SELECT
  'account',
  a.id,
  a.name,
  i.name,
  t.name
FROM
  accounts AS a
  INNER JOIN institutions AS i ON i.id = a.institution_id
  INNER JOIN account_types AS t ON t.id = a.type_id;

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

INSERT INTO
  app_settings (id)
VALUES
  (1);
