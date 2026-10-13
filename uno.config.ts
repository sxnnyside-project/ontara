import { defineConfig, presetIcons, presetUno } from 'unocss';

export default defineConfig({
  presets: [
    presetUno(),
    presetIcons({
      scale: 1.2,
      warn: true,
      extraProperties: {
        display: 'inline-block',
        'vertical-align': 'middle',
      },
      collections: {
        mingcute: () => import('@iconify-json/mingcute/icons.json'),
      },
    }),
  ],
  theme: {
    fontFamily: {
      serif: 'var(--font-serif)',
      sans: 'var(--font-sans)',
      mono: 'var(--font-mono)',
    },
    colors: {
      crown: {
        DEFAULT: 'var(--color-crown-red)',
        hover: 'var(--color-crown-red-hover)',
        active: 'var(--color-crown-red-active)',
        muted: 'var(--color-crown-red-muted)',
      },
      surface: {
        base: 'var(--color-surface-base)',
        panel: 'var(--color-surface-panel)',
        elevated: 'var(--color-surface-elevated)',
        overlay: 'var(--color-surface-overlay)',
        hover: 'var(--color-surface-hover)',
      },
      border: {
        subtle: 'var(--color-border-subtle)',
        strong: 'var(--color-border-strong)',
        focus: 'var(--color-border-focus)',
      },
      canon: {
        established: 'var(--color-canon-established)',
        provisional: 'var(--color-canon-provisional)',
        disputed: 'var(--color-canon-disputed)',
        deprecated: 'var(--color-canon-deprecated)',
        rejected: 'var(--color-canon-rejected)',
        speculative: 'var(--color-canon-speculative)',
      },
      meridian: {
        blue: 'var(--color-meridian-blue)',
        timeline: 'var(--color-meridian-timeline)',
      },
    },
  },
});
