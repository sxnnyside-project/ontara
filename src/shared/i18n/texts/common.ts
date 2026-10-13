import type { TextGroup } from '../define';

/** Texts shared by every part of the interface: actions, states of the application, time. */
export const commonTexts = {
  common: {
    appName: { en: 'Ontara', es: 'Ontara' },
    search: { en: 'Search', es: 'Buscar' },
    searchShortcut: { en: '⌘K', es: '⌘K' },
    themeDark: { en: 'Switch to Midnight mode (dark)', es: 'Cambiar a modo Midnight (oscuro)' },
    themeLight: { en: 'Switch to Galley mode (light)', es: 'Cambiar a modo Galera (claro)' },
    language: { en: 'Language', es: 'Idioma' },
    loading: { en: 'Loading…', es: 'Cargando…' },
    inspect: { en: 'Inspect', es: 'Inspeccionar' },
    edit: { en: 'Edit', es: 'Editar' },
    add: { en: 'Add', es: 'Añadir' },
    cancel: { en: 'Cancel', es: 'Cancelar' },
    dismiss: { en: 'Dismiss', es: 'Descartar' },
    close: { en: 'Close', es: 'Cerrar' },
    save: { en: 'Save', es: 'Guardar' },
    remove: { en: 'Remove', es: 'Quitar' },
    discard: { en: 'Discard changes', es: 'Descartar cambios' },
    unsaved: { en: 'Unsaved changes', es: 'Cambios sin guardar' },
  },
  time: {
    words: { en: 'In words', es: 'En palabras' },
    day: { en: 'Day', es: 'Día' },
    moment: { en: 'Moment', es: 'Momento' },
    from: { en: 'From', es: 'Desde' },
    until: { en: 'Until', es: 'Hasta' },
    openStart: {
      en: 'Leave empty if it has no known start.',
      es: 'Déjalo vacío si no tiene un inicio conocido.',
    },
    openEnd: { en: 'Leave empty if it has not ended.', es: 'Déjalo vacío si no ha terminado.' },
    unconstrained: { en: 'Not tied to the timeline', es: 'Sin lugar en la línea de tiempo' },
    unknown: { en: 'Date not known yet', es: 'Fecha aún desconocida' },
    at: { en: 'At a moment', es: 'En un momento' },
    span: { en: 'Over a span', es: 'Durante un periodo' },
  },
  status: {
    localProject: { en: 'Local project', es: 'Proyecto local' },
    types: { en: 'entity types', es: 'tipos de entidad' },
  },
  picker: {
    search: { en: 'Type to narrow the list', es: 'Escribe para filtrar la lista' },
    choose: { en: 'Choose', es: 'Elegir' },
    none: { en: 'Nobody chosen', es: 'Nadie elegido' },
  },
} satisfies TextGroup;
