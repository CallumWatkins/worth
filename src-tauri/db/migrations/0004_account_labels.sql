CREATE TABLE labels (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL CHECK (LENGTH(name) BETWEEN 1 AND 20 AND name = TRIM(name)),
  -- Rust supplies the trimmed, case-normalized key; SQLite NOCASE is ASCII-only.
  name_key TEXT NOT NULL UNIQUE CHECK (LENGTH(name_key) > 0),
  description TEXT,
  created_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE account_labels (
  account_id INTEGER NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
  label_id INTEGER NOT NULL REFERENCES labels (id) ON DELETE CASCADE,
  PRIMARY KEY (account_id, label_id)
);

CREATE INDEX idx_account_labels_label ON account_labels (label_id, account_id);

-- Centralize the joined account document used by the search synchronization triggers.
CREATE VIEW account_search_documents AS
SELECT
  'account' AS kind,
  a.id AS entity_id,
  a.name,
  i.name AS institution_name,
  t.name AS account_type,
  COALESCE((
    SELECT GROUP_CONCAT(l.name, ' ')
    FROM account_labels AS al
    INNER JOIN labels AS l ON l.id = al.label_id
    WHERE al.account_id = a.id
  ), '') AS labels
FROM accounts AS a
INNER JOIN institutions AS i ON i.id = a.institution_id
INNER JOIN account_types AS t ON t.id = a.type_id;

DROP TRIGGER institutions_ai;
DROP TRIGGER institutions_au;
DROP TRIGGER institutions_ad;
DROP TRIGGER accounts_ai;
DROP TRIGGER accounts_au;
DROP TRIGGER accounts_ad;
DROP TRIGGER account_types_au;
DROP TABLE search_fts;

CREATE VIRTUAL TABLE search_fts USING fts5 (
  kind unindexed,
  entity_id unindexed,
  name,
  institution_name,
  account_type,
  labels,
  tokenize = 'unicode61 remove_diacritics 2',
  prefix = '2 3 4 5 6 7 8'
);

CREATE TRIGGER institutions_ai AFTER INSERT ON institutions BEGIN
  INSERT INTO search_fts (kind, entity_id, name, institution_name, account_type, labels)
  VALUES ('institution', new.id, new.name, '', '', '');
END;

CREATE TRIGGER institutions_au AFTER UPDATE OF name ON institutions BEGIN
  UPDATE search_fts SET name = new.name
  WHERE kind = 'institution' AND entity_id = new.id;

  UPDATE search_fts SET institution_name = new.name
  WHERE kind = 'account' AND entity_id IN (
    SELECT id FROM accounts WHERE institution_id = new.id
  );
END;

CREATE TRIGGER institutions_ad AFTER DELETE ON institutions BEGIN
  DELETE FROM search_fts WHERE kind = 'institution' AND entity_id = old.id;
END;

CREATE TRIGGER accounts_ai AFTER INSERT ON accounts BEGIN
  INSERT INTO search_fts (kind, entity_id, name, institution_name, account_type, labels)
  SELECT kind, entity_id, name, institution_name, account_type, labels
  FROM account_search_documents WHERE entity_id = new.id;
END;

CREATE TRIGGER accounts_au AFTER UPDATE OF name, institution_id, type_id ON accounts BEGIN
  DELETE FROM search_fts WHERE kind = 'account' AND entity_id = old.id;

  INSERT INTO search_fts (kind, entity_id, name, institution_name, account_type, labels)
  SELECT kind, entity_id, name, institution_name, account_type, labels
  FROM account_search_documents WHERE entity_id = new.id;
END;

CREATE TRIGGER accounts_ad AFTER DELETE ON accounts BEGIN
  DELETE FROM search_fts WHERE kind = 'account' AND entity_id = old.id;
END;

CREATE TRIGGER account_types_au AFTER UPDATE OF name ON account_types BEGIN
  UPDATE search_fts SET account_type = new.name
  WHERE kind = 'account' AND entity_id IN (
    SELECT id FROM accounts WHERE type_id = new.id
  );
END;

CREATE TRIGGER account_labels_ai AFTER INSERT ON account_labels BEGIN
  UPDATE search_fts SET labels = (
    SELECT labels FROM account_search_documents WHERE entity_id = new.account_id
  )
  WHERE kind = 'account' AND entity_id = new.account_id;
END;

CREATE TRIGGER account_labels_ad AFTER DELETE ON account_labels BEGIN
  UPDATE search_fts SET labels = COALESCE((
    SELECT labels FROM account_search_documents WHERE entity_id = old.account_id
  ), '')
  WHERE kind = 'account' AND entity_id = old.account_id;
END;

CREATE TRIGGER account_labels_au AFTER UPDATE OF account_id, label_id ON account_labels BEGIN
  UPDATE search_fts SET labels = (
    SELECT labels FROM account_search_documents WHERE entity_id = search_fts.entity_id
  )
  WHERE kind = 'account' AND entity_id IN (old.account_id, new.account_id);
END;

CREATE TRIGGER labels_au AFTER UPDATE OF name ON labels BEGIN
  UPDATE search_fts SET labels = (
    SELECT labels FROM account_search_documents WHERE entity_id = search_fts.entity_id
  )
  WHERE kind = 'account' AND entity_id IN (
    SELECT account_id FROM account_labels WHERE label_id = new.id
  );
END;

-- Rebuild existing search entries without changing accounts or their balance history.
INSERT INTO search_fts (kind, entity_id, name, institution_name, account_type, labels)
SELECT 'institution', id, name, '', '', '' FROM institutions;

INSERT INTO search_fts (kind, entity_id, name, institution_name, account_type, labels)
SELECT kind, entity_id, name, institution_name, account_type, labels
FROM account_search_documents;
