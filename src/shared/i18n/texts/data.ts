import type { TextGroup } from '../define';

/** Backups, restoring and exporting the universe. */
export const dataTexts = {
  data: {
    title: { en: 'Backups and export', es: 'Copias y exportación' },
    open: { en: 'Backups and export…', es: 'Copias y exportación…' },
    backups: { en: 'Backups', es: 'Copias de seguridad' },
    backupsHint: {
      en: 'A copy is made automatically once a day, and before anything risky. Restoring keeps the current project as a backup first.',
      es: 'Se hace una copia automática una vez al día, y antes de lo riesgoso. Al restaurar, el proyecto actual se conserva como copia primero.',
    },
    backupNow: { en: 'Back up now', es: 'Hacer una copia ahora' },
    noBackups: { en: 'No backups yet.', es: 'Aún no hay copias.' },
    restore: { en: 'Restore', es: 'Restaurar' },
    restoreTitle: { en: 'Restore this backup?', es: '¿Restaurar esta copia?' },
    restoreBody: {
      en: 'The project goes back to how it was on {when}. How it is now is kept as a backup, so you can come back to it.',
      es: 'El proyecto vuelve a como estaba el {when}. Como está ahora se conserva como copia, para que puedas volver a ello.',
    },
    restored: { en: 'Project restored', es: 'Proyecto restaurado' },
    kind_backup: { en: 'Manual', es: 'Manual' },
    kind_auto: { en: 'Automatic', es: 'Automática' },
    'kind_pre-restore': { en: 'Before a restore', es: 'Antes de restaurar' },
    'kind_pre-migration': { en: 'Before an upgrade', es: 'Antes de actualizar' },
    export: { en: 'Export', es: 'Exportar' },
    exportHint: {
      en: 'Files are written into the exports folder of the project.',
      es: 'Los archivos se escriben en la carpeta de exportaciones del proyecto.',
    },
    formatMarkdown: { en: 'Document (Markdown)', es: 'Documento (Markdown)' },
    formatMarkdownHint: {
      en: 'Entities by type with their text, properties, relationships and notes, then the structure of the work.',
      es: 'Entidades por tipo con su texto, propiedades, relaciones y notas, y después la estructura de la obra.',
    },
    formatJson: { en: 'Everything (JSON)', es: 'Todo (JSON)' },
    formatJsonHint: {
      en: 'Every record as the application stores it, to keep or to process elsewhere.',
      es: 'Cada registro como lo guarda la aplicación, para conservarlo o procesarlo en otro lugar.',
    },
    exportNow: { en: 'Export now', es: 'Exportar ahora' },
    exported: {
      en: 'Exported {entities} entities to {path}',
      es: 'Se exportaron {entities} entidades a {path}',
    },
    contents: { en: 'Contents', es: 'Contenido' },
    summary: { en: 'Summary', es: 'Resumen' },
    description: { en: 'Description', es: 'Descripción' },
    tags: { en: 'Tags', es: 'Etiquetas' },
    aliases: { en: 'Also known as', es: 'También conocido como' },
    properties: { en: 'Properties', es: 'Propiedades' },
    relationships: { en: 'Relationships', es: 'Relaciones' },
    notes: { en: 'Notes', es: 'Notas' },
    structure: { en: 'Structure', es: 'Estructura' },
    canon: { en: 'Canon', es: 'Canon' },
    close: { en: 'Close', es: 'Cerrar' },
  },
} satisfies TextGroup;
