import type { TextGroup } from '../define';

/** The getting-started list and the example universe. */
export const onboardingTexts = {
  onboarding: {
    title: { en: 'Getting started', es: 'Primeros pasos' },
    progress: { en: '{done} of {total}', es: '{done} de {total}' },
    dismiss: { en: 'Hide this list', es: 'Ocultar esta lista' },
    done: { en: 'Done', es: 'Hecho' },
    entities: { en: 'Create three entities', es: 'Crea tres entidades' },
    entitiesText: {
      en: 'Characters, places, a faction: start with what the story cannot do without.',
      es: 'Personajes, lugares, una facción: empieza por lo que la historia no puede perder.',
    },
    relationships: { en: 'Relate two of them', es: 'Relaciona dos de ellas' },
    relationshipsText: {
      en: 'Open an entity and use its Relationships tab.',
      es: 'Abre una entidad y usa su pestaña de Relaciones.',
    },
    structure: { en: 'Add a section to the work', es: 'Añade una sección a la obra' },
    structureText: {
      en: 'A volume, a chapter, a scene: whatever the work is made of.',
      es: 'Un volumen, un capítulo, una escena: de lo que esté hecha la obra.',
    },
    appearances: { en: 'Say who appears where', es: 'Di quién aparece dónde' },
    appearancesText: {
      en: 'In the matrix, mark an entity as present in a scene.',
      es: 'En la matriz, marca a una entidad como presente en una escena.',
    },
    events: { en: 'Record an event', es: 'Registra un evento' },
    eventsText: {
      en: 'Give it a date and who took part, and it joins the timeline.',
      es: 'Dale una fecha y quién participó, y se une a la cronología.',
    },
    loadExample: { en: 'Load a small example', es: 'Cargar un ejemplo pequeño' },
    exampleLoaded: {
      en: 'Example loaded: {entities} entities. Open Checks to see what it caught.',
      es: 'Ejemplo cargado: {entities} entidades. Abre Revisión para ver qué detectó.',
    },
    withExample: {
      en: 'Start with a small example universe',
      es: 'Empezar con un universo de ejemplo pequeño',
    },
    withExampleHint: {
      en: 'Characters, places, a timeline and one contradiction for the checks to find.',
      es: 'Personajes, lugares, una cronología y una contradicción para que la revisión la encuentre.',
    },
  },
} satisfies TextGroup;
