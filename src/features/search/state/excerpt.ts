const START = '\u0002';
const END = '\u0003';

/** A piece of an excerpt, and whether it is one of the words that matched. */
export interface ExcerptPart {
  text: string;
  matched: boolean;
}

/** Splits an excerpt into plain and matching parts, so it can be shown with the matches marked. */
export function excerptParts(excerpt: string): ExcerptPart[] {
  const parts: ExcerptPart[] = [];
  let matched = false;
  let text = '';
  for (const character of excerpt) {
    if (character === START || character === END) {
      if (text !== '') parts.push({ text, matched });
      text = '';
      matched = character === START;
    } else {
      text += character;
    }
  }
  if (text !== '') parts.push({ text, matched });
  return parts;
}
