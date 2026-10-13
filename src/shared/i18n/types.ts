import type { WorkspaceSchema } from './locales/workspace-en';

export type SupportedLocale = 'en' | 'es';

export interface TranslationSchema {
  common: {
    appName: string;
    tagline: string;
    search: string;
    searchShortcut: string;
    themeDark: string;
    themeLight: string;
    language: string;
    loading: string;
    error: string;
    inspect: string;
    edit: string;
    add: string;
    filter: string;
  };
  tabs: {
    graph: string;
    matrix: string;
    timeline: string;
    consistency: string;
  };
  canon: {
    established: string;
    provisional: string;
    disputed: string;
    deprecated: string;
    rejected: string;
    speculative: string;
    filterLabel: string;
  };
  appearance: {
    physical: string;
    dialogue: string;
    pov: string;
    mention: string;
    flashback: string;
    absence: string;
    none: string;
  };
  sidebar: {
    navigator: string;
    editorialStructure: string;
    entities: string;
    addEntity: string;
  };
  inspector: {
    title: string;
    description: string;
    aliases: string;
    typedRelations: string;
    sources: string;
    editEntity: string;
  };
  timeline: {
    title: string;
    subtitle: string;
    scale: string;
    dayPrefix: string;
  };
  matrixView: {
    title: string;
    subtitle: string;
    entityHeader: string;
  };
  graphView: {
    title: string;
    subtitle: string;
  };
  consistency: {
    title: string;
    subtitle: string;
    rerun: string;
    affects: string;
    anomaliesDetected: string;
  };
  entityTypes: {
    character: string;
    faction: string;
    location: string;
    event: string;
    relic: string;
  };
  status: {
    sqliteLocal: string;
  };
}

export type NestedKeyOf<T> = {
  [K in keyof T & string]: T[K] extends Record<string, unknown> ? `${K}.${keyof T[K] & string}` : K;
}[keyof T & string];

/** Every text the interface has: the prototype's and the working interface's. */
export type FullSchema = TranslationSchema & WorkspaceSchema;

export type TranslationKey = NestedKeyOf<FullSchema>;
