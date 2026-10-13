# 0007. Change history by journal, and full-text search by index

Status: accepted

## Context

Two things deferred from the domain model (decision 0004) are what let an author work without
fear and find what they wrote: taking an edit back (FR-13) and searching all the text of a
universe (FR-09). Both must keep working as the model grows, without each new table having to
remember them.

## Decision

**Undo is a journal kept by the database.** Every table is watched by three triggers that, while
a change set is open, append the statement that reverses each row change to `change_entry`. One
`Project::edit` is one change set, named by a stable machine label (`entity.update`) that the
interface words. Undoing runs a set's statements newest first and records what they did as a set
of the other stack, so redo is the same operation seen from the other side. A new edit empties
the redo stack; the history keeps the latest 200 edits and survives closing the project.

- The triggers are generated from the schema (`journal::install`), not written per table. A new
  table is covered by a migration alone; after any migration they are installed again and the
  history is dropped, because a statement from before the migration may no longer mean the same.
- Undoing defers foreign-key checks to the end of the set, since a cascade may log a child's row
  before its parent's.
- A file copied into `assets/` by an import is not undone; its asset record is. Importing the
  same content again finds no record and copies it once more under a new name.
- Time is injected: the adapter passes the clock when it names an edit.

**Search is an index the migrations own.** `search_index` is one FTS5 row per entity holding its
names, tags, summary, description, notes and the text of its property values, tokenized without
case or diacritics. Triggers on the tables that feed it keep it current, including when an edit
is undone; its row id is the entity's row id, so a refresh is a point lookup. Queries are built
from words: every word must appear and the last may be a prefix, and nothing typed is read as
query syntax. Names weigh most, long prose least. Results come with the passage that matched, the
matching words marked with control characters the interface turns into emphasis.

## Consequences

- Any write the domain makes is undoable, with no code per record type.
- The search index is derived data: it can be rebuilt from its view, and it never needs backup
  logic of its own.
- A new source of searchable text means a trigger in a migration that refreshes the same view.
- The edit label is part of the contract with the interface: the frontend tests that every label
  the backend records has a text.
