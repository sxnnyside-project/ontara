-- Undo and redo (FR-13).
--
-- Every edit is one change set. While a set is open, triggers installed by the storage layer
-- (journal.rs, one trio per table) append to `change_entry` the statement that reverses each row
-- change. Undoing a set runs its statements newest first and records what they did as a set of
-- the other stack, so redo is the same operation seen from the other side.
--
-- The triggers are not part of this file: they follow the tables, which later migrations add to.

CREATE TABLE change_set (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    -- Stable machine name of the operation ("entity.update"); the interface words it.
    label      TEXT NOT NULL,
    stack      TEXT NOT NULL CHECK (stack IN ('undo', 'redo')),
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX change_set_stack ON change_set (stack, id);

CREATE TABLE change_entry (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    change_set INTEGER NOT NULL REFERENCES change_set (id) ON DELETE CASCADE,
    statement  TEXT NOT NULL
) STRICT;

CREATE INDEX change_entry_set ON change_entry (change_set, id);

-- The set that row changes are being recorded into, if any.
CREATE TABLE journal_state (
    id       INTEGER PRIMARY KEY CHECK (id = 1),
    open_set INTEGER
) STRICT;

INSERT INTO journal_state (id, open_set) VALUES (1, NULL);
