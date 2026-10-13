import type { TextGroup } from '../define';

/** Opening, creating and configuring a project. */
export const projectTexts = {
  welcome: {
    subtitle: {
      en: 'Local-first environment for modeling fictional universes',
      es: 'Entorno local-first para modelar universos ficticios',
    },
    manifesto: {
      en: "The system should understand the author's world without taking ownership of the author's creativity.",
      es: 'El sistema debe entender el mundo del autor sin arrebatarle su creatividad.',
    },
    brandStudio: { en: 'Sxnnyside Project', es: 'Sxnnyside Project' },
    brandEditorial: { en: 'Crown Red Editorial', es: 'Crown Red Editorial' },
    createTitle: { en: 'Create a project', es: 'Crear un proyecto' },
    createTab: { en: 'Create a project', es: 'Crear un proyecto' },
    openTab: { en: 'Open a project', es: 'Abrir un proyecto' },
    projectName: { en: 'Universe name', es: 'Nombre del universo' },
    projectNamePlaceholder: { en: 'The Ashen Empire', es: 'El Imperio de Ceniza' },
    premise: { en: 'Premise', es: 'Premisa' },
    premisePlaceholder: {
      en: 'One line defining the core nature of this world…',
      es: 'Una línea que resuma la esencia de este mundo…',
    },
    description: { en: 'Universe description', es: 'Descripción del universo' },
    descriptionPlaceholder: {
      en: 'Narrative vision notes, tone, or worldbuilding overview…',
      es: 'Anotaciones de visión narrativa, tono o trasfondo general…',
    },
    optionalDetails: {
      en: 'Premise & description (optional)',
      es: 'Premisa y descripción (opcional)',
    },
    location: { en: 'Where to keep it', es: 'Dónde guardarlo' },
    chooseLocation: { en: 'Choose a folder', es: 'Elegir una carpeta' },
    browseFolder: { en: 'Browse folder…', es: 'Examinar carpeta…' },
    noLocation: { en: 'No folder chosen', es: 'Ninguna carpeta elegida' },
    destinationPreview: { en: 'Project folder:', es: 'Carpeta del proyecto:' },
    createAction: { en: 'Found Project', es: 'Fundar Proyecto' },
    openTitle: { en: 'Open project', es: 'Abrir un proyecto' },
    openHint: {
      en: 'Choose an existing .ontara folder to continue working on it.',
      es: 'Elige una carpeta .ontara existente para continuar trabajando en ella.',
    },
    openFolderHint: {
      en: 'A project is a self-contained folder ending in .ontara.',
      es: 'Un proyecto es una carpeta independiente que termina en .ontara.',
    },
    openAction: { en: 'Choose project…', es: 'Elegir proyecto…' },
    recentTitle: { en: 'Recent projects', es: 'Proyectos recientes' },
    noRecent: {
      en: 'Projects you open will appear here.',
      es: 'Los proyectos que abras aparecerán aquí.',
    },
    forget: { en: 'Remove from the list', es: 'Quitar de la lista' },
    forgotten: {
      en: 'Removed from recent projects. The folder was not touched.',
      es: 'Quitado de recientes. La carpeta no se tocó.',
    },
    migrationTitle: { en: 'Upgrade this project?', es: '¿Actualizar este proyecto?' },
    migrationBody: {
      en: 'This project was saved by an older version. Ontara will take a backup inside the project and then upgrade it.',
      es: 'Este proyecto lo guardó una versión anterior. Ontara hará una copia de seguridad dentro del proyecto y luego lo actualizará.',
    },
    migrate: { en: 'Back up and upgrade', es: 'Respaldar y actualizar' },
    resumeProject: { en: 'Open project', es: 'Abrir proyecto' },
    pressEnter: { en: 'Press Enter to create', es: 'Presiona Enter para crear' },
    pillarsTitle: { en: 'Ontara Archival Pillars', es: 'Pilares Archivísticos de Ontara' },
    pillarGraph: { en: 'Typed Relationship Graph', es: 'Grafo Relacional Tipado' },
    pillarGraphDesc: {
      en: 'Configurable entities, directed links, and causal dependencies.',
      es: 'Entidades configurables, enlaces dirigidos y dependencias causales.',
    },
    pillarChronology: { en: 'Chronology & Matrix', es: 'Cronología In-Universe y Matriz' },
    pillarChronologyDesc: {
      en: 'Absolute Universe Days and scene-by-scene classified appearances.',
      es: 'Días de Universo absolutos y apariciones clasificadas escena por escena.',
    },
    pillarConsistency: { en: 'Deterministic Consistency', es: 'Consistencia Determinista' },
    pillarConsistencyDesc: {
      en: 'Explainable rule checks that never mutate canon without approval.',
      es: 'Verificación explicable de reglas que jamás altera el canon sin permiso.',
    },
    localBadge: {
      en: '100% Local-First • Autonomous Storage • Zero Telemetry',
      es: '100% Local-First • Almacenamiento Autónomo • Cero Telemetría',
    },
  },
  project: {
    settingsTitle: { en: 'Project settings', es: 'Ajustes del proyecto' },
    name: { en: 'Name', es: 'Nombre' },
    premise: { en: 'Premise', es: 'Premisa' },
    premiseHint: {
      en: 'One line that says what this universe is.',
      es: 'Una línea que diga qué es este universo.',
    },
    description: { en: 'Description', es: 'Descripción' },
    folder: { en: 'Project folder', es: 'Carpeta del proyecto' },
    backup: { en: 'Create backup', es: 'Crear copia de seguridad' },
    backupDone: { en: 'Backup written to {path}', es: 'Copia escrita en {path}' },
  },
} satisfies TextGroup;
