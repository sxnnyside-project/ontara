-- Ontara relational model: proposal for review.
--
-- Not wired into the application. The current migration is
-- src-tauri/crates/ontara-storage/migrations/0001_initial_schema.sql; this file replaces it once
-- the model is approved.
--
-- Conventions
--   * Every table is STRICT. Identifiers are UUID text, keys are snake_case text, timestamps are
--     milliseconds since the Unix epoch, booleans are 0/1 with a CHECK.
--   * Real-world time (created_at, updated_at) is never mixed with in-universe time. In-universe
--     time is an `instant` row; a fact that holds over a period has `time_state`, `starts_at`
--     and `ends_at`.
--   * Anything an author states carries canon_state, origin, created_at and updated_at.
--   * Facts that change over time (membership, tenure, citizenship, ownership, census,
--     containment, appearance) are tables of their own with a period, never overwritten.
--   * Foreign keys to `entity` do not say what type the entity must be. The domain engine
--     enforces that, using the entity type hierarchy.

---------------------------------------------------------------------------------------------
-- 1. Universe
---------------------------------------------------------------------------------------------

CREATE TABLE universe_profile (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    summary     TEXT NOT NULL,
    description TEXT NOT NULL,
    updated_at  INTEGER
) STRICT;

---------------------------------------------------------------------------------------------
-- 2. Time: calendars and instants
---------------------------------------------------------------------------------------------

CREATE TABLE calendar (
    id               TEXT PRIMARY KEY,
    name             TEXT NOT NULL,
    description      TEXT NOT NULL,
    is_default       INTEGER NOT NULL CHECK (is_default IN (0, 1)),
    -- Universe day of the first day of year 0, aligning calendars on the shared timeline.
    universe_offset  INTEGER NOT NULL,
    -- Index into calendar_weekday of the first day of year 0.
    epoch_weekday    INTEGER NOT NULL,
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL
) STRICT;

CREATE UNIQUE INDEX calendar_single_default ON calendar (is_default) WHERE is_default = 1;

CREATE TABLE calendar_month (
    calendar_id TEXT NOT NULL REFERENCES calendar (id),
    position    INTEGER NOT NULL CHECK (position >= 1),
    label       TEXT NOT NULL,
    days        INTEGER NOT NULL CHECK (days >= 1),
    PRIMARY KEY (calendar_id, position)
) STRICT;

CREATE TABLE calendar_weekday (
    calendar_id TEXT NOT NULL REFERENCES calendar (id),
    position    INTEGER NOT NULL CHECK (position >= 0),
    label       TEXT NOT NULL,
    PRIMARY KEY (calendar_id, position)
) STRICT;

-- Adds days to one month in the years the rule selects: every `every_years`, except every
-- `except_every_years`, unless every `unless_every_years` (Gregorian is 4 / 100 / 400).
CREATE TABLE calendar_leap_rule (
    id                 INTEGER PRIMARY KEY,
    calendar_id        TEXT NOT NULL REFERENCES calendar (id),
    every_years        INTEGER NOT NULL CHECK (every_years >= 1),
    except_every_years INTEGER CHECK (except_every_years >= 1),
    unless_every_years INTEGER CHECK (unless_every_years >= 1),
    month_position     INTEGER NOT NULL,
    extra_days         INTEGER NOT NULL CHECK (extra_days >= 1),
    CHECK (unless_every_years IS NULL OR except_every_years IS NOT NULL),
    FOREIGN KEY (calendar_id, month_position) REFERENCES calendar_month (calendar_id, position)
) STRICT;

-- An era lasts until the next era's start year. A backward era counts down towards it.
CREATE TABLE calendar_era (
    calendar_id TEXT NOT NULL REFERENCES calendar (id),
    start_year  INTEGER NOT NULL,
    label       TEXT NOT NULL,
    direction   TEXT NOT NULL CHECK (direction IN ('forward', 'backward')),
    PRIMARY KEY (calendar_id, start_year)
) STRICT;

-- A moment the author tells. It always says something (a label, a day, or both); an exact
-- position on the shared timeline is optional. Without a day an instant cannot be ordered, but
-- it exists, can be cited and can be given a day later.
CREATE TABLE instant (
    id           TEXT PRIMARY KEY,
    -- Position on the shared timeline: the ordering key, when the author knows it.
    day          INTEGER,
    precision    TEXT NOT NULL DEFAULT 'day'
        CHECK (precision IN ('day', 'month', 'year', 'decade', 'century', 'millennium')),
    -- Bounds of an uncertain day. Only meaningful when a day exists.
    earliest_day INTEGER,
    latest_day   INTEGER,
    calendar_id  TEXT REFERENCES calendar (id),
    -- The author's own wording: "the winter the river froze".
    label        TEXT,
    CHECK (day IS NOT NULL OR (label IS NOT NULL AND length(trim(label)) > 0)),
    CHECK (day IS NOT NULL OR (earliest_day IS NULL AND latest_day IS NULL)),
    CHECK (earliest_day IS NULL OR earliest_day <= day),
    CHECK (latest_day IS NULL OR latest_day >= day)
) STRICT;

CREATE INDEX instant_by_day ON instant (day);

---------------------------------------------------------------------------------------------
-- 3. Catalogs: what a project is allowed to say
---------------------------------------------------------------------------------------------

CREATE TABLE canon_state (
    key             TEXT PRIMARY KEY,
    label           TEXT NOT NULL,
    description     TEXT NOT NULL,
    counts_as_canon INTEGER NOT NULL CHECK (counts_as_canon IN (0, 1)),
    sort_order      INTEGER NOT NULL,
    builtin         INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

CREATE TABLE entity_type (
    key         TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT NOT NULL,
    parent      TEXT REFERENCES entity_type (key),
    is_abstract INTEGER NOT NULL CHECK (is_abstract IN (0, 1)),
    role        TEXT NOT NULL CHECK (role IN ('generic', 'event', 'editorial_unit', 'asset')),
    sort_order  INTEGER NOT NULL,
    builtin     INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

CREATE INDEX entity_type_by_parent ON entity_type (parent);

CREATE TABLE relationship_type (
    key            TEXT PRIMARY KEY,
    label          TEXT NOT NULL,
    inverse_label  TEXT,
    description    TEXT NOT NULL,
    directionality TEXT NOT NULL CHECK (directionality IN ('directed', 'undirected')),
    acyclic        INTEGER NOT NULL CHECK (acyclic IN (0, 1)),
    sort_order     INTEGER NOT NULL,
    builtin        INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

CREATE TABLE relationship_type_endpoint (
    relationship_type TEXT NOT NULL REFERENCES relationship_type (key),
    side              TEXT NOT NULL CHECK (side IN ('source', 'target')),
    entity_type       TEXT NOT NULL REFERENCES entity_type (key),
    PRIMARY KEY (relationship_type, side, entity_type)
) STRICT;

CREATE TABLE appearance_kind (
    key         TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT NOT NULL,
    is_presence INTEGER NOT NULL CHECK (is_presence IN (0, 1)),
    sort_order  INTEGER NOT NULL,
    builtin     INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

CREATE TABLE editorial_level (
    key     TEXT PRIMARY KEY,
    label   TEXT NOT NULL,
    depth   INTEGER NOT NULL UNIQUE,
    builtin INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

-- Vocabularies. Each is a table so a project can add terms and every use is a foreign key.
-- Built-in terms are listed in docs/data-model/overview.md.

CREATE TABLE trait_kind (
    key         TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT NOT NULL,
    sort_order  INTEGER NOT NULL,
    builtin     INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

CREATE TABLE containment_kind (
    key         TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT NOT NULL,
    sort_order  INTEGER NOT NULL,
    builtin     INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

CREATE TABLE family_tie_kind (
    key         TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT NOT NULL,
    sort_order  INTEGER NOT NULL,
    builtin     INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

CREATE TABLE partnership_kind (
    key         TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT NOT NULL,
    sort_order  INTEGER NOT NULL,
    builtin     INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

CREATE TABLE diplomatic_stance (
    key         TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT NOT NULL,
    -- Whether the stance is between unequals, so one side is the senior party.
    is_hierarchical INTEGER NOT NULL CHECK (is_hierarchical IN (0, 1)),
    sort_order  INTEGER NOT NULL,
    builtin     INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

CREATE TABLE office_acquisition (
    key         TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT NOT NULL,
    sort_order  INTEGER NOT NULL,
    builtin     INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

CREATE TABLE item_acquisition (
    key         TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT NOT NULL,
    sort_order  INTEGER NOT NULL,
    builtin     INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

CREATE TABLE citizenship_status (
    key         TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    description TEXT NOT NULL,
    sort_order  INTEGER NOT NULL,
    builtin     INTEGER NOT NULL CHECK (builtin IN (0, 1))
) STRICT;

---------------------------------------------------------------------------------------------
-- 4. Entities
---------------------------------------------------------------------------------------------

CREATE TABLE entity (
    id          TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL REFERENCES entity_type (key),
    name        TEXT NOT NULL,
    -- Name used to order lists ("Wanderer, The").
    sort_name   TEXT,
    summary     TEXT NOT NULL,
    description TEXT NOT NULL,
    archived    INTEGER NOT NULL CHECK (archived IN (0, 1)),
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
) STRICT;

CREATE INDEX entity_by_type ON entity (entity_type);
CREATE INDEX entity_by_name ON entity (name);
CREATE INDEX entity_by_canon_state ON entity (canon_state);

-- Every name an entity has had or is known by, with when and where each applies.
CREATE TABLE entity_name (
    id        TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL REFERENCES entity (id),
    name      TEXT NOT NULL,
    kind      TEXT NOT NULL CHECK (kind IN
        ('alias', 'epithet', 'birth_name', 'former_name', 'title', 'translation', 'nickname')),
    language_id TEXT REFERENCES entity (id),
    is_preferred INTEGER NOT NULL CHECK (is_preferred IN (0, 1)),
    note      TEXT,
    time_state TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at TEXT REFERENCES instant (id),
    ends_at   TEXT REFERENCES instant (id),
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX entity_name_by_entity ON entity_name (entity_id);
CREATE INDEX entity_name_by_name ON entity_name (name);

CREATE TABLE tag (
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
) STRICT;

CREATE TABLE entity_tag (
    entity_id TEXT NOT NULL REFERENCES entity (id),
    tag_id    INTEGER NOT NULL REFERENCES tag (id),
    PRIMARY KEY (entity_id, tag_id)
) STRICT;

CREATE INDEX entity_tag_by_tag ON entity_tag (tag_id);

CREATE TABLE note (
    id         TEXT PRIMARY KEY,
    entity_id  TEXT NOT NULL REFERENCES entity (id),
    title      TEXT NOT NULL,
    body       TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

CREATE INDEX note_by_entity ON note (entity_id);

---------------------------------------------------------------------------------------------
-- 5. Beings: characters, deities and every other creature with a personal story
---------------------------------------------------------------------------------------------

CREATE TABLE being_profile (
    entity_id     TEXT PRIMARY KEY REFERENCES entity (id),
    gender        TEXT,
    pronouns      TEXT,
    species_id    TEXT REFERENCES entity (id),
    birth_at      TEXT REFERENCES instant (id),
    death_at      TEXT REFERENCES instant (id),
    birthplace_id TEXT REFERENCES entity (id),
    upbringing    TEXT,
    occupation    TEXT
) STRICT;

CREATE INDEX being_profile_by_species ON being_profile (species_id);
CREATE INDEX being_profile_by_birthplace ON being_profile (birthplace_id);

-- How a being looks over a period. A being has one row with no period for its usual look, and
-- more when it changes: aging, a scar, a curse, a disguise.
CREATE TABLE being_appearance (
    id               TEXT PRIMARY KEY,
    being_id         TEXT NOT NULL REFERENCES entity (id),
    label            TEXT,
    hair_color       TEXT,
    hair_style       TEXT,
    eye_color        TEXT,
    skin_tone        TEXT,
    height_cm        REAL CHECK (height_cm IS NULL OR height_cm > 0),
    weight_kg        REAL CHECK (weight_kg IS NULL OR weight_kg > 0),
    build            TEXT,
    voice            TEXT,
    clothing_style   TEXT,
    time_state       TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at        TEXT REFERENCES instant (id),
    ends_at          TEXT REFERENCES instant (id),
    note             TEXT,
    canon_state      TEXT NOT NULL REFERENCES canon_state (key),
    origin           TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL,
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX being_appearance_by_being ON being_appearance (being_id);

CREATE TABLE being_mark (
    id           TEXT PRIMARY KEY,
    being_id     TEXT NOT NULL REFERENCES entity (id),
    description  TEXT NOT NULL,
    body_location TEXT,
    acquired_at  TEXT REFERENCES instant (id),
    canon_state  TEXT NOT NULL REFERENCES canon_state (key),
    origin       TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL
) STRICT;

CREATE INDEX being_mark_by_being ON being_mark (being_id);

-- What makes a being individual: personality, values, fears, quirks, habits, flaws, secrets,
-- abilities, skills, weaknesses. Each can hold for a period and be disputed.
CREATE TABLE being_trait (
    id          TEXT PRIMARY KEY,
    being_id    TEXT NOT NULL REFERENCES entity (id),
    kind        TEXT NOT NULL REFERENCES trait_kind (key),
    text        TEXT NOT NULL,
    -- 1 (barely) to 5 (defining), when the author wants to say how strong the trait is.
    intensity   INTEGER CHECK (intensity BETWEEN 1 AND 5),
    time_state  TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at   TEXT REFERENCES instant (id),
    ends_at     TEXT REFERENCES instant (id),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX being_trait_by_being ON being_trait (being_id, kind);

CREATE TABLE being_language (
    being_id    TEXT NOT NULL REFERENCES entity (id),
    language_id TEXT NOT NULL REFERENCES entity (id),
    proficiency TEXT NOT NULL CHECK (proficiency IN
        ('native', 'fluent', 'conversational', 'basic', 'reads_only')),
    note        TEXT,
    PRIMARY KEY (being_id, language_id)
) STRICT;

CREATE INDEX being_language_by_language ON being_language (language_id);

CREATE TABLE family_tie (
    id          TEXT PRIMARY KEY,
    parent_id   TEXT NOT NULL REFERENCES entity (id),
    child_id    TEXT NOT NULL REFERENCES entity (id),
    kind        TEXT NOT NULL REFERENCES family_tie_kind (key),
    time_state  TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at   TEXT REFERENCES instant (id),
    ends_at     TEXT REFERENCES instant (id),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (parent_id <> child_id),
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX family_tie_by_parent ON family_tie (parent_id);
CREATE INDEX family_tie_by_child ON family_tie (child_id);

CREATE TABLE partnership (
    id          TEXT PRIMARY KEY,
    first_id    TEXT NOT NULL REFERENCES entity (id),
    second_id   TEXT NOT NULL REFERENCES entity (id),
    kind        TEXT NOT NULL REFERENCES partnership_kind (key),
    end_reason  TEXT CHECK (end_reason IN
        ('death', 'divorce', 'annulment', 'separation', 'broken', 'ongoing')),
    time_state  TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at   TEXT REFERENCES instant (id),
    ends_at     TEXT REFERENCES instant (id),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    -- The pair is unordered; storing it in id order keeps one row per pair and kind.
    CHECK (first_id < second_id),
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX partnership_by_first ON partnership (first_id);
CREATE INDEX partnership_by_second ON partnership (second_id);

CREATE TABLE deity_profile (
    entity_id TEXT PRIMARY KEY REFERENCES entity (id),
    rank      TEXT,
    alignment TEXT
) STRICT;

CREATE TABLE deity_domain (
    id        TEXT PRIMARY KEY,
    deity_id  TEXT NOT NULL REFERENCES entity (id),
    domain    TEXT NOT NULL,
    note      TEXT
) STRICT;

CREATE INDEX deity_domain_by_deity ON deity_domain (deity_id);

CREATE TABLE species_profile (
    entity_id         TEXT PRIMARY KEY REFERENCES entity (id),
    is_sapient        INTEGER CHECK (is_sapient IN (0, 1)),
    lifespan_min_years INTEGER CHECK (lifespan_min_years >= 0),
    lifespan_max_years INTEGER CHECK (lifespan_max_years >= 0),
    average_height_cm REAL,
    diet              TEXT,
    CHECK (lifespan_max_years IS NULL OR lifespan_min_years IS NULL
           OR lifespan_max_years >= lifespan_min_years)
) STRICT;

CREATE TABLE species_habitat (
    species_id TEXT NOT NULL REFERENCES entity (id),
    place_id   TEXT NOT NULL REFERENCES entity (id),
    note       TEXT,
    PRIMARY KEY (species_id, place_id)
) STRICT;

---------------------------------------------------------------------------------------------
-- 6. Places and polities
---------------------------------------------------------------------------------------------

CREATE TABLE place_profile (
    entity_id    TEXT PRIMARY KEY REFERENCES entity (id),
    climate      TEXT,
    terrain      TEXT,
    area_km2     REAL CHECK (area_km2 IS NULL OR area_km2 >= 0),
    elevation_m  REAL,
    status       TEXT CHECK (status IN
        ('thriving', 'declining', 'abandoned', 'ruined', 'destroyed', 'lost', 'hidden')),
    founded_at   TEXT REFERENCES instant (id),
    dissolved_at TEXT REFERENCES instant (id)
) STRICT;

-- Where a place sits in another: a city in a province, a province in a kingdom, a kingdom on a
-- continent. A place can sit in several at once (a geographic and a political parent) and in
-- different ones over time. Following one kind never leads back to where it started.
CREATE TABLE place_containment (
    id          TEXT PRIMARY KEY,
    child_id    TEXT NOT NULL REFERENCES entity (id),
    parent_id   TEXT NOT NULL REFERENCES entity (id),
    kind        TEXT NOT NULL REFERENCES containment_kind (key),
    time_state  TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at   TEXT REFERENCES instant (id),
    ends_at     TEXT REFERENCES instant (id),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (child_id <> parent_id),
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX place_containment_by_child ON place_containment (child_id, kind);
CREATE INDEX place_containment_by_parent ON place_containment (parent_id, kind);

CREATE TABLE place_border (
    id          TEXT PRIMARY KEY,
    first_id    TEXT NOT NULL REFERENCES entity (id),
    second_id   TEXT NOT NULL REFERENCES entity (id),
    kind        TEXT NOT NULL CHECK (kind IN
        ('land', 'river', 'sea', 'mountain', 'wall', 'magical', 'disputed')),
    length_km   REAL CHECK (length_km IS NULL OR length_km >= 0),
    time_state  TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at   TEXT REFERENCES instant (id),
    ends_at     TEXT REFERENCES instant (id),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (first_id < second_id),
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX place_border_by_first ON place_border (first_id);
CREATE INDEX place_border_by_second ON place_border (second_id);

CREATE TABLE place_resource (
    id        TEXT PRIMARY KEY,
    place_id  TEXT NOT NULL REFERENCES entity (id),
    resource  TEXT NOT NULL,
    abundance TEXT CHECK (abundance IN ('scarce', 'common', 'abundant', 'exhausted')),
    note      TEXT
) STRICT;

CREATE INDEX place_resource_by_place ON place_resource (place_id);

-- Population of a place at a moment, and where the figure comes from.
CREATE TABLE census (
    id          TEXT PRIMARY KEY,
    place_id    TEXT NOT NULL REFERENCES entity (id),
    taken_at    TEXT NOT NULL REFERENCES instant (id),
    population  INTEGER NOT NULL CHECK (population >= 0),
    is_estimate INTEGER NOT NULL CHECK (is_estimate IN (0, 1)),
    source_id   TEXT REFERENCES source (id),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
) STRICT;

CREATE INDEX census_by_place ON census (place_id, taken_at);

-- Kingdoms, countries, empires and provinces.
CREATE TABLE polity_profile (
    entity_id         TEXT PRIMARY KEY REFERENCES entity (id),
    government_type   TEXT,
    succession_rule   TEXT CHECK (succession_rule IN
        ('hereditary', 'elective', 'appointed', 'rotating', 'conquest', 'mixed')),
    currency          TEXT,
    motto             TEXT,
    capital_id        TEXT REFERENCES entity (id),
    state_religion_id TEXT REFERENCES entity (id)
) STRICT;

CREATE TABLE polity_language (
    polity_id   TEXT NOT NULL REFERENCES entity (id),
    language_id TEXT NOT NULL REFERENCES entity (id),
    status      TEXT NOT NULL CHECK (status IN ('official', 'regional', 'trade', 'liturgical')),
    PRIMARY KEY (polity_id, language_id)
) STRICT;

CREATE TABLE polity_symbol (
    id          TEXT PRIMARY KEY,
    polity_id   TEXT NOT NULL REFERENCES entity (id),
    kind        TEXT NOT NULL CHECK (kind IN
        ('flag', 'coat_of_arms', 'anthem', 'colors', 'seal', 'emblem')),
    description TEXT NOT NULL
) STRICT;

CREATE INDEX polity_symbol_by_polity ON polity_symbol (polity_id);

-- Who came before: a kingdom that split, a state that was conquered, an empire renamed.
CREATE TABLE polity_succession (
    id             TEXT PRIMARY KEY,
    predecessor_id TEXT NOT NULL REFERENCES entity (id),
    successor_id   TEXT NOT NULL REFERENCES entity (id),
    kind           TEXT NOT NULL CHECK (kind IN
        ('renaming', 'conquest', 'split', 'merger', 'revolution', 'dissolution', 'restoration')),
    occurred_at    TEXT REFERENCES instant (id),
    note           TEXT,
    canon_state    TEXT NOT NULL REFERENCES canon_state (key),
    origin         TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at     INTEGER NOT NULL,
    updated_at     INTEGER NOT NULL,
    CHECK (predecessor_id <> successor_id)
) STRICT;

CREATE INDEX polity_succession_by_predecessor ON polity_succession (predecessor_id);
CREATE INDEX polity_succession_by_successor ON polity_succession (successor_id);

-- A seat of power a polity or faction has: king, queen, council, high priest.
CREATE TABLE office (
    id          TEXT PRIMARY KEY,
    body_id     TEXT NOT NULL REFERENCES entity (id),
    title       TEXT NOT NULL,
    description TEXT NOT NULL,
    is_head     INTEGER NOT NULL CHECK (is_head IN (0, 1)),
    seats       INTEGER NOT NULL CHECK (seats >= 1),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
) STRICT;

CREATE INDEX office_by_body ON office (body_id);

CREATE TABLE office_tenure (
    id          TEXT PRIMARY KEY,
    office_id   TEXT NOT NULL REFERENCES office (id),
    holder_id   TEXT NOT NULL REFERENCES entity (id),
    acquisition TEXT REFERENCES office_acquisition (key),
    end_reason  TEXT CHECK (end_reason IN
        ('death', 'abdication', 'deposition', 'resignation', 'term_ended', 'exile', 'ongoing')),
    time_state  TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at   TEXT REFERENCES instant (id),
    ends_at     TEXT REFERENCES instant (id),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX office_tenure_by_office ON office_tenure (office_id);
CREATE INDEX office_tenure_by_holder ON office_tenure (holder_id);

CREATE TABLE citizenship (
    id          TEXT PRIMARY KEY,
    being_id    TEXT NOT NULL REFERENCES entity (id),
    polity_id   TEXT NOT NULL REFERENCES entity (id),
    status      TEXT NOT NULL REFERENCES citizenship_status (key),
    time_state  TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at   TEXT REFERENCES instant (id),
    ends_at     TEXT REFERENCES instant (id),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX citizenship_by_being ON citizenship (being_id);
CREATE INDEX citizenship_by_polity ON citizenship (polity_id);

-- Relations between two polities or factions. For a hierarchical stance (vassalage, protectorate)
-- `senior_id` says which side is the senior party.
CREATE TABLE diplomatic_relation (
    id          TEXT PRIMARY KEY,
    first_id    TEXT NOT NULL REFERENCES entity (id),
    second_id   TEXT NOT NULL REFERENCES entity (id),
    stance      TEXT NOT NULL REFERENCES diplomatic_stance (key),
    senior_id   TEXT REFERENCES entity (id),
    time_state  TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at   TEXT REFERENCES instant (id),
    ends_at     TEXT REFERENCES instant (id),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (first_id < second_id),
    CHECK (senior_id IS NULL OR senior_id IN (first_id, second_id)),
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX diplomatic_relation_by_first ON diplomatic_relation (first_id);
CREATE INDEX diplomatic_relation_by_second ON diplomatic_relation (second_id);

---------------------------------------------------------------------------------------------
-- 7. Organizations
---------------------------------------------------------------------------------------------

CREATE TABLE faction_profile (
    entity_id       TEXT PRIMARY KEY REFERENCES entity (id),
    ideology        TEXT,
    size_estimate   INTEGER CHECK (size_estimate IS NULL OR size_estimate >= 0),
    headquarters_id TEXT REFERENCES entity (id),
    founded_at      TEXT REFERENCES instant (id),
    dissolved_at    TEXT REFERENCES instant (id)
) STRICT;

CREATE TABLE faction_parent (
    id          TEXT PRIMARY KEY,
    child_id    TEXT NOT NULL REFERENCES entity (id),
    parent_id   TEXT NOT NULL REFERENCES entity (id),
    time_state  TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at   TEXT REFERENCES instant (id),
    ends_at     TEXT REFERENCES instant (id),
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (child_id <> parent_id),
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX faction_parent_by_child ON faction_parent (child_id);
CREATE INDEX faction_parent_by_parent ON faction_parent (parent_id);

CREATE TABLE membership (
    id          TEXT PRIMARY KEY,
    faction_id  TEXT NOT NULL REFERENCES entity (id),
    member_id   TEXT NOT NULL REFERENCES entity (id),
    role        TEXT,
    rank        TEXT,
    end_reason  TEXT CHECK (end_reason IN
        ('death', 'resignation', 'expulsion', 'defection', 'dissolution', 'ongoing')),
    time_state  TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at   TEXT REFERENCES instant (id),
    ends_at     TEXT REFERENCES instant (id),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (faction_id <> member_id),
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX membership_by_faction ON membership (faction_id);
CREATE INDEX membership_by_member ON membership (member_id);

---------------------------------------------------------------------------------------------
-- 8. Culture, language and belief
---------------------------------------------------------------------------------------------

CREATE TABLE culture_practice (
    id          TEXT PRIMARY KEY,
    culture_id  TEXT NOT NULL REFERENCES entity (id),
    kind        TEXT NOT NULL CHECK (kind IN
        ('custom', 'value', 'taboo', 'festival', 'cuisine', 'dress', 'art', 'law', 'ritual')),
    description TEXT NOT NULL,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
) STRICT;

CREATE INDEX culture_practice_by_culture ON culture_practice (culture_id, kind);

CREATE TABLE culture_language (
    culture_id  TEXT NOT NULL REFERENCES entity (id),
    language_id TEXT NOT NULL REFERENCES entity (id),
    PRIMARY KEY (culture_id, language_id)
) STRICT;

CREATE TABLE language_profile (
    entity_id        TEXT PRIMARY KEY REFERENCES entity (id),
    parent_language_id TEXT REFERENCES entity (id),
    writing_system   TEXT,
    speakers_estimate INTEGER CHECK (speakers_estimate IS NULL OR speakers_estimate >= 0),
    status           TEXT CHECK (status IN ('living', 'dead', 'ancient', 'ritual', 'constructed'))
) STRICT;

CREATE TABLE religion_profile (
    entity_id   TEXT PRIMARY KEY REFERENCES entity (id),
    founder_id  TEXT REFERENCES entity (id),
    founded_at  TEXT REFERENCES instant (id),
    is_monotheistic INTEGER CHECK (is_monotheistic IN (0, 1))
) STRICT;

CREATE TABLE religion_tenet (
    id          TEXT PRIMARY KEY,
    religion_id TEXT NOT NULL REFERENCES entity (id),
    kind        TEXT NOT NULL CHECK (kind IN
        ('tenet', 'practice', 'taboo', 'festival', 'sacrament', 'prophecy')),
    description TEXT NOT NULL,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
) STRICT;

CREATE INDEX religion_tenet_by_religion ON religion_tenet (religion_id, kind);

CREATE TABLE religion_deity (
    religion_id TEXT NOT NULL REFERENCES entity (id),
    deity_id    TEXT NOT NULL REFERENCES entity (id),
    role        TEXT,
    PRIMARY KEY (religion_id, deity_id)
) STRICT;

CREATE TABLE holy_place (
    religion_id TEXT NOT NULL REFERENCES entity (id),
    place_id    TEXT NOT NULL REFERENCES entity (id),
    significance TEXT,
    PRIMARY KEY (religion_id, place_id)
) STRICT;

-- A being, culture or polity following a religion over a period.
CREATE TABLE adherence (
    id          TEXT PRIMARY KEY,
    adherent_id TEXT NOT NULL REFERENCES entity (id),
    religion_id TEXT NOT NULL REFERENCES entity (id),
    devotion    TEXT CHECK (devotion IN ('nominal', 'observant', 'devout', 'zealous', 'clergy')),
    time_state  TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at   TEXT REFERENCES instant (id),
    ends_at     TEXT REFERENCES instant (id),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX adherence_by_adherent ON adherence (adherent_id);
CREATE INDEX adherence_by_religion ON adherence (religion_id);

---------------------------------------------------------------------------------------------
-- 9. Items
---------------------------------------------------------------------------------------------

CREATE TABLE item_profile (
    entity_id    TEXT PRIMARY KEY REFERENCES entity (id),
    material     TEXT,
    rarity       TEXT CHECK (rarity IN
        ('mundane', 'common', 'uncommon', 'rare', 'legendary', 'unique')),
    creator_id   TEXT REFERENCES entity (id),
    made_at      TEXT REFERENCES instant (id),
    destroyed_at TEXT REFERENCES instant (id)
) STRICT;

CREATE TABLE item_power (
    id          TEXT PRIMARY KEY,
    item_id     TEXT NOT NULL REFERENCES entity (id),
    description TEXT NOT NULL,
    cost        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
) STRICT;

CREATE INDEX item_power_by_item ON item_power (item_id);

CREATE TABLE item_ownership (
    id          TEXT PRIMARY KEY,
    item_id     TEXT NOT NULL REFERENCES entity (id),
    owner_id    TEXT NOT NULL REFERENCES entity (id),
    acquisition TEXT REFERENCES item_acquisition (key),
    time_state  TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at   TEXT REFERENCES instant (id),
    ends_at     TEXT REFERENCES instant (id),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX item_ownership_by_item ON item_ownership (item_id);
CREATE INDEX item_ownership_by_owner ON item_ownership (owner_id);

---------------------------------------------------------------------------------------------
-- 10. Events
---------------------------------------------------------------------------------------------

CREATE TABLE event_detail (
    entity_id    TEXT PRIMARY KEY REFERENCES entity (id),
    time_state   TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at    TEXT REFERENCES instant (id),
    ends_at      TEXT REFERENCES instant (id),
    location_id  TEXT REFERENCES entity (id),
    scale        TEXT CHECK (scale IN ('personal', 'local', 'regional', 'national', 'world')),
    consequences TEXT NOT NULL,
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX event_detail_by_location ON event_detail (location_id);

CREATE TABLE event_participant (
    event_id       TEXT NOT NULL REFERENCES event_detail (entity_id),
    participant_id TEXT NOT NULL REFERENCES entity (id),
    role           TEXT NOT NULL,
    side           TEXT,
    position       INTEGER NOT NULL,
    PRIMARY KEY (event_id, participant_id, role)
) STRICT;

CREATE INDEX event_participant_by_participant ON event_participant (participant_id);

CREATE TABLE event_causality (
    id          TEXT PRIMARY KEY,
    cause_id    TEXT NOT NULL REFERENCES entity (id),
    effect_id   TEXT NOT NULL REFERENCES entity (id),
    kind        TEXT NOT NULL CHECK (kind IN
        ('direct', 'contributing', 'enabling', 'prevented', 'triggered')),
    note        TEXT,
    canon_state TEXT NOT NULL REFERENCES canon_state (key),
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (cause_id <> effect_id)
) STRICT;

CREATE INDEX event_causality_by_cause ON event_causality (cause_id);
CREATE INDEX event_causality_by_effect ON event_causality (effect_id);

---------------------------------------------------------------------------------------------
-- 11. Narrative: appearances and the editorial structure
---------------------------------------------------------------------------------------------

CREATE TABLE appearance (
    id           TEXT PRIMARY KEY,
    entity_id    TEXT NOT NULL REFERENCES entity (id),
    context_id   TEXT NOT NULL REFERENCES entity (id),
    kind         TEXT NOT NULL REFERENCES appearance_kind (key),
    position     INTEGER NOT NULL,
    viewpoint_id TEXT REFERENCES entity (id),
    note         TEXT,
    canon_state  TEXT NOT NULL REFERENCES canon_state (key),
    origin       TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL
) STRICT;

CREATE INDEX appearance_by_entity ON appearance (entity_id);
CREATE INDEX appearance_by_context ON appearance (context_id, position);
CREATE INDEX appearance_by_viewpoint ON appearance (viewpoint_id);

CREATE TABLE editorial_node (
    entity_id TEXT PRIMARY KEY REFERENCES entity (id),
    level     TEXT NOT NULL REFERENCES editorial_level (key),
    parent_id TEXT REFERENCES editorial_node (entity_id),
    position  INTEGER NOT NULL
) STRICT;

CREATE INDEX editorial_node_by_parent ON editorial_node (parent_id, position);

---------------------------------------------------------------------------------------------
-- 12. Generic relationships and project-specific properties
---------------------------------------------------------------------------------------------

-- Connections that have no table of their own above.
CREATE TABLE relationship (
    id                TEXT PRIMARY KEY,
    relationship_type TEXT NOT NULL REFERENCES relationship_type (key),
    source_id         TEXT NOT NULL REFERENCES entity (id),
    target_id         TEXT NOT NULL REFERENCES entity (id),
    qualifier         TEXT,
    context_id        TEXT REFERENCES entity (id),
    lifecycle         TEXT NOT NULL CHECK (lifecycle IN ('active', 'historical')),
    time_state        TEXT NOT NULL CHECK (time_state IN ('unknown', 'unconstrained', 'known')),
    starts_at         TEXT REFERENCES instant (id),
    ends_at           TEXT REFERENCES instant (id),
    note              TEXT,
    canon_state       TEXT NOT NULL REFERENCES canon_state (key),
    origin            TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at        INTEGER NOT NULL,
    updated_at        INTEGER NOT NULL,
    CHECK (time_state = 'known' OR (starts_at IS NULL AND ends_at IS NULL))
) STRICT;

CREATE INDEX relationship_by_source ON relationship (source_id);
CREATE INDEX relationship_by_target ON relationship (target_id);
CREATE INDEX relationship_by_type ON relationship (relationship_type);
CREATE INDEX relationship_by_context ON relationship (context_id);

CREATE TABLE property_group (
    key        TEXT PRIMARY KEY,
    label      TEXT NOT NULL,
    sort_order INTEGER NOT NULL
) STRICT;

CREATE TABLE property_definition (
    id                    TEXT PRIMARY KEY,
    entity_type           TEXT REFERENCES entity_type (key),
    relationship_type     TEXT REFERENCES relationship_type (key),
    key                   TEXT NOT NULL,
    label                 TEXT NOT NULL,
    description           TEXT NOT NULL,
    group_key             TEXT REFERENCES property_group (key),
    data_type             TEXT NOT NULL CHECK (data_type IN
        ('text', 'integer', 'real', 'boolean', 'date', 'entity_reference', 'choice')),
    reference_entity_type TEXT REFERENCES entity_type (key),
    required              INTEGER NOT NULL CHECK (required IN (0, 1)),
    multiple              INTEGER NOT NULL CHECK (multiple IN (0, 1)),
    sort_order            INTEGER NOT NULL,
    builtin               INTEGER NOT NULL CHECK (builtin IN (0, 1)),
    CHECK ((entity_type IS NULL) <> (relationship_type IS NULL)),
    CHECK (reference_entity_type IS NULL OR data_type = 'entity_reference')
) STRICT;

CREATE UNIQUE INDEX property_definition_key_in_entity_type
    ON property_definition (entity_type, key) WHERE entity_type IS NOT NULL;
CREATE UNIQUE INDEX property_definition_key_in_relationship_type
    ON property_definition (relationship_type, key) WHERE relationship_type IS NOT NULL;

CREATE TABLE property_choice_option (
    definition_id TEXT NOT NULL REFERENCES property_definition (id),
    key           TEXT NOT NULL,
    label         TEXT NOT NULL,
    position      INTEGER NOT NULL,
    PRIMARY KEY (definition_id, key)
) STRICT;

-- One recorded value. The typed columns are mutually exclusive; the one that applies is the
-- definition's data_type.
CREATE TABLE property_value (
    id                   TEXT PRIMARY KEY,
    entity_id            TEXT REFERENCES entity (id),
    relationship_id      TEXT REFERENCES relationship (id),
    definition_id        TEXT NOT NULL REFERENCES property_definition (id),
    value_text           TEXT,
    value_integer        INTEGER,
    value_real           REAL,
    value_boolean        INTEGER CHECK (value_boolean IN (0, 1)),
    value_instant        TEXT REFERENCES instant (id),
    value_entity_id      TEXT REFERENCES entity (id),
    value_choice         TEXT,
    valid_from           TEXT REFERENCES instant (id),
    valid_until          TEXT REFERENCES instant (id),
    note                 TEXT,
    position             INTEGER NOT NULL,
    canon_state          TEXT NOT NULL REFERENCES canon_state (key),
    origin               TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    created_at           INTEGER NOT NULL,
    updated_at           INTEGER NOT NULL,
    CHECK ((entity_id IS NULL) <> (relationship_id IS NULL)),
    CHECK ((value_text IS NOT NULL) + (value_integer IS NOT NULL) + (value_real IS NOT NULL)
         + (value_boolean IS NOT NULL) + (value_instant IS NOT NULL)
         + (value_entity_id IS NOT NULL) + (value_choice IS NOT NULL) = 1)
) STRICT;

CREATE INDEX property_value_by_entity ON property_value (entity_id, definition_id);
CREATE INDEX property_value_by_relationship ON property_value (relationship_id, definition_id);
CREATE INDEX property_value_by_value_entity ON property_value (value_entity_id);

---------------------------------------------------------------------------------------------
-- 13. Assets, sources and canon history
---------------------------------------------------------------------------------------------

CREATE TABLE asset_file (
    entity_id     TEXT PRIMARY KEY REFERENCES entity (id),
    relative_path TEXT NOT NULL UNIQUE,
    media_type    TEXT,
    size_bytes    INTEGER NOT NULL,
    content_hash  TEXT,
    imported_at   INTEGER NOT NULL
) STRICT;

CREATE TABLE attachment (
    id         TEXT PRIMARY KEY,
    owner_id   TEXT NOT NULL REFERENCES entity (id),
    asset_id   TEXT NOT NULL REFERENCES entity (id),
    purpose    TEXT,
    position   INTEGER NOT NULL,
    note       TEXT,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX attachment_by_owner ON attachment (owner_id, position);
CREATE INDEX attachment_by_asset ON attachment (asset_id);

CREATE TABLE source (
    id         TEXT PRIMARY KEY,
    kind       TEXT NOT NULL CHECK (kind IN ('manuscript', 'document', 'external', 'session')),
    title      TEXT NOT NULL,
    locator    TEXT,
    note       TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

-- A citation of a source for one record of any fact table. The target table is named, so the
-- reference is explicit even though it cannot be a foreign key.
CREATE TABLE source_reference (
    id           TEXT PRIMARY KEY,
    source_id    TEXT NOT NULL REFERENCES source (id),
    target_table TEXT NOT NULL,
    target_id    TEXT NOT NULL,
    locator      TEXT,
    quote        TEXT,
    created_at   INTEGER NOT NULL
) STRICT;

CREATE INDEX source_reference_by_target ON source_reference (target_table, target_id);
CREATE INDEX source_reference_by_source ON source_reference (source_id);

CREATE TABLE canon_state_change (
    id           TEXT PRIMARY KEY,
    target_table TEXT NOT NULL,
    target_id    TEXT NOT NULL,
    from_state   TEXT NOT NULL REFERENCES canon_state (key),
    to_state     TEXT NOT NULL REFERENCES canon_state (key),
    reason       TEXT,
    origin       TEXT NOT NULL CHECK (origin IN ('user', 'ai', 'mcp', 'import')),
    changed_at   INTEGER NOT NULL
) STRICT;

CREATE INDEX canon_state_change_by_target
    ON canon_state_change (target_table, target_id, changed_at);
