import { expect, test } from 'bun:test';
import {
  contrast,
  contrastProblems,
  declarations,
  sourceProblems,
  themes,
} from '../../scripts/check-accessibility';

test('contrast follows the WCAG ratio', () => {
  expect(contrast('#000000', '#ffffff')).toBeCloseTo(21, 0);
  expect(contrast('#777777', '#777777')).toBeCloseTo(1, 5);
  expect(contrast('#767676', '#ffffff')).toBeGreaterThan(4.5);
});

test('tokens that are too faint for text are reported by name', () => {
  const tokens = declarations(`
    --color-surface-base: #0c0e12;
    --color-surface-panel: #13161c;
    --color-surface-elevated: #1a1e26;
    --color-text-primary: #f3f4f6;
    --color-text-muted: #3a3f4a;
  `);

  const problems = contrastProblems('dark', tokens);

  expect(problems.length).toBe(3);
  expect(problems.every((problem) => problem.what.startsWith('--color-text-muted'))).toBe(true);
});

test('the light theme restates only what differs from the dark one', () => {
  const css = `:root { --a: #111111; --b: #222222; }
[data-theme='light'],
.theme-light { --b: #eeeeee; }`;

  const { dark, light } = themes(css);

  expect(dark.get('--b')).toBe('#222222');
  expect(light.get('--a')).toBe('#111111');
  expect(light.get('--b')).toBe('#eeeeee');
});

test('a button with only an icon, an image with no alt and hidden focus are found', () => {
  const source = [
    '<button type="button"><span class="i-x"></span></button>',
    '<button type="button" aria-label={close}><span class="i-x"></span></button>',
    '<button type="button">{label}</button>',
    '<img src="a.png">',
    '<img src="a.png" alt="">',
    '<input class="outline-none">',
    '<input class="outline-none focus:border-red">',
    '<span title="Close"></span>',
    '<span title={t("x")}></span>',
  ].join('\n');

  const found = sourceProblems('x.svelte', source)
    .map((problem) => problem.where)
    .sort();

  expect(found).toEqual(['x.svelte:1', 'x.svelte:4', 'x.svelte:6', 'x.svelte:8']);
});
