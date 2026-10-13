import type { TextGroup } from '../define';

/** The appearance matrix. */
export const matrixTexts = {
  matrixView: {
    title: { en: 'Appearances', es: 'Apariciones' },
    subtitle: {
      en: 'Who is present in which scene or event.',
      es: 'Quién está presente en cada escena o evento.',
    },
    entityHeader: { en: 'Entity', es: 'Entidad' },
    columns: { en: 'Columns', es: 'Columnas' },
    byStructure: { en: 'Sections of the work', es: 'Secciones de la obra' },
    byEvents: { en: 'Events', es: 'Eventos' },
    level: { en: 'Level', es: 'Nivel' },
    innermost: { en: 'Innermost sections', es: 'Secciones más internas' },
    section: { en: 'Within', es: 'Dentro de' },
    wholeWork: { en: 'Whole work', es: 'Toda la obra' },
    rows: { en: 'Rows', es: 'Filas' },
    everyone: { en: 'Every entity', es: 'Todas las entidades' },
    includeEmpty: { en: 'Include entities never placed', es: 'Incluir entidades sin colocar' },
    noColumnsTitle: { en: 'Nothing to put in columns', es: 'Nada que poner en columnas' },
    noColumnsHint: {
      en: 'Add chapters or scenes in Structure, or events in the timeline.',
      es: 'Añade capítulos o escenas en Estructura, o eventos en la línea de tiempo.',
    },
    noRowsTitle: { en: 'Nobody appears yet', es: 'Todavía no aparece nadie' },
    noRowsHint: {
      en: 'Click a cell to record that an entity appears in a section.',
      es: 'Haz clic en una celda para registrar que una entidad aparece en una sección.',
    },
    place: { en: 'Record an appearance', es: 'Registrar una aparición' },
    total: { en: 'Total', es: 'Total' },
    editTitle: { en: 'Appearance', es: 'Aparición' },
    editHint: { en: '{entity} in {context}', es: '{entity} en {context}' },
    kind: { en: 'How it appears', es: 'Cómo aparece' },
    viewpoint: { en: 'Told through', es: 'Narrada a través de' },
    noViewpoint: { en: 'Nobody in particular', es: 'Nadie en particular' },
    viewpointHint: {
      en: 'The entity whose perspective tells this appearance.',
      es: 'La entidad desde cuya perspectiva se cuenta esta aparición.',
    },
    clear: { en: 'Remove appearance', es: 'Quitar aparición' },
  },
} satisfies TextGroup;
