import type { TranslationSchema } from '../types';

export const es: TranslationSchema = {
  common: {
    appName: 'Ontara',
    tagline: 'The Meridian Design System ✦ Modelado Narrativo Local-First',
    search: 'Buscar',
    searchShortcut: '⌘K',
    themeDark: 'Cambiar a modo Midnight (oscuro)',
    themeLight: 'Cambiar a modo Galera (claro)',
    language: 'Idioma',
    loading: 'Cargando…',
    error: 'Error',
    inspect: 'Inspeccionar',
    edit: 'Editar',
    add: 'Añadir',
    filter: 'Filtrar',
  },
  tabs: {
    graph: 'Grafo',
    matrix: 'Matriz',
    timeline: 'Cronología',
    consistency: 'Consistencia',
  },
  canon: {
    established: 'Establecido',
    provisional: 'Provisional',
    disputed: 'Disputado',
    deprecated: 'Descartado',
    rejected: 'Rechazado',
    speculative: 'Especulativo',
    filterLabel: 'Filtro de Canon:',
  },
  appearance: {
    physical: 'Presencia Física',
    dialogue: 'Diálogo Activo',
    pov: 'Punto de Vista (POV)',
    mention: 'Mención Indirecta',
    flashback: 'Flashback / Recuerdo',
    absence: 'Ausencia Confirmada',
    none: 'Sin Aparición',
  },
  sidebar: {
    navigator: 'Navegador',
    editorialStructure: 'Estructura Editorial',
    entities: 'Entidades del Universo',
    addEntity: 'Añadir entidad',
  },
  inspector: {
    title: 'Expediente Canónico',
    description: 'Descripción Canónica',
    aliases: 'Alias y Títulos',
    typedRelations: 'Relaciones Tipadas',
    sources: 'Fuente y Citas',
    editEntity: 'Editar entidad',
  },
  timeline: {
    title: 'Cronología In-Universe (Tiempo Canónico)',
    subtitle:
      'Eventos ordenados por Días de Universo absolutos (D+), independientes del orden de lectura.',
    scale: 'Escala: Días de Universo (D+)',
    dayPrefix: 'D+',
  },
  matrixView: {
    title: 'Matriz de Apariciones Narrativas',
    subtitle:
      'Presencia clasificada por entidad a lo largo del manuscrito (0% emojis, 100% vector).',
    entityHeader: 'Entidad',
  },
  graphView: {
    title: 'Lienzo de Grafo de Relaciones',
    subtitle: 'Topología conectada del universo ficcional y dependencias causales.',
  },
  consistency: {
    title: 'Motor Determinista de Consistencia',
    subtitle: 'Reglas formales de verificación de canon ejecutadas sobre el grafo relacional.',
    rerun: 'Re-ejecutar Reglas',
    affects: 'Afecta',
    anomaliesDetected: 'anomalías detectadas',
  },
  entityTypes: {
    character: 'Personaje',
    faction: 'Facción',
    location: 'Localización',
    event: 'Evento',
    relic: 'Reliquia',
  },
  status: {
    sqliteLocal: 'SQLite Local-First (142 registros)',
  },
};
