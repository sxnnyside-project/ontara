# Glossary

One concept, one term, across code, files, tests and docs. Add a term in the same change that introduces it.

| Term | Meaning |
|---|---|
| Universe | The top-level fictional world a project models. |
| Project | A portable local folder holding one universe: manifest, database, assets, exports, backups. |
| Entity | Any identified element of a universe: character, location, event, chapter and so on. Identified by a stable id, never by name. |
| Entity type | The kind of an entity. Built-in types exist and projects may define more. |
| Relationship | A first-class, typed, optionally directed connection between two entities, with its own properties, context and canon status. |
| Appearance | The recorded presence of an entity in a narrative context (a chapter, scene or event), with a classification and an order. |
| Event | Something that happens in the universe, with participants, location and temporal information. |
| Editorial structure | The configurable hierarchy that organizes narrative material: series, volume, part, chapter, scene. |
| In-universe chronology | The order in which events happen inside the fictional world. |
| Narrative order | The order in which a reader encounters material. May differ from in-universe chronology. |
| Canon state | Configurable state of a record with respect to canon: established, provisional, disputed, deprecated, rejected, disestablished, or one the project defines. |
| Consistency check | A deterministic rule run over the model that reports findings without modifying canon. |
| Finding | The result of a consistency check: severity, affected records and the rule that triggered it. |
| Domain engine | The Rust crate (`ontara-core`) that holds authoritative domain rules, independent of Tauri. |
| Adapter | Code that connects the domain to an outside world: Tauri commands, and later AI providers and MCP. |
| Type key | The stable snake_case machine identity of an entity type or relationship type. Visible labels may change; the key does not. |
| Manifest | The `ontara.project.json` file at the root of a project: format version, project id and name. |
| Schema version | The version of the database structure, stored in the database itself and advanced only by migrations. |
| Migration | An ordered, append-only change to the database schema. |
| Backup | A verified copy of a project's database kept in the project's `backups/` folder. |
| Model editor | The domain component through which every write passes, enforcing domain rules. |
| Property definition | A trait, characteristic or attribute that records of an entity type or relationship type can carry, with a data type, a group and whether it is required or multiple. |
| Property value | One recorded value of a property definition, with its own canon state, validity period and sources. |
| Property group | A section that organizes the property definitions of an entity type, such as Appearance or Personality. |
| Entity role | The structural job of an entity type: generic, event, editorial unit or asset. |
| Instant | A point on the shared in-universe timeline: universe day, precision, optional uncertainty and label. |
| Temporal | When something holds: unknown, unconstrained, an instant, or a span. |
| Calendar | An in-universe way of counting days: months, leap rules, weekdays and eras. |
| Universe day | The integer position of a day on the shared timeline; the key every calendar converts to. |
| Appearance kind | How an entity is present in a context: physical presence, dialogue, mention, flashback, remote presence or confirmed absence. |
| Citation | A source cited as support for a record. |
| Origin | Whether a record was made by the user, an AI provider, MCP or an import. |
| Moment | What the author tells about a time: a label, a day on the shared timeline, or both. A day is optional. |
| Period | When a fact holds: unknown, unconstrained, a point, or a span with either end open. |
| Profile | The record that holds what is true of an entity as a whole, such as a being's gender and birthplace. |
| Containment | A place sitting inside another, of a given kind and over a period. |
| Office | A seat of power a polity or faction has; a reign is a tenure of an office. |
| Tenure | Who held an office, when, and how they came to hold it. |
| Census | The population of a place at a moment, with where the figure comes from. |
| Vocabulary | A set of terms an author may extend, kept as a table so every use is a reference. |
| Catalog entry | One configurable row of a catalog: an entity type, relationship type, canon state, editorial level, appearance kind, property or vocabulary term. Its key never changes; its label may. |
| Section | A place in the editorial structure: a volume, part, chapter or scene, each also an entity. |
| Connection | One link between an entity and another, in either direction: a relationship, an appearance, event participation, an attachment or a section. |
| Asset import | Copying a file into the project's `assets/`, deduplicated by content, and registering it as an asset entity. |
| Graph | The neighborhood of an entity: entities within a number of steps and the links between them. |
| Appearance matrix | Entities against sections or events, showing how each is present, absent or unrecorded. |
| Timeline | Events ordered by in-universe day, apart from the order in which the work tells them. |
| Out of order | An event the work tells after something that happens later in the universe. |
| Rule | One consistency check, with a stable code (CC-01 …) and a severity. |
| Focus | The entity every view is looking at; changing view keeps it. |
| Change set | Everything one edit changed, recorded so it can be undone as a whole. |
| History | The change sets of a project that can be undone, and those that were undone and can be redone. |
| Search index | The derived full-text index of an entity's names, tags, text, notes and property text. |
| Dossier | The records that describe one entity beyond its name and text, grouped in sections that apply to its type. |
| Dossier section | One kind of record in a dossier: traits, family ties, borders, offices and so on. Not an editorial section. |
| Command palette | The keyboard entry point that searches the universe and runs commands. |
| Example universe | A small universe, with one deliberate contradiction, that can be loaded into an empty project. |
| Export | A document (Markdown) or complete copy (JSON) of the universe written into the project's `exports/` folder. |
| Provider | A model service the author configured with their own key: Anthropic, an OpenAI-compatible server or one running on this computer. |
| Proposal | A set of changes an assistant suggests, kept apart from the universe until the author approves each one. |
| Proposed change | One operation of the closed set an assistant may suggest: create an entity, update its text, add a relationship, add a note. |
| Assistant policy | What a project allows assistants to reach: sending text to providers over the internet, and MCP access. Both start off. |
| MCP | The protocol the local `ontara-mcp` server speaks so an assistant on this computer can read a project and propose changes. |
