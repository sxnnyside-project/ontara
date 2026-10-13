---
name: the-meridian-design-system
description: The Meridian Design System specification for Ontara (Sxnnyside Project x Crown Red Editorial). Design tokens, UnoCSS setup, MingCute icon guidelines (0% emojis), typography, and Svelte 5 component patterns for local-first narrative modeling.
---

# The Meridian Design System (Ontara)

*The Meridian Design System* is the design language and component architecture of **Ontara**, developed by **Sxnnyside Project** in collaboration with **Crown Red Editorial**.

It provides the visual dignity of an editorial publishing house combined with the density, keyboard ergonomics, and precision of a local-first narrative modeling environment.

---

## 1. Core Principles

1. **0% Emojis Policy**: No emojis are allowed in any UI controls, status badges, tree nodes, or graph cards. All iconography is strictly vector-based using **MingCute** icons through UnoCSS (`i-mingcute:<icon-name>`).
2. **Editorial Dignity + Cartographic Precision**:
   - High-contrast, literary serifs for narrative context and editorial titles (*Newsreader*).
   - High-legibility, neutral sans-serif for dense data views, trees, and forms (*Geist Sans*).
   - Tabular monospace for chronology, universe days, timestamps, and UUIDs (*Geist Mono*).
3. **Local-First & Offline Resilience**: Zero remote CDN requests. All fonts, styles, and vector assets are bundled locally into the desktop client under strict CSP (`font-src 'self'`).
4. **Density Without Clutter**: Designed for long desktop writing and worldbuilding sessions (8+ hours) with minimal visual noise and low-contrast borders.

---

## 2. Iconography: MingCute (Vector Only)

All icons use the `@iconify-json/mingcute` collection via UnoCSS icon classes.

### Canonical Mapping for Ontara Concepts

| Concept / Domain | MingCute Icon Class | Description |
|---|---|---|
| **Universe / Project** | `i-mingcute:earth-2-line` | Top-level project world |
| **Editorial Volume** | `i-mingcute:book-2-line` | Volumes and manuscripts |
| **Chapter / Scene** | `i-mingcute:file-line` / `i-mingcute:movie-line` | Scene or chapter nodes |
| **Entity (General)** | `i-mingcute:box-line` | Any identified narrative entity |
| **Character** | `i-mingcute:user-3-line` | Persona or sentient character |
| **Faction / Group** | `i-mingcute:group-line` | Organizations, clans, alliances |
| **Location / Region** | `i-mingcute:map-pin-line` | Geography, realms, planets |
| **Event** | `i-mingcute:calendar-line` | In-universe occurrence |
| **Relationship / Link** | `i-mingcute:git-branch-line` | Directed/undirected connections |
| **Canon Established** | `i-mingcute:check-circle-line` | Verified canon |
| **Canon Provisional** | `i-mingcute:time-line` | Work-in-progress / draft |
| **Canon Disputed** | `i-mingcute:alert-line` | Contradiction between sources |
| **Canon Deprecated** | `i-mingcute:archive-line` | Historical or superseded canon |
| **Appearance (Physical)**| `i-mingcute:walk-line` | Physical presence in scene |
| **Appearance (Dialogue)**| `i-mingcute:chat-3-line` | Spoken or reported dialogue |
| **Appearance (POV)**     | `i-mingcute:eye-line` | Scene perspective or viewpoint |
| **Appearance (Mention)** | `i-mingcute:quill-pen-line`| Mentioned in text |
| **Timeline / Day**      | `i-mingcute:history-line` | Chronological universe day |
| **Consistency Engine**  | `i-mingcute:shield-shape-line` | Deterministic validator |
| **Search / Palette**    | `i-mingcute:search-line` | Global command palette |

*Rule:* Never use emoji characters (e.g. 📚, 👤, ⚡, ❌) inside buttons, status chips, or table cells. Use `<span class="i-mingcute:<name>"></span>`.

---

## 3. Typography Stack

```css
/* Editorial narrative & book headings */
font-family: var(--font-serif); /* 'Newsreader', Georgia, serif */

/* UI controls, inspectors, trees, tables */
font-family: var(--font-sans);  /* 'Geist Sans', -apple-system, sans-serif */

/* Universe days, UUIDs, rules, dates */
font-family: var(--font-mono);  /* 'Geist Mono', monospace */
font-feature-settings: 'tnum' on, 'zero' on;
```

---

## 4. Design Tokens & Palette

### Surfaces & Borders
* `var(--color-surface-base)`: `#0c0e12` (Main viewport background)
* `var(--color-surface-panel)`: `#13161c` (Sidebar, inspect panels)
* `var(--color-surface-elevated)`: `#1a1e26` (Floating cards, graph nodes, popovers)
* `var(--color-surface-hover)`: `#29303d`
* `var(--color-border-subtle)`: `rgba(255, 255, 255, 0.07)`
* `var(--color-border-strong)`: `rgba(255, 255, 255, 0.14)`

### Brand (Crown Red Editorial)
* `var(--color-crown-red)`: `#e03144`
* `var(--color-crown-red-hover)`: `#f04255`
* `var(--color-crown-red-subtle)`: `rgba(224, 49, 68, 0.12)`

### Canon States
* **Established**: `var(--color-canon-established)` (`#10b981`)
* **Provisional**: `var(--color-canon-provisional)` (`#f59e0b`)
* **Disputed**: `var(--color-canon-disputed)` (`#f97316`)
* **Deprecated**: `var(--color-canon-deprecated)` (`#6b7280`)
* **Rejected**: `var(--color-canon-rejected)` (`#ef4444`)
* **Speculative**: `var(--color-canon-speculative)` (`#a855f7`)

---

## 5. UnoCSS Utilities & Component Patterns

With UnoCSS configured, write components using Svelte 5 runes (`$props`, `$state`, `$derived`):

### Example: Canon Status Badge (`canon-badge.svelte`)

```svelte
<script lang="ts">
interface Props {
  state: 'established' | 'provisional' | 'disputed' | 'deprecated' | 'rejected';
}

const { state }: Props = $props();

const config = $derived({
  established: {
    label: 'Established',
    icon: 'i-mingcute:check-circle-line',
    textClass: 'text-[var(--color-canon-established)]',
    bgClass: 'bg-[var(--color-canon-established-bg)]',
  },
  provisional: {
    label: 'Provisional',
    icon: 'i-mingcute:time-line',
    textClass: 'text-[var(--color-canon-provisional)]',
    bgClass: 'bg-[var(--color-canon-provisional-bg)]',
  },
  disputed: {
    label: 'Disputed',
    icon: 'i-mingcute:alert-line',
    textClass: 'text-[var(--color-canon-disputed)]',
    bgClass: 'bg-[var(--color-canon-disputed-bg)]',
  },
  deprecated: {
    label: 'Deprecated',
    icon: 'i-mingcute:archive-line',
    textClass: 'text-[var(--color-canon-deprecated)]',
    bgClass: 'bg-[var(--color-canon-deprecated-bg)]',
  },
  rejected: {
    label: 'Rejected',
    icon: 'i-mingcute:close-circle-line',
    textClass: 'text-[var(--color-canon-rejected)]',
    bgClass: 'bg-[var(--color-canon-rejected-bg)]',
  },
}[state]);
</script>

<span
  class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-xs font-mono font-medium tracking-tight {config.bgClass} {config.textClass}"
>
  <span class="{config.icon} w-3.5 h-3.5"></span>
  <span>{config.label}</span>
</span>
```

### Example: Timeline Scrubber Node (`timeline-node.svelte`)

```svelte
<script lang="ts">
interface Props {
  universeDay: number;
  label: string;
  active?: boolean;
}

const { universeDay, label, active = false }: Props = $props();
</script>

<button
  type="button"
  class="flex items-center gap-2 px-2.5 py-1 rounded border text-xs transition-colors
  {active 
    ? 'border-[var(--color-crown-red)] bg-[var(--color-crown-red-subtle)] text-[var(--color-text-primary)]' 
    : 'border-[var(--color-border-subtle)] hover:border-[var(--color-border-strong)] text-[var(--color-text-secondary)]'}"
>
  <span class="i-mingcute:history-line w-3.5 h-3.5 text-[var(--color-meridian-timeline)]"></span>
  <span class="font-mono tabular-nums font-semibold">D+{universeDay}</span>
  <span class="text-[var(--color-text-muted)] truncate max-w-32">{label}</span>
</button>
```

---

## 6. Pre-flight Quality Checklist

Before submitting frontend code in Ontara:
- [ ] No raw emojis in markup or strings.
- [ ] Fonts use CSS variables (`var(--font-serif)`, `var(--font-sans)`, `var(--font-mono)`).
- [ ] Strict CSP compliance (no inline script eval, no external web font requests).
- [ ] File names in kebab-case (`.svelte`, `.ts`, `.css`).
- [ ] `just check` passes cleanly (format-check, lint, typecheck, test, deny).
