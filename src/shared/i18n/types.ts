import type { Texts } from './texts';

export type { SupportedLocale } from './define';

type NestedKeyOf<T> = {
  [Section in keyof T & string]: `${Section}.${keyof T[Section] & string}`;
}[keyof T & string];

/** Every text the interface has, as `section.key`. */
export type TranslationKey = NestedKeyOf<Texts>;
