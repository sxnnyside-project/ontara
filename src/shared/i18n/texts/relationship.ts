import type { TextGroup } from '../define';

/** Typed relationships between entities. */
export const relationshipTexts = {
  relationship: {
    add: { en: 'Add relationship', es: 'Añadir relación' },
    addTitle: { en: 'New relationship', es: 'Nueva relación' },
    editTitle: { en: 'Edit relationship', es: 'Editar relación' },
    from: { en: 'From {name}', es: 'Desde {name}' },
    type: { en: 'Relationship', es: 'Relación' },
    chooseType: { en: 'Choose a relationship…', es: 'Elige una relación…' },
    searchTarget: { en: 'Search the entity to connect', es: 'Busca la entidad a conectar' },
    target: { en: 'To', es: 'Hacia' },
    source: { en: 'From', es: 'Desde' },
    chooseEntity: { en: 'Choose an entity…', es: 'Elige una entidad…' },
    qualifier: { en: 'Qualifier', es: 'Matiz' },
    qualifierHint: {
      en: 'A short detail, such as "in swordsmanship".',
      es: 'Un detalle breve, como "en esgrima".',
    },
    lifecycle: { en: 'Status', es: 'Estado' },
    active: { en: 'Active', es: 'Activa' },
    historical: { en: 'Historical', es: 'Histórica' },
    period: { en: 'When it holds', es: 'Cuándo se cumple' },
    note: { en: 'Note', es: 'Nota' },
    properties: { en: 'Properties of this relationship', es: 'Propiedades de esta relación' },
    inContext: { en: 'in {name}', es: 'en {name}' },
    noneTitle: { en: 'No relationships', es: 'Sin relaciones' },
    noneHint: {
      en: 'Connect this entity to others with typed, directed relationships.',
      es: 'Conecta esta entidad con otras mediante relaciones tipadas y dirigidas.',
    },
    removeTitle: { en: 'Remove relationship?', es: '¿Quitar la relación?' },
    removeBody: {
      en: 'The relationship and its properties are removed. The entities are kept.',
      es: 'Se quitan la relación y sus propiedades. Las entidades se conservan.',
    },
  },
} satisfies TextGroup;
