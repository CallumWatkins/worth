CREATE TABLE institution_identifiers (
  institution_id INTEGER NOT NULL REFERENCES institutions (id) ON DELETE CASCADE,
  position INTEGER NOT NULL CHECK (position >= 0),
  label TEXT NOT NULL CHECK (LENGTH(label) BETWEEN 1 AND 80),
  value TEXT NOT NULL CHECK (LENGTH(value) <= 250),
  search_value TEXT NOT NULL,
  PRIMARY KEY (institution_id, position)
);
