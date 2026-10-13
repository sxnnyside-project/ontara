// Static accessibility checks that need no browser: colour contrast of the design tokens, and
// patterns in the Svelte sources that leave something unnamed or unreachable.
//
// Run with `bun scripts/check-accessibility.ts`; it exits non-zero when it finds a problem.

import { readdirSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';

export interface Problem {
  where: string;
  what: string;
}

/** WCAG relative luminance of a `#rrggbb` colour. */
export function luminance(hex: string): number {
  const value = hex.replace('#', '');
  const channels = [0, 2, 4].map((start) => {
    const channel = Number.parseInt(value.slice(start, start + 2), 16) / 255;
    return channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  });
  const [red = 0, green = 0, blue = 0] = channels;
  return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
}

/** WCAG contrast ratio between two `#rrggbb` colours, from 1 to 21. */
export function contrast(first: string, second: string): number {
  const [light, dark] = [luminance(first), luminance(second)].sort((a, b) => b - a);
  return ((light ?? 0) + 0.05) / ((dark ?? 0) + 0.05);
}

/** The custom properties a block of CSS declares, by name. */
export function declarations(block: string): Map<string, string> {
  const found = new Map<string, string>();
  for (const match of block.matchAll(/(--[a-z0-9-]+)\s*:\s*([^;]+);/g)) {
    const [, name, value] = match;
    if (name && value) found.set(name, value.trim());
  }
  return found;
}

const SURFACES = ['--color-surface-base', '--color-surface-panel', '--color-surface-elevated'];
const TEXT = [
  { name: '--color-text-primary', minimum: 4.5 },
  { name: '--color-text-secondary', minimum: 4.5 },
  { name: '--color-text-muted', minimum: 4.5 },
  { name: '--color-crown-red-text', minimum: 4.5 },
  { name: '--color-finding-error', minimum: 4.5 },
  { name: '--color-finding-warning', minimum: 4.5 },
  { name: '--color-finding-info', minimum: 4.5 },
  { name: '--color-canon-established', minimum: 4.5 },
  { name: '--color-canon-provisional', minimum: 4.5 },
  { name: '--color-canon-disputed', minimum: 4.5 },
  { name: '--color-canon-deprecated', minimum: 4.5 },
  { name: '--color-canon-rejected', minimum: 4.5 },
  { name: '--color-canon-speculative', minimum: 4.5 },
];

/** Tokens that are plain `#rrggbb` and fall short of the contrast text needs on a surface. */
export function contrastProblems(theme: string, tokens: Map<string, string>): Problem[] {
  const problems: Problem[] = [];
  for (const text of TEXT) {
    const foreground = tokens.get(text.name);
    if (!foreground || !/^#[0-9a-f]{6}$/i.test(foreground)) continue;
    for (const surface of SURFACES) {
      const background = tokens.get(surface);
      if (!background || !/^#[0-9a-f]{6}$/i.test(background)) continue;
      const ratio = contrast(foreground, background);
      if (ratio < text.minimum) {
        problems.push({
          where: `tokens.css (${theme})`,
          what: `${text.name} on ${surface} is ${ratio.toFixed(2)}:1, under ${text.minimum}:1`,
        });
      }
    }
  }
  // White text on the red fill of a primary button.
  const fill = tokens.get('--color-crown-red');
  if (fill && /^#[0-9a-f]{6}$/i.test(fill) && contrast('#ffffff', fill) < 4.5) {
    problems.push({
      where: `tokens.css (${theme})`,
      what: `white on --color-crown-red is ${contrast('#ffffff', fill).toFixed(2)}:1, under 4.5:1`,
    });
  }
  return problems;
}

/** The text inside an element with its tags removed and expressions kept as a word. */
function visibleText(inner: string): string {
  return inner
    .replace(/\{@render[^}]*\}/g, 'x')
    .replace(/<[^>]*>/g, ' ')
    .replace(/\{[^}]*\}/g, 'x')
    .trim();
}

/** Patterns in a Svelte file that leave a control without a name or without a focus style. */
export function sourceProblems(file: string, source: string): Problem[] {
  const problems: Problem[] = [];
  const line = (index: number) => source.slice(0, index).split('\n').length;

  for (const match of source.matchAll(/<img\b[^>]*>/g)) {
    if (!/\balt\s*=/.test(match[0])) {
      problems.push({ where: `${file}:${line(match.index ?? 0)}`, what: 'an <img> has no alt' });
    }
  }

  for (const match of source.matchAll(/<button\b([^>]*)>([\s\S]*?)<\/button>/g)) {
    const attributes = match[1] ?? '';
    const named = /\baria-label\s*=|\btitle\s*=|\baria-labelledby\s*=/.test(attributes);
    if (!named && visibleText(match[2] ?? '') === '') {
      problems.push({
        where: `${file}:${line(match.index ?? 0)}`,
        what: 'a <button> has no text, aria-label or title',
      });
    }
  }

  for (const match of source.matchAll(/class="([^"]*)"/g)) {
    const classes = match[1] ?? '';
    const groupShowsFocus = source.includes('focus-within:');
    if (
      /\boutline-none\b/.test(classes) &&
      !/\bfocus(-visible)?:/.test(classes) &&
      !groupShowsFocus
    ) {
      problems.push({
        where: `${file}:${line(match.index ?? 0)}`,
        what: 'outline-none without a replacement focus style',
      });
    }
  }

  for (const match of source.matchAll(
    /\b(aria-label|title|placeholder)="([^"{]*[A-Za-zÁ-ú][^"{]*)"/g,
  )) {
    problems.push({
      where: `${file}:${line(match.index ?? 0)}`,
      what: `${match[1]}="${match[2]}" is written in one language; use a text from the dictionary`,
    });
  }
  return problems;
}

function svelteFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return svelteFiles(path);
    return entry.name.endsWith('.svelte') ? [path] : [];
  });
}

/** Splits the stylesheet into the dark (default) and light token blocks. */
export function themes(css: string): { dark: Map<string, string>; light: Map<string, string> } {
  const lightStart = css.search(/\[data-theme='light'\]/);
  const darkBlock = css.slice(css.indexOf(':root'), lightStart === -1 ? undefined : lightStart);
  const lightBlock = lightStart === -1 ? '' : css.slice(lightStart);
  const light = declarations(lightBlock.slice(0, lightBlock.indexOf('}') + 1));
  const dark = declarations(darkBlock);
  // The light theme only restates what differs from the dark one.
  return { dark, light: new Map([...dark, ...light]) };
}

function main(): number {
  const root = join(import.meta.dir, '..');
  const problems: Problem[] = [];

  const css = readFileSync(join(root, 'src/app/tokens.css'), 'utf8');
  if (!css.includes('prefers-reduced-motion')) {
    problems.push({ where: 'tokens.css', what: 'there is no rule for prefers-reduced-motion' });
  }
  if (!css.includes(':focus-visible')) {
    problems.push({ where: 'tokens.css', what: 'there is no visible keyboard focus style' });
  }
  const { dark, light } = themes(css);
  problems.push(...contrastProblems('dark', dark), ...contrastProblems('light', light));

  for (const file of svelteFiles(join(root, 'src'))) {
    problems.push(...sourceProblems(relative(root, file), readFileSync(file, 'utf8')));
  }

  for (const problem of problems) console.error(`${problem.where}: ${problem.what}`);
  console.log(`accessibility: ${problems.length} problem(s)`);
  return problems.length === 0 ? 0 : 1;
}

if (import.meta.main) process.exit(main());
