// Which fields each section of a dossier has. The record types are in `ontara-core`; the order
// here is the order the sections and their fields are shown in.

import type { Vocabulary } from '@/features/catalog';

/** The sets of words a choice field offers. Their texts are in `dossierChoices`. */
export type ChoiceGroup =
  | 'proficiency'
  | 'partnership_end_reason'
  | 'border_kind'
  | 'place_status'
  | 'resource_abundance'
  | 'tenure_end_reason'
  | 'polity_language_status'
  | 'succession_rule'
  | 'polity_succession_kind'
  | 'polity_symbol_kind'
  | 'membership_end_reason'
  | 'devotion'
  | 'culture_practice_kind'
  | 'language_status'
  | 'tenet_kind'
  | 'item_rarity';

export type FieldKind =
  | 'text'
  | 'longtext'
  | 'integer'
  | 'number'
  /** Yes or no; when optional, also "not said". */
  | 'flag'
  | 'choice'
  | 'term'
  | 'entity'
  | 'instant'
  | 'period'
  | 'canon'
  | 'stance'
  /** One of the offices of the body the dossier belongs to. */
  | 'office';

export interface FieldDef {
  /** The name of the field in the record, which is also the key of its label. */
  name: string;
  kind: FieldKind;
  /** The record cannot be saved without it. */
  required?: boolean;
  /** For `choice`: which set of words. */
  group?: ChoiceGroup;
  /** For `term`: which vocabulary. */
  vocabulary?: Vocabulary;
  /** For `entity`: the kinds of entity it may hold; empty means any. */
  classes?: readonly string[];
  min?: number;
  max?: number;
}

/** How the records of a section relate to the entity. Mirrors `Shape` in `src-tauri/src/dossier.rs`. */
export type SectionShape = 'profile' | 'items' | 'links';

export interface SectionDef {
  key: string;
  shape: SectionShape;
  /** Fields that hold an entity the record is about, with the kinds each may hold. */
  owners: readonly { field: string; classes: readonly string[] }[];
  /** The fields that name a record in a list, in order. */
  title: readonly string[];
  /** The fields that describe it further. */
  detail: readonly string[];
  fields: readonly FieldDef[];
}

export const SECTIONS: readonly SectionDef[] = [
  {
    key: 'being_profile',
    shape: 'profile',
    owners: [{ field: 'entity', classes: ['being'] }],
    title: [],
    detail: [],
    fields: [
      { name: 'gender', kind: 'text' },
      { name: 'pronouns', kind: 'text' },
      { name: 'species', kind: 'entity', classes: ['species'] },
      { name: 'birth', kind: 'instant' },
      { name: 'death', kind: 'instant' },
      { name: 'birthplace', kind: 'entity', classes: ['place'] },
      { name: 'upbringing', kind: 'text' },
      { name: 'occupation', kind: 'text' },
    ],
  },
  {
    key: 'being_appearance',
    shape: 'items',
    owners: [{ field: 'being', classes: ['being'] }],
    title: ['label'],
    detail: ['build', 'period'],
    fields: [
      { name: 'label', kind: 'text' },
      { name: 'hair_color', kind: 'text' },
      { name: 'hair_style', kind: 'text' },
      { name: 'eye_color', kind: 'text' },
      { name: 'skin_tone', kind: 'text' },
      { name: 'height_cm', kind: 'number' },
      { name: 'weight_kg', kind: 'number' },
      { name: 'build', kind: 'text' },
      { name: 'voice', kind: 'text' },
      { name: 'clothing_style', kind: 'text' },
      { name: 'period', kind: 'period' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'being_trait',
    shape: 'items',
    owners: [{ field: 'being', classes: ['being'] }],
    title: ['text'],
    detail: ['kind', 'intensity', 'period'],
    fields: [
      { name: 'kind', kind: 'term', vocabulary: 'trait_kind', required: true },
      { name: 'text', kind: 'text', required: true },
      { name: 'intensity', kind: 'integer', min: 1, max: 5 },
      { name: 'period', kind: 'period' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'being_mark',
    shape: 'items',
    owners: [{ field: 'being', classes: ['being'] }],
    title: ['description'],
    detail: ['body_location', 'acquired'],
    fields: [
      { name: 'description', kind: 'text', required: true },
      { name: 'body_location', kind: 'text' },
      { name: 'acquired', kind: 'instant' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'being_language',
    shape: 'links',
    owners: [
      { field: 'being', classes: ['being'] },
      { field: 'language', classes: ['language'] },
    ],
    title: ['being', 'language'],
    detail: ['proficiency'],
    fields: [
      { name: 'being', kind: 'entity', classes: ['being'], required: true },
      { name: 'language', kind: 'entity', classes: ['language'], required: true },
      { name: 'proficiency', kind: 'choice', group: 'proficiency', required: true },
      { name: 'note', kind: 'longtext' },
    ],
  },
  {
    key: 'family_tie',
    shape: 'items',
    owners: [
      { field: 'parent', classes: ['being'] },
      { field: 'child', classes: ['being'] },
    ],
    title: ['parent', 'child'],
    detail: ['kind', 'period'],
    fields: [
      { name: 'parent', kind: 'entity', classes: ['being'], required: true },
      { name: 'child', kind: 'entity', classes: ['being'], required: true },
      { name: 'kind', kind: 'term', vocabulary: 'family_tie_kind', required: true },
      { name: 'period', kind: 'period' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'partnership',
    shape: 'items',
    owners: [
      { field: 'first', classes: ['being'] },
      { field: 'second', classes: ['being'] },
    ],
    title: ['first', 'second'],
    detail: ['kind', 'end_reason', 'period'],
    fields: [
      { name: 'first', kind: 'entity', classes: ['being'], required: true },
      { name: 'second', kind: 'entity', classes: ['being'], required: true },
      { name: 'kind', kind: 'term', vocabulary: 'partnership_kind', required: true },
      { name: 'end_reason', kind: 'choice', group: 'partnership_end_reason' },
      { name: 'period', kind: 'period' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'citizenship',
    shape: 'items',
    owners: [
      { field: 'being', classes: ['being'] },
      { field: 'polity', classes: ['polity'] },
    ],
    title: ['being', 'polity'],
    detail: ['status', 'period'],
    fields: [
      { name: 'being', kind: 'entity', classes: ['being'], required: true },
      { name: 'polity', kind: 'entity', classes: ['polity'], required: true },
      { name: 'status', kind: 'term', vocabulary: 'citizenship_status', required: true },
      { name: 'period', kind: 'period' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'office_tenure',
    shape: 'items',
    owners: [{ field: 'holder', classes: ['being'] }],
    title: ['holder', 'office'],
    detail: ['end_reason', 'period'],
    fields: [
      { name: 'office', kind: 'office', required: true },
      { name: 'holder', kind: 'entity', classes: ['being'], required: true },
      { name: 'acquisition', kind: 'term', vocabulary: 'office_acquisition' },
      { name: 'end_reason', kind: 'choice', group: 'tenure_end_reason' },
      { name: 'period', kind: 'period' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'deity_profile',
    shape: 'profile',
    owners: [{ field: 'entity', classes: ['deity'] }],
    title: [],
    detail: [],
    fields: [
      { name: 'rank', kind: 'text' },
      { name: 'alignment', kind: 'text' },
    ],
  },
  {
    key: 'deity_domain',
    shape: 'items',
    owners: [{ field: 'deity', classes: ['deity'] }],
    title: ['domain'],
    detail: [],
    fields: [
      { name: 'domain', kind: 'text', required: true },
      { name: 'note', kind: 'longtext' },
    ],
  },
  {
    key: 'species_profile',
    shape: 'profile',
    owners: [{ field: 'entity', classes: ['species'] }],
    title: [],
    detail: [],
    fields: [
      { name: 'is_sapient', kind: 'flag' },
      { name: 'lifespan_min_years', kind: 'integer' },
      { name: 'lifespan_max_years', kind: 'integer' },
      { name: 'average_height_cm', kind: 'number' },
      { name: 'diet', kind: 'text' },
    ],
  },
  {
    key: 'species_habitat',
    shape: 'links',
    owners: [
      { field: 'species', classes: ['species'] },
      { field: 'place', classes: ['place'] },
    ],
    title: ['species', 'place'],
    detail: [],
    fields: [
      { name: 'species', kind: 'entity', classes: ['species'], required: true },
      { name: 'place', kind: 'entity', classes: ['place'], required: true },
      { name: 'note', kind: 'longtext' },
    ],
  },
  {
    key: 'place_profile',
    shape: 'profile',
    owners: [{ field: 'entity', classes: ['place'] }],
    title: [],
    detail: [],
    fields: [
      { name: 'climate', kind: 'text' },
      { name: 'terrain', kind: 'text' },
      { name: 'area_km2', kind: 'number' },
      { name: 'elevation_m', kind: 'number' },
      { name: 'status', kind: 'choice', group: 'place_status' },
      { name: 'founded', kind: 'instant' },
      { name: 'dissolved', kind: 'instant' },
    ],
  },
  {
    key: 'place_containment',
    shape: 'items',
    owners: [
      { field: 'child', classes: ['place'] },
      { field: 'parent', classes: ['place'] },
    ],
    title: ['child', 'parent'],
    detail: ['kind', 'period'],
    fields: [
      { name: 'child', kind: 'entity', classes: ['place'], required: true },
      { name: 'parent', kind: 'entity', classes: ['place'], required: true },
      { name: 'kind', kind: 'term', vocabulary: 'containment_kind', required: true },
      { name: 'period', kind: 'period' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'place_border',
    shape: 'items',
    owners: [
      { field: 'first', classes: ['place'] },
      { field: 'second', classes: ['place'] },
    ],
    title: ['first', 'second'],
    detail: ['kind', 'length_km', 'period'],
    fields: [
      { name: 'first', kind: 'entity', classes: ['place'], required: true },
      { name: 'second', kind: 'entity', classes: ['place'], required: true },
      { name: 'kind', kind: 'choice', group: 'border_kind', required: true },
      { name: 'length_km', kind: 'number' },
      { name: 'period', kind: 'period' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'place_resource',
    shape: 'items',
    owners: [{ field: 'place', classes: ['place'] }],
    title: ['resource'],
    detail: ['abundance'],
    fields: [
      { name: 'resource', kind: 'text', required: true },
      { name: 'abundance', kind: 'choice', group: 'resource_abundance' },
      { name: 'note', kind: 'longtext' },
    ],
  },
  {
    key: 'census',
    shape: 'items',
    owners: [{ field: 'place', classes: ['place'] }],
    title: ['population'],
    detail: ['taken', 'is_estimate'],
    fields: [
      { name: 'taken', kind: 'instant', required: true },
      { name: 'population', kind: 'integer', required: true },
      { name: 'is_estimate', kind: 'flag', required: true },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'polity_profile',
    shape: 'profile',
    owners: [{ field: 'entity', classes: ['polity'] }],
    title: [],
    detail: [],
    fields: [
      { name: 'government_type', kind: 'text' },
      { name: 'succession_rule', kind: 'choice', group: 'succession_rule' },
      { name: 'currency', kind: 'text' },
      { name: 'motto', kind: 'text' },
      { name: 'capital', kind: 'entity', classes: ['place'] },
      { name: 'state_religion', kind: 'entity', classes: ['religion'] },
    ],
  },
  {
    key: 'polity_language',
    shape: 'links',
    owners: [
      { field: 'polity', classes: ['polity'] },
      { field: 'language', classes: ['language'] },
    ],
    title: ['polity', 'language'],
    detail: ['status'],
    fields: [
      { name: 'polity', kind: 'entity', classes: ['polity'], required: true },
      { name: 'language', kind: 'entity', classes: ['language'], required: true },
      { name: 'status', kind: 'choice', group: 'polity_language_status', required: true },
    ],
  },
  {
    key: 'polity_symbol',
    shape: 'items',
    owners: [{ field: 'polity', classes: ['polity'] }],
    title: ['description'],
    detail: ['kind'],
    fields: [
      { name: 'kind', kind: 'choice', group: 'polity_symbol_kind', required: true },
      { name: 'description', kind: 'text', required: true },
    ],
  },
  {
    key: 'polity_succession',
    shape: 'items',
    owners: [
      { field: 'predecessor', classes: ['polity'] },
      { field: 'successor', classes: ['polity'] },
    ],
    title: ['predecessor', 'successor'],
    detail: ['kind', 'occurred'],
    fields: [
      { name: 'predecessor', kind: 'entity', classes: ['polity'], required: true },
      { name: 'successor', kind: 'entity', classes: ['polity'], required: true },
      { name: 'kind', kind: 'choice', group: 'polity_succession_kind', required: true },
      { name: 'occurred', kind: 'instant' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'office',
    shape: 'items',
    owners: [{ field: 'body', classes: ['polity', 'faction'] }],
    title: ['title'],
    detail: ['is_head', 'seats'],
    fields: [
      { name: 'title', kind: 'text', required: true },
      { name: 'description', kind: 'longtext' },
      { name: 'is_head', kind: 'flag', required: true },
      { name: 'seats', kind: 'integer', required: true },
    ],
  },
  {
    key: 'diplomatic_relation',
    shape: 'items',
    owners: [
      { field: 'first', classes: ['polity', 'faction'] },
      { field: 'second', classes: ['polity', 'faction'] },
    ],
    title: ['first', 'second'],
    detail: ['stance', 'period'],
    fields: [
      { name: 'first', kind: 'entity', classes: ['polity', 'faction'], required: true },
      { name: 'second', kind: 'entity', classes: ['polity', 'faction'], required: true },
      { name: 'stance', kind: 'stance', required: true },
      { name: 'senior', kind: 'entity', classes: ['polity', 'faction'] },
      { name: 'period', kind: 'period' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'faction_profile',
    shape: 'profile',
    owners: [{ field: 'entity', classes: ['faction'] }],
    title: [],
    detail: [],
    fields: [
      { name: 'ideology', kind: 'text' },
      { name: 'size_estimate', kind: 'integer' },
      { name: 'headquarters', kind: 'entity', classes: ['place'] },
      { name: 'founded', kind: 'instant' },
      { name: 'dissolved', kind: 'instant' },
    ],
  },
  {
    key: 'faction_parent',
    shape: 'items',
    owners: [
      { field: 'child', classes: ['faction'] },
      { field: 'parent', classes: ['faction'] },
    ],
    title: ['child', 'parent'],
    detail: ['period'],
    fields: [
      { name: 'child', kind: 'entity', classes: ['faction'], required: true },
      { name: 'parent', kind: 'entity', classes: ['faction'], required: true },
      { name: 'period', kind: 'period' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'membership',
    shape: 'items',
    owners: [
      { field: 'faction', classes: ['faction'] },
      { field: 'member', classes: ['being', 'faction', 'polity', 'culture'] },
    ],
    title: ['member', 'faction'],
    detail: ['role', 'rank', 'period'],
    fields: [
      { name: 'faction', kind: 'entity', classes: ['faction'], required: true },
      { name: 'member', kind: 'entity', classes: [], required: true },
      { name: 'role', kind: 'text' },
      { name: 'rank', kind: 'text' },
      { name: 'end_reason', kind: 'choice', group: 'membership_end_reason' },
      { name: 'period', kind: 'period' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'language_profile',
    shape: 'profile',
    owners: [{ field: 'entity', classes: ['language'] }],
    title: [],
    detail: [],
    fields: [
      { name: 'parent_language', kind: 'entity', classes: ['language'] },
      { name: 'writing_system', kind: 'text' },
      { name: 'speakers_estimate', kind: 'integer' },
      { name: 'status', kind: 'choice', group: 'language_status' },
    ],
  },
  {
    key: 'religion_profile',
    shape: 'profile',
    owners: [{ field: 'entity', classes: ['religion'] }],
    title: [],
    detail: [],
    fields: [
      { name: 'founder', kind: 'entity', classes: ['being'] },
      { name: 'founded', kind: 'instant' },
      { name: 'is_monotheistic', kind: 'flag' },
    ],
  },
  {
    key: 'religion_tenet',
    shape: 'items',
    owners: [{ field: 'religion', classes: ['religion'] }],
    title: ['description'],
    detail: ['kind'],
    fields: [
      { name: 'kind', kind: 'choice', group: 'tenet_kind', required: true },
      { name: 'description', kind: 'text', required: true },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'religion_deity',
    shape: 'links',
    owners: [
      { field: 'religion', classes: ['religion'] },
      { field: 'deity', classes: ['deity'] },
    ],
    title: ['religion', 'deity'],
    detail: ['role'],
    fields: [
      { name: 'religion', kind: 'entity', classes: ['religion'], required: true },
      { name: 'deity', kind: 'entity', classes: ['deity'], required: true },
      { name: 'role', kind: 'text' },
    ],
  },
  {
    key: 'holy_place',
    shape: 'links',
    owners: [
      { field: 'religion', classes: ['religion'] },
      { field: 'place', classes: ['place'] },
    ],
    title: ['religion', 'place'],
    detail: ['significance'],
    fields: [
      { name: 'religion', kind: 'entity', classes: ['religion'], required: true },
      { name: 'place', kind: 'entity', classes: ['place'], required: true },
      { name: 'significance', kind: 'text' },
    ],
  },
  {
    key: 'adherence',
    shape: 'items',
    owners: [
      { field: 'adherent', classes: ['being', 'faction', 'polity', 'culture'] },
      { field: 'religion', classes: ['religion'] },
    ],
    title: ['adherent', 'religion'],
    detail: ['devotion', 'period'],
    fields: [
      { name: 'adherent', kind: 'entity', classes: [], required: true },
      { name: 'religion', kind: 'entity', classes: ['religion'], required: true },
      { name: 'devotion', kind: 'choice', group: 'devotion' },
      { name: 'period', kind: 'period' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'culture_practice',
    shape: 'items',
    owners: [{ field: 'culture', classes: ['culture'] }],
    title: ['description'],
    detail: ['kind'],
    fields: [
      { name: 'kind', kind: 'choice', group: 'culture_practice_kind', required: true },
      { name: 'description', kind: 'text', required: true },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'culture_language',
    shape: 'links',
    owners: [
      { field: 'culture', classes: ['culture'] },
      { field: 'language', classes: ['language'] },
    ],
    title: ['culture', 'language'],
    detail: [],
    fields: [
      { name: 'culture', kind: 'entity', classes: ['culture'], required: true },
      { name: 'language', kind: 'entity', classes: ['language'], required: true },
    ],
  },
  {
    key: 'item_profile',
    shape: 'profile',
    owners: [{ field: 'entity', classes: ['item'] }],
    title: [],
    detail: [],
    fields: [
      { name: 'material', kind: 'text' },
      { name: 'rarity', kind: 'choice', group: 'item_rarity' },
      { name: 'creator', kind: 'entity', classes: [] },
      { name: 'made', kind: 'instant' },
      { name: 'destroyed', kind: 'instant' },
    ],
  },
  {
    key: 'item_power',
    shape: 'items',
    owners: [{ field: 'item', classes: ['item'] }],
    title: ['description'],
    detail: ['cost'],
    fields: [
      { name: 'description', kind: 'text', required: true },
      { name: 'cost', kind: 'text' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
  {
    key: 'item_ownership',
    shape: 'items',
    owners: [
      { field: 'item', classes: ['item'] },
      { field: 'owner', classes: ['being', 'faction', 'polity', 'culture'] },
    ],
    title: ['owner', 'item'],
    detail: ['acquisition', 'period'],
    fields: [
      { name: 'item', kind: 'entity', classes: ['item'], required: true },
      { name: 'owner', kind: 'entity', classes: [], required: true },
      { name: 'acquisition', kind: 'term', vocabulary: 'item_acquisition' },
      { name: 'period', kind: 'period' },
      { name: 'note', kind: 'longtext' },
      { name: 'canon_state', kind: 'canon' },
    ],
  },
];

/** Every set of words a choice field offers, in the order they are listed. */
export const CHOICES: Readonly<Record<ChoiceGroup, readonly string[]>> = {
  proficiency: ['native', 'fluent', 'conversational', 'basic', 'reads_only'],
  partnership_end_reason: ['death', 'divorce', 'annulment', 'separation', 'broken', 'ongoing'],
  border_kind: ['land', 'river', 'sea', 'mountain', 'wall', 'magical', 'disputed'],
  place_status: ['thriving', 'declining', 'abandoned', 'ruined', 'destroyed', 'lost', 'hidden'],
  resource_abundance: ['scarce', 'common', 'abundant', 'exhausted'],
  tenure_end_reason: [
    'death',
    'abdication',
    'deposition',
    'resignation',
    'term_ended',
    'exile',
    'ongoing',
  ],
  polity_language_status: ['official', 'regional', 'trade', 'liturgical'],
  succession_rule: ['hereditary', 'elective', 'appointed', 'rotating', 'conquest', 'mixed'],
  polity_succession_kind: [
    'renaming',
    'conquest',
    'split',
    'merger',
    'revolution',
    'dissolution',
    'restoration',
  ],
  polity_symbol_kind: ['flag', 'coat_of_arms', 'anthem', 'colors', 'seal', 'emblem'],
  membership_end_reason: [
    'death',
    'resignation',
    'expulsion',
    'defection',
    'dissolution',
    'ongoing',
  ],
  devotion: ['nominal', 'observant', 'devout', 'zealous', 'clergy'],
  culture_practice_kind: [
    'custom',
    'value',
    'taboo',
    'festival',
    'cuisine',
    'dress',
    'art',
    'law',
    'ritual',
  ],
  language_status: ['living', 'dead', 'ancient', 'ritual', 'constructed'],
  tenet_kind: ['tenet', 'practice', 'taboo', 'festival', 'sacrament', 'prophecy'],
  item_rarity: ['mundane', 'common', 'uncommon', 'rare', 'legendary', 'unique'],
};
