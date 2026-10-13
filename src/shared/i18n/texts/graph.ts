import type { TextGroup } from '../define';

/** Graph exploration. */
export const graphTexts = {
  graphView: {
    selected: { en: 'Selection', es: 'Selección' },
    around: { en: 'Around {name}', es: 'En torno a {name}' },
    selectHint: { en: 'Click a node to see it here.', es: 'Haz clic en un nodo para verlo aquí.' },
    alone: {
      en: 'Nothing is connected to this entity yet. Add relationships from its page.',
      es: 'Esta entidad aún no está conectada a nada. Añade relaciones desde su página.',
    },
    zoomIn: { en: 'Zoom in', es: 'Acercar' },
    zoomOut: { en: 'Zoom out', es: 'Alejar' },
    reset: { en: 'Reset view', es: 'Restablecer vista' },
    title: { en: 'Relationship graph', es: 'Grafo de relaciones' },
    subtitle: {
      en: 'Explore who is connected to whom, around one entity.',
      es: 'Explora quién se conecta con quién en torno a una entidad.',
    },
    depth: { en: 'Depth', es: 'Profundidad' },
    steps: { en: '{count} step(s)', es: '{count} paso(s)' },
    includeEvents: { en: 'Include events and places', es: 'Incluir eventos y lugares' },
    allTypes: { en: 'All relationship types', es: 'Todos los tipos de relación' },
    someTypes: {
      en: '{count} relationship type(s) chosen',
      es: '{count} tipo(s) de relación elegidos',
    },
    clearTypes: { en: 'Clear', es: 'Quitar' },
    pickTitle: { en: 'Pick an entity to explore', es: 'Elige una entidad para explorar' },
    pickHint: {
      en: 'Choose something in the navigator and its neighborhood appears here.',
      es: 'Elige algo en el navegador y su entorno aparecerá aquí.',
    },
    truncated: {
      en: 'This neighborhood is larger than {limit} entities, so only the nearest are shown. Narrow it by type or depth.',
      es: 'Este entorno supera las {limit} entidades, así que solo se muestran las más cercanas. Redúcelo por tipo o profundidad.',
    },
    location: { en: 'takes place at', es: 'ocurre en' },
    participation: { en: 'takes part in', es: 'participa en' },
    legend: { en: 'Legend', es: 'Leyenda' },
    legendCustomize: { en: 'Customize', es: 'Personalizar' },
    legendDone: { en: 'Done', es: 'Listo' },
    legendReset: { en: 'Restore defaults', es: 'Restaurar valores' },
    color: { en: 'Color', es: 'Color' },
    alwaysLabels: { en: 'Always show link names', es: 'Mostrar siempre los nombres' },
    pattern_solid: { en: 'Solid', es: 'Continua' },
    pattern_dashed: { en: 'Dashed', es: 'Rayas' },
    pattern_dotted: { en: 'Dotted', es: 'Puntos' },
    mark_circle: { en: 'Circle', es: 'Círculo' },
    mark_diamond: { en: 'Diamond', es: 'Rombo' },
    mark_square: { en: 'Square', es: 'Cuadrado' },
    mark_bar: { en: 'Bar', es: 'Barra' },
    centerHere: { en: 'Center here', es: 'Centrar aquí' },
    openEntity: { en: 'Open entity', es: 'Abrir entidad' },
    stats: { en: '{nodes} entities, {edges} links', es: '{nodes} entidades, {edges} vínculos' },
  },
} satisfies TextGroup;
