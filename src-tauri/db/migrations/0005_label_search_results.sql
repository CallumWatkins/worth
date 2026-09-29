-- Index labels as standalone results, including labels without any accounts.
CREATE TRIGGER labels_search_ai AFTER INSERT ON labels BEGIN
  INSERT INTO search_fts (kind, entity_id, name, institution_name, account_type, labels)
  VALUES ('label', new.id, new.name, '', '', '');
END;

CREATE TRIGGER labels_search_au AFTER UPDATE OF name ON labels BEGIN
  UPDATE search_fts SET name = new.name
  WHERE kind = 'label' AND entity_id = new.id;
END;

CREATE TRIGGER labels_search_ad AFTER DELETE ON labels BEGIN
  DELETE FROM search_fts WHERE kind = 'label' AND entity_id = old.id;
END;

INSERT INTO search_fts (kind, entity_id, name, institution_name, account_type, labels)
SELECT 'label', id, name, '', '', '' FROM labels;
