ALTER TABLE accounts ADD COLUMN include_in_dashboard INTEGER NOT NULL DEFAULT 1 CHECK (include_in_dashboard IN (0, 1));
