# 0004. Narrative domain model

Status: accepted (resolves open decision 6 of the product definition for the foundation phase)

## Context

Worldbuilding needs more than entities and links. What makes an entity individual, when things
hold in the fictional world, where characters appear, who ruled, who belonged to what, and how
settled each statement is are all part of the model, not annotations on it.

## Decision

**The model is relational and explicit.** What every author needs has a table or a column with
a name: a being's profile, how it looks over a period, its traits, marks and languages; places,
their containment, borders and census; polities, offices and their tenures, citizenship and
diplomacy; factions and memberships; culture, language and belief; items and ownership; events
and causality. The full catalog is in `docs/data-model/overview.md`. Generic properties remain
only for what is particular to one universe.

**Types form a hierarchy.** An entity type may specialize another: a city is a settlement is a
place. A type inherits the properties of its ancestors and satisfies any rule written for them.
Abstract types only group subtypes.

**Canon is data.** A canon state is a catalog row with a key, label and a flag saying whether it
counts as canon. Six ship with every project: established, provisional, disputed, deprecated,
rejected and disestablished. Projects add their own. Every claim carries a state, and every
change of state is kept with its reason.

**Time is a shared timeline plus calendars.** A moment is what the author tells: a label, a day
on the shared timeline, or both. A day is optional, and may carry uncertainty bounds and a
calendar. A calendar is normalized into months, weekdays, leap rules with exceptions and named
eras that can count backwards, and converts dates to universe days and back. A fact's period is
unknown (a date that must still be found), unconstrained (deliberately off the timeline), a
point, or a span with either end open.

**Facts about the world are tables with periods.** Containment, family ties, partnerships,
memberships, tenures, citizenship, ownership and diplomacy each hold a period and a canon
state. Whether something holds today is derived from its period; no flag is stored.

**Hierarchical facts never loop.** Containment (per kind), family ties, faction parents,
succession and causality cannot lead back to where they started. Only facts that have not ended
count, so history cannot block the present.

**Types have structural roles.** An entity type has a role: generic, event, editorial unit or
asset. Rules attach to roles, not names.

**Appearances are records.** An appearance links an entity to a narrative context with a
configurable kind, a narrative order, an optional viewpoint and a canon state. A kind can be a
confirmed absence, so the matrix can tell absent from unknown: no record means unknown.

**Provenance.** Every claim says whether it was made by the user, an AI provider, MCP or an
import. Sources and citations let any record point to what supports it.

## Not in this phase

Undo history and change sets (FR-13), the full-text search index (FR-09), stored consistency
findings and their documented exceptions (FR-10) and the operations that edit catalog entries
after creation (relabeling, FR-02) arrive with the phases that need them. None of them changes
what is stored here; each adds tables or operations through new migrations.

## Consequences

- The domain engine knows no genre; the starting traits and vocabularies are data.
- A custom calendar is part of the model from the start, so events and dates are never
  reinterpreted.
- Consistency checks can rely on canon flags, roles, periods and the shared timeline.
- Adding a concept means a table, its record type, its editor operations and its storage
  mapping, all following one pattern, with tests at both levels.
