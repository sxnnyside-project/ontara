/** Languages the interface speaks. Adding one means adding it here and to every text. */
export const LOCALES = ['es', 'en'] as const;

export type SupportedLocale = (typeof LOCALES)[number];

/** One text in every language. A missing translation is a type error, not a runtime gap. */
export type Translation = Record<SupportedLocale, string>;

/** A group of texts: sections of keys, each key holding its translations. */
export type TextGroup = Record<string, Record<string, Translation>>;
