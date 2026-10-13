# 0008. The dossier, and what makes the workflow lovable

Status: accepted

## Context

Beings, places, polities, factions, religions, cultures, languages and items each have records
that describe them (profiles, traits, borders, offices, memberships), which the model has held
since decision 0004 but no screen showed. Beyond that, the application had to become something
an author prefers to their current process: quick to start, reachable from the keyboard,
forgiving, and safe.

## Decision

**One dossier, driven by data.** An entity's dossier is the set of records tied to it, grouped in
sections that apply to its type (a character has traits and family ties, a city has borders and
censuses). The adapter holds one table of sections: the record type, the fields that tie it to
entities and the editor operations that write it; macros per shape (profile, items, links) turn
that into list, save and remove. Records travel as JSON, so one pair of commands serves every
kind, and the domain still validates every write. The interface holds the matching table of
fields; a test checks that every section, field and choice has its words. Fields that point at
an entity use the same picker as relationships, so what an author can choose is what the type
allows.

**One palette.** Searching the universe and running commands share one keyboard entry point
(Ctrl or Cmd with K). Typing searches; a leading `>` shows commands only. Every command is also
reachable some other way, so the palette never becomes the only door.

**Mistakes are cheap, loss is rare.** Undo and redo from the keyboard and a history dialog
(decision 0007); a verified backup once a day on opening, kept to the latest seven, and before
every upgrade; restoring keeps the project as it was as a backup first. Export writes a readable
document (Markdown) or a complete copy (JSON) into the project's `exports/` folder.

**Starting is a decision, not a chore.** A small example universe with one deliberate
contradiction can be loaded into an empty project, so the checks have something true to show.
The universe home keeps a short getting-started list until it is done or dismissed.

**Quality is measured by what can be measured.**
- `scripts/check-accessibility.ts` (in `just lint`) checks the contrast of the design tokens in
  both themes, that every button has a name, no image lacks alternative text, no control hides
  its focus, no label is written in one language, and that reduced motion is respected.
- `tests/scale.rs` builds a universe of about 1,700 entities and keeps list, search, graph,
  matrix, timeline, checks, undo and backup within a few seconds.
- `src/workflow.rs` walks the primary path from an empty folder to an exported universe and back
  out of a mistake.
- `docs/usability-protocol.md` says how to measure with representative users what these cannot.

## Consequences

- A new kind of record is a row in two tables (adapter and interface) and its texts.
- Usability against a person's current process is a human measurement; the protocol makes it
  repeatable but the exit condition of the phase is only met once it has been run.
