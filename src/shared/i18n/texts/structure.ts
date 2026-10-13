import type { TextGroup } from '../define';

/** The editorial structure of the work. */
export const structureTexts = {
  structure: {
    stepVolume: { en: 'Add a volume', es: 'Añade un volumen' },
    stepVolumeText: {
      en: 'The top of the work: a book, a series, a season.',
      es: 'Lo más alto de la obra: un libro, una serie, una temporada.',
    },
    stepChapter: { en: 'Divide it', es: 'Divídelo' },
    stepChapterText: {
      en: 'Chapters and scenes go inside, in the order they are read.',
      es: 'Dentro van capítulos y escenas, en el orden en que se leen.',
    },
    stepAppear: { en: 'Say who appears', es: 'Di quién aparece' },
    stepAppearText: {
      en: 'The matrix records which entities are present in each scene.',
      es: 'La matriz registra qué entidades están presentes en cada escena.',
    },
    title: { en: 'Editorial structure', es: 'Estructura editorial' },
    subtitle: {
      en: 'How the work is divided: volumes, parts, chapters and scenes.',
      es: 'Cómo se divide la obra: volúmenes, partes, capítulos y escenas.',
    },
    addTop: { en: 'Add top-level section', es: 'Añadir sección de primer nivel' },
    addTitle: { en: 'New section', es: 'Nueva sección' },
    addInside: { en: 'Add inside this section', es: 'Añadir dentro de esta sección' },
    topLevel: { en: 'At the top of the work', es: 'En lo más alto de la obra' },
    insideOf: { en: 'Inside {name}', es: 'Dentro de {name}' },
    level: { en: 'Level', es: 'Nivel' },
    entityType: { en: 'Entity type', es: 'Tipo de entidad' },
    noDeeperLevel: {
      en: 'No level sits below this one. Define one in Configuration.',
      es: 'Ningún nivel queda por debajo de este. Define uno en Configuración.',
    },
    moveUp: { en: 'Move up', es: 'Subir' },
    moveDown: { en: 'Move down', es: 'Bajar' },
    expand: { en: 'Expand', es: 'Expandir' },
    collapse: { en: 'Collapse', es: 'Contraer' },
    empty: { en: 'No sections yet.', es: 'Aún no hay secciones.' },
    emptyTitle: { en: 'The work has no structure yet', es: 'La obra aún no tiene estructura' },
    emptyHint: {
      en: 'Add a volume, a part or a chapter to start organizing the manuscript.',
      es: 'Añade un volumen, una parte o un capítulo para empezar a organizar el manuscrito.',
    },
    removeTitle: { en: 'Remove {name} from the structure?', es: '¿Sacar {name} de la estructura?' },
    removeBody: {
      en: 'The section leaves the structure. Its entity, appearances and relationships are kept.',
      es: 'La sección sale de la estructura. Su entidad, apariciones y relaciones se conservan.',
    },
    removeFromStructure: { en: 'Take out of the structure', es: 'Sacar de la estructura' },
    removeAndDelete: { en: 'Remove the entity as well', es: 'Quitar también la entidad' },
    removeHasChildren: {
      en: 'It still has {count} section(s) inside. Move or remove them first.',
      es: 'Aún tiene {count} sección(es) dentro. Muévelas o quítalas primero.',
    },
    removeImpact: {
      en: 'It has {appearances} appearance(s) and is the context of {contexts} relationship(s).',
      es: 'Tiene {appearances} aparición(es) y es el contexto de {contexts} relación(es).',
    },
  },
} satisfies TextGroup;
