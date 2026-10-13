import type { TextGroup } from '../define';

/** Assets: importing, previewing and attaching files. */
export const libraryTexts = {
  assets: {
    stepImport: { en: 'Import files', es: 'Importa archivos' },
    stepImportText: {
      en: 'Images, maps and documents are copied into the project, so it stays portable.',
      es: 'Imágenes, mapas y documentos se copian al proyecto, que sigue siendo portable.',
    },
    stepAttach: { en: 'Attach them', es: 'Adjúntalos' },
    stepAttachText: {
      en: 'Attach a file to any entity from its page: a portrait, a map, a reference.',
      es: 'Adjunta un archivo a cualquier entidad desde su página: un retrato, un mapa, una referencia.',
    },
    title: { en: 'Asset library', es: 'Biblioteca de archivos' },
    subtitle: {
      en: 'Files that belong to the project: maps, portraits, references.',
      es: 'Archivos que pertenecen al proyecto: mapas, retratos, referencias.',
    },
    import: { en: 'Import files', es: 'Importar archivos' },
    chooseFiles: { en: 'Choose files to import', es: 'Elige los archivos a importar' },
    imported: {
      en: '{added} imported, {repeated} already in the project',
      es: '{added} importado(s), {repeated} ya estaban en el proyecto',
    },
    emptyTitle: { en: 'No assets yet', es: 'Aún no hay archivos' },
    emptyHint: {
      en: 'Import images or documents. They are copied into the project folder.',
      es: 'Importa imágenes o documentos. Se copian dentro de la carpeta del proyecto.',
    },
    missing: {
      en: 'The file is missing from the project folder.',
      es: 'El archivo no está en la carpeta del proyecto.',
    },
    missingBanner: {
      en: '{count} file(s) are missing from the project folder.',
      es: '{count} archivo(s) no están en la carpeta del proyecto.',
    },
    attachedCount: { en: '{count} attached', es: '{count} adjunto(s)' },
    importAndAttach: { en: 'Import and attach', es: 'Importar y adjuntar' },
    attachExisting: { en: 'Attach an existing asset', es: 'Adjuntar un archivo existente' },
    attach: { en: 'Attach', es: 'Adjuntar' },
    asset: { en: 'Asset', es: 'Archivo' },
    chooseAsset: { en: 'Choose an asset…', es: 'Elige un archivo…' },
    nothingToAttach: {
      en: 'Every asset is already attached. Import a file first.',
      es: 'Todos los archivos ya están adjuntos. Importa uno primero.',
    },
    purpose: { en: 'Purpose', es: 'Propósito' },
    purposeHint: {
      en: 'Optional, such as "portrait" or "reference".',
      es: 'Opcional, como "retrato" o "referencia".',
    },
    detach: { en: 'Detach', es: 'Desadjuntar' },
    detachTitle: { en: 'Detach this asset?', es: '¿Desadjuntar este archivo?' },
    detachBody: {
      en: 'Only the association is removed. The file stays in the library.',
      es: 'Solo se quita la asociación. El archivo sigue en la biblioteca.',
    },
    noAttachmentsTitle: { en: 'No assets attached', es: 'Sin archivos adjuntos' },
    noAttachmentsHint: {
      en: 'Attach images and documents to keep references next to the entity.',
      es: 'Adjunta imágenes y documentos para tener las referencias junto a la entidad.',
    },
    file: { en: 'File', es: 'Archivo' },
    size: { en: 'Size', es: 'Tamaño' },
    mediaType: { en: 'Type', es: 'Tipo' },
  },
} satisfies TextGroup;
