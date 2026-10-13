-- Full-text search (FR-09).
--
-- `search_index` holds one row per entity: its names, tags, summary, description, notes and the
-- text of its property values, so one query finds a character by what is written about them.
-- The row id is the entity's row id, which makes a refresh a point lookup. Triggers on the
-- tables that feed it keep the index current, including when an edit is undone.

CREATE VIEW search_document AS
SELECT
    e.rowid AS id,
    e.id AS entity_id,
    e.archived AS archived,
    e.name AS name,
    TRIM(COALESCE(e.sort_name, '') || ' ' ||
         COALESCE((SELECT group_concat(n.name, ' ') FROM entity_name n
                   WHERE n.entity_id = e.id), '')) AS aliases,
    COALESCE((SELECT group_concat(t.name, ' ') FROM entity_tag et
              JOIN tag t ON t.id = et.tag_id WHERE et.entity_id = e.id), '') AS tags,
    e.summary AS summary,
    e.description AS description,
    COALESCE((SELECT group_concat(note.title || ' ' || note.body, ' ') FROM note
              WHERE note.entity_id = e.id), '') AS notes,
    COALESCE((SELECT group_concat(v.value_text, ' ') FROM property_value v
              WHERE v.entity_id = e.id AND v.value_text IS NOT NULL), '') AS details
FROM entity e;

CREATE VIRTUAL TABLE search_index USING fts5(
    entity_id UNINDEXED,
    archived UNINDEXED,
    name,
    aliases,
    tags,
    summary,
    description,
    notes,
    details,
    tokenize = 'unicode61 remove_diacritics 2'
);

INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description,
                          notes, details)
SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details
FROM search_document;

CREATE TRIGGER search_entity_insert AFTER INSERT ON entity
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = NEW.id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = NEW.id;
END;

CREATE TRIGGER search_entity_update AFTER UPDATE ON entity
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = NEW.id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = NEW.id;
END;

CREATE TRIGGER search_entity_delete AFTER DELETE ON entity
BEGIN
    DELETE FROM search_index WHERE rowid = OLD.rowid;
END;

CREATE TRIGGER search_entity_name_insert AFTER INSERT ON entity_name
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = NEW.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = NEW.entity_id;
END;

CREATE TRIGGER search_entity_name_update AFTER UPDATE ON entity_name
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = OLD.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = OLD.entity_id;
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = NEW.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = NEW.entity_id;
END;

CREATE TRIGGER search_entity_name_delete AFTER DELETE ON entity_name
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = OLD.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = OLD.entity_id;
END;

CREATE TRIGGER search_entity_tag_insert AFTER INSERT ON entity_tag
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = NEW.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = NEW.entity_id;
END;

CREATE TRIGGER search_entity_tag_update AFTER UPDATE ON entity_tag
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = OLD.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = OLD.entity_id;
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = NEW.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = NEW.entity_id;
END;

CREATE TRIGGER search_entity_tag_delete AFTER DELETE ON entity_tag
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = OLD.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = OLD.entity_id;
END;

CREATE TRIGGER search_note_insert AFTER INSERT ON note
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = NEW.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = NEW.entity_id;
END;

CREATE TRIGGER search_note_update AFTER UPDATE ON note
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = OLD.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = OLD.entity_id;
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = NEW.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = NEW.entity_id;
END;

CREATE TRIGGER search_note_delete AFTER DELETE ON note
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = OLD.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = OLD.entity_id;
END;

CREATE TRIGGER search_property_value_insert AFTER INSERT ON property_value
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = NEW.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = NEW.entity_id;
END;

CREATE TRIGGER search_property_value_update AFTER UPDATE ON property_value
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = OLD.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = OLD.entity_id;
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = NEW.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = NEW.entity_id;
END;

CREATE TRIGGER search_property_value_delete AFTER DELETE ON property_value
BEGIN
    DELETE FROM search_index WHERE rowid = (SELECT rowid FROM entity WHERE id = OLD.entity_id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary, description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details FROM search_document WHERE entity_id = OLD.entity_id;
END;

-- `tag` rows are renamed in place; every entity carrying the tag is refreshed.
CREATE TRIGGER search_tag_update AFTER UPDATE OF name ON tag
BEGIN
    DELETE FROM search_index WHERE rowid IN
        (SELECT e.rowid FROM entity e JOIN entity_tag et ON et.entity_id = e.id
         WHERE et.tag_id = NEW.id);
    INSERT INTO search_index (rowid, entity_id, archived, name, aliases, tags, summary,
                              description, notes, details)
    SELECT id, entity_id, archived, name, aliases, tags, summary, description, notes, details
    FROM search_document
    WHERE entity_id IN (SELECT entity_id FROM entity_tag WHERE tag_id = NEW.id);
END;
