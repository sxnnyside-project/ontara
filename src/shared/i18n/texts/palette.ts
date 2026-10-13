import type { TextGroup } from '../define';

/** The command palette: searching the universe and running commands from the keyboard. */
export const paletteTexts = {
  palette: {
    placeholder: {
      en: 'Search the universe, or type > for commands',
      es: 'Busca en el universo, o escribe > para ver comandos',
    },
    label: { en: 'Command palette', es: 'Paleta de comandos' },
    commands: { en: 'Commands', es: 'Comandos' },
    entities: { en: 'Entities', es: 'Entidades' },
    none: { en: 'Nothing matches.', es: 'Nada coincide.' },
    hint: {
      en: 'Enter to run, arrows to move, Esc to close',
      es: 'Enter para ejecutar, flechas para moverte, Esc para cerrar',
    },
    goTo: { en: 'Go to {view}', es: 'Ir a {view}' },
    newEntity: { en: 'New entity', es: 'Nueva entidad' },
    undo: { en: 'Undo', es: 'Deshacer' },
    redo: { en: 'Redo', es: 'Rehacer' },
    history: { en: 'Show history', es: 'Ver historial' },
    toggleTheme: { en: 'Switch light and dark mode', es: 'Cambiar entre modo claro y oscuro' },
    toggleLanguage: { en: 'Switch language', es: 'Cambiar de idioma' },
    toggleNavigator: { en: 'Show or hide the navigator', es: 'Mostrar u ocultar el navegador' },
    settings: { en: 'Project settings', es: 'Ajustes del proyecto' },
    backup: { en: 'Create a backup now', es: 'Crear una copia de seguridad ahora' },
    export: { en: 'Export the universe…', es: 'Exportar el universo…' },
    closeProject: { en: 'Close the project', es: 'Cerrar el proyecto' },
    backupDone: { en: 'Backup saved to {path}', es: 'Copia guardada en {path}' },
  },
} satisfies TextGroup;
