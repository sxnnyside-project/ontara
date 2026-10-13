# 0006. Analysis views and deterministic consistency checks

Status: accepted

## Context

Once a universe is modeled, maintaining it means answering questions across the whole model:
who is connected to whom, who appears where, what happens when, and what contradicts what.

## Decision

**Analysis is domain code over the reader.** `graph`, `appearance_matrix`, `timeline` and
`consistency` live in `ontara-core`, read through `ModelReader`, never write, and have pure
cores (`build_timeline`, `check_snapshot`) that run on records made by hand, so rules are tested
without a database.

**Graph exploration is bounded.** A neighborhood is walked breadth first from one entity up to
four steps, through relationships and, optionally, event participation and location. It stops at
150 entities and says it was cut. The layout is computed in the frontend, deterministically.

**The matrix shows only what was recorded.** Rows are entities (any entity filter), columns are
sections in reading order or events in chronological order. Absence is a recorded fact, distinct
from nothing recorded. One gesture places, changes or clears an appearance.

**Chronology and narrative order are independent.** The timeline orders events by in-universe
day; the reading order comes from the editorial structure and the appearances of the event in
sections. An event told after something that happens later is flagged as out of order.

**Consistency checks are deterministic and read-only.** A rule has a stable code and severity.
The same data gives the same findings in the same order, each with a stable fingerprint. Records
whose canon state does not count as canon are not held to the rules; checks never change canon.

| Code | Rule | Severity |
|---|---|---|
| CC-01 / CC-02 | An event is dated after a participant's death / before their birth | error |
| CC-03 | A parent was born on or after their child | error |
| CC-04 | An effect starts before its cause | error |
| CC-05 | An entity is present and confirmed absent in one context | error |
| CC-06 | An entity is in overlapping events at different places | warning |
| CC-07 | A canon record rests on an entity that is not canon | warning |
| CC-08 | A required property has no value | warning |
| CC-09 | A scene is told through someone who is not present | warning |
| CC-10 | An office has more holders at once than seats | warning |
| CC-11 | An editorial unit is not placed in the structure | warning |
| CC-12 | An event has no known day | info |
| CC-13 | An entity is connected to nothing | info |
| CC-14 | Two entities of one type share a name | warning |

**Views share one focus.** The entity in focus and the view are one navigation state, with a
trail that records both, so the same entity can be followed from the page to the graph, matrix
and timeline and back.

## Consequences

- Adding a rule means: a variant of `Rule` with code and severity, its check in the matching
  `consistency/` theme file, a test on hand-made records, and its texts in both languages.
- Dismissing or acknowledging a finding is not stored yet; it needs a table and a migration.
