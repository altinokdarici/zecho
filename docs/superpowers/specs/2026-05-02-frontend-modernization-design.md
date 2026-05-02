# Frontend Modernization & Theming

## Overview

Replace the raw static HTML/CSS/JS frontend with a modern build pipeline (Vite 8 + React + TypeScript + PostCSS), introduce a semantic design token system with automatic light/dark theming, and add a CONTRIBUTING.md with development documentation.

## Goals

- Type-safe frontend with TypeScript and typed Tauri IPC
- Semantic design tokens as a single source of truth, replacing scattered/inconsistent CSS custom properties
- Automatic light/dark theming that follows macOS system preference
- Scoped component styles via CSS Modules
- Unit testing via Vitest
- Clear contribution documentation

## Non-Goals

- Introducing a CSS-in-JS solution or Tailwind
- Changing the Rust backend or Tauri IPC contract
- Adding a manual theme toggle (system preference only)
- Shared component library across pages (extract later if patterns emerge)

## Build Pipeline

### Stack

- **Vite 8** (ships with Rolldown) — bundler, dev server, multi-page mode
- **React 19 + ReactDOM** — component framework
- **TypeScript** — type-checked with tsgo, transpiled by Vite/Rolldown
- **PostCSS** — generates CSS custom properties from token file
- **Vitest** — unit testing, co-located with source
- **Jotai** — lightweight atomic state management
- **pnpm** — package manager

### Tauri Integration

- `tauri.conf.json` updated:
  - `frontendDist` changed from `../ui` to `../ui/dist`
  - `beforeDevCommand`: `pnpm --dir ../ui dev`
  - `beforeBuildCommand`: `pnpm --dir ../ui build`
- `withGlobalTauri: true` retained — Tauri APIs accessed via `window.__TAURI__`
- Multi-page mode: each Tauri window has its own HTML entry point and Vite build entry

## Project Structure

```
ui/
  package.json
  tsconfig.json
  postcss.config.ts
  vite.config.ts
  src/
    tokens/
      tokens.json
      generate-variables.ts
      generated/
        variables.css
    shared/
      theme-atoms.ts
      theme-hook.ts
      tauri-hook.ts
      reset.css
      global.css
    pages/
      pill/
        index.html
        pill-entry.tsx
        pill-app.tsx
        pill-atoms.ts
        components/
          pill-states/
            PillStates.tsx
            PillStates.module.css
            PillStates.test.tsx
          waveform/
            Waveform.tsx
            Waveform.module.css
      settings/
        index.html
        settings-entry.tsx
        settings-app.tsx
        settings-atoms.ts
        components/
          style-cards/
            StyleCards.tsx
            StyleCards.module.css
          cleanup-cards/
            CleanupCards.tsx
            CleanupCards.module.css
          model-list/
            ModelList.tsx
            ModelList.module.css
          model-card/
            ModelCard.tsx
            ModelCard.module.css
          toggle/
            Toggle.tsx
            Toggle.module.css
          custom-prompt/
            CustomPrompt.tsx
            CustomPrompt.module.css
      history/
        index.html
        history-entry.tsx
        history-app.tsx
        history-atoms.ts
        components/
          history-item/
            HistoryItem.tsx
            HistoryItem.module.css
            HistoryItem.test.tsx
          history-detail/
            HistoryDetail.tsx
            HistoryDetail.module.css
      setup/
        index.html
        setup-entry.tsx
        setup-app.tsx
        setup-atoms.ts
        components/
          setup-step/
            SetupStep.tsx
            SetupStep.module.css
            SetupStep.test.tsx
          action-button/
            ActionButton.tsx
            ActionButton.module.css
```

### Naming Conventions

- **Folders**: kebab-case (`pill-states/`, `model-card/`)
- **Component files**: PascalCase (`PillStates.tsx`, `PillStates.module.css`)
- **Non-component files**: kebab-case (`pill-entry.tsx`, `pill-atoms.ts`, `theme-hook.ts`)
- **CSS Modules**: named to match their component (`PillStates.module.css` for `PillStates.tsx`)
- **Test files**: co-located, named to match source (`PillStates.test.tsx`)
- **All filenames are unique across the project** — no two files share the same name in different folders

## Design Tokens

### Token File

`tokens.json` — flat semantic structure with light and dark values per token:

```json
{
  "color": {
    "bg":              { "light": "#ffffff",               "dark": "#1c1c1e" },
    "surface":         { "light": "#f2f2f7",               "dark": "#2c2c2e" },
    "surface-hover":   { "light": "#e5e5ea",               "dark": "#3a3a3c" },
    "text":            { "light": "#1c1c1e",               "dark": "#f5f5f7" },
    "text-secondary":  { "light": "#6c6c70",               "dark": "#98989d" },
    "text-dim":        { "light": "rgba(0,0,0,0.35)",      "dark": "rgba(255,255,255,0.35)" },
    "accent":          { "light": "#6c5ce7",               "dark": "#6c5ce7" },
    "accent-light":    { "light": "rgba(108,92,231,0.10)", "dark": "rgba(108,92,231,0.15)" },
    "blue":            { "light": "#007aff",               "dark": "#4a9eff" },
    "red":             { "light": "#ff3b30",               "dark": "#ff453a" },
    "green":           { "light": "#28cd41",               "dark": "#30d158" },
    "border":          { "light": "rgba(0,0,0,0.08)",      "dark": "rgba(255,255,255,0.08)" },
    "hover":           { "light": "rgba(0,0,0,0.04)",      "dark": "rgba(255,255,255,0.06)" },
    "pill-bg":         { "light": "rgba(255,255,255,0.92)", "dark": "rgba(28,28,30,0.92)" }
  },
  "radius": {
    "sm": "8px",
    "md": "14px",
    "lg": "22px",
    "pill": "4px"
  },
  "shadow": {
    "pill":       { "light": "0 4px 20px rgba(0,0,0,0.12)", "dark": "0 4px 20px rgba(0,0,0,0.5)" },
    "card-hover": { "light": "0 12px 40px rgba(0,0,0,0.08)", "dark": "0 12px 40px rgba(0,0,0,0.3)" }
  }
}
```

### Light Theme Derivation

Light mode values are derived from the existing dark palette using macOS Human Interface Guidelines:

- **Backgrounds**: white/system gray (`#ffffff`, `#f2f2f7`, `#e5e5ea`) instead of dark grays
- **Text**: dark-on-light (`#1c1c1e`, `#6c6c70`) instead of light-on-dark
- **Accents**: same hue, shifted for light-background contrast (e.g. system blue `#007aff` instead of `#4a9eff`)
- **Borders/hovers**: black-based alpha instead of white-based alpha
- **Shadows**: lower opacity for light backgrounds

### CSS Generation

A build-time script (`generate-variables.ts`) reads `tokens.json` and writes `generated/variables.css`:

```css
:root {
  --color-bg: #ffffff;
  --color-surface: #f2f2f7;
  --color-text: #1c1c1e;
  --radius-sm: 8px;
  --radius-md: 14px;
  --shadow-pill: 0 4px 20px rgba(0,0,0,0.12);
  /* ... */
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-bg: #1c1c1e;
    --color-surface: #2c2c2e;
    --color-text: #f5f5f7;
    --shadow-pill: 0 4px 20px rgba(0,0,0,0.5);
    /* ... */
  }
}
```

This runs as a Vite plugin (pre-build step). Components reference tokens as `var(--color-bg)`, `var(--radius-md)`, etc.

### Pill Transparency

The pill window uses `rgba` backgrounds with `backdrop-filter: blur()`. The pill-specific `--pill-bg` token uses `rgba` values in both themes:

```json
"pill-bg": { "light": "rgba(255,255,255,0.92)", "dark": "rgba(28,28,30,0.92)" }
```

## Theming

### System Preference Detection

A `useTheme` hook in `theme-hook.ts` watches `matchMedia('(prefers-color-scheme: dark)')`:

- On load and on change events, sets `data-theme="light"` or `data-theme="dark"` on `<html>`
- The actual theming is handled entirely by the CSS `@media (prefers-color-scheme: dark)` block — the `data-theme` attribute is available for edge cases where JS needs to know the current theme
- No manual toggle — follows system preference only

### Theme Atom

A Jotai atom (`theme-atoms.ts`) exposes the current theme as reactive state for components that need conditional logic based on theme (e.g. the waveform canvas drawing colors):

```ts
export const themeAtom = atom<"light" | "dark">("dark");
```

Hydrated by `useTheme` on mount and on media query change events.

## State Management

### Jotai Atoms

One atoms file per page, shared atoms in `shared/`:

| File | Atoms | Purpose |
|------|-------|---------|
| `theme-atoms.ts` | `themeAtom` | Current light/dark theme |
| `pill-atoms.ts` | `recordingStateAtom`, `recordingLockedAtom`, `barLevelsAtom` | Pill UI state machine |
| `settings-atoms.ts` | `settingsAtom`, `activeWhisperModelAtom`, `activeCleanupModelAtom` | Settings + model state |
| `history-atoms.ts` | `historyItemsAtom` | History list |
| `setup-atoms.ts` | `setupStateAtom` (accessibility, microphone, whisper, cleanup, downloading) | Setup wizard state |

### Tauri Event Hydration

Each page's root component (`pill-app.tsx`, etc.) sets up Tauri event listeners in a `useEffect` that hydrate atoms via `useSetAtom`. IPC calls (`invoke`) happen in event handlers, not inside atoms.

## React Components

### Pill Page

| Component | Responsibility |
|-----------|---------------|
| `PillStates` | Renders the correct state view (idle/recording/processing/done/setup) based on `recordingStateAtom`. Idle shows history + settings buttons. Recording shows cancel + waveform + stop. |
| `Waveform` | Canvas-based audio level visualization. Reads `barLevelsAtom`, polls `get_audio_level` via invoke on an interval during recording. Reads `themeAtom` for bar colors. |

### Settings Page

| Component | Responsibility |
|-----------|---------------|
| `StyleCards` | 3-column card grid for writing style selection (Formal/Casual/Very Casual) |
| `CleanupCards` | 2-column card grid for cleanup level (None/Light/Medium/High) |
| `ModelList` | Lists models for a category (Whisper or Cleanup), renders `ModelCard` for each |
| `ModelCard` | Single model with name, description, quality/speed dots, download/use/active button |
| `Toggle` | Reusable toggle switch (used for cleanup enabled, auto-paste) |
| `CustomPrompt` | Debounced textarea for custom cleanup instructions |

### History Page

| Component | Responsibility |
|-----------|---------------|
| `HistoryItem` | Single history entry — truncated text, timestamp, timing info. Click to copy. |
| `HistoryDetail` | Expandable raw vs. cleaned comparison panel |

### Setup Page

| Component | Responsibility |
|-----------|---------------|
| `SetupStep` | Reusable step row — icon (pending/active/done), title, description, status text |
| `ActionButton` | Context-aware primary button that advances through the setup flow |

## Testing

### Vitest Configuration

- Configured in `vite.config.ts` via `vitest` plugin
- jsdom environment for component tests
- Co-located test files: `ComponentName.test.tsx` next to `ComponentName.tsx`

### What Gets Tests

- Utility functions (time formatting, HTML escaping, token generation script)
- Jotai atoms with derived logic
- Components with interaction logic (card selection, toggle, model download flow, setup step progression)
- `useTheme` hook (media query response)

### What Doesn't Get Tests

- Purely presentational components with no logic
- Direct Tauri IPC wrappers

### Tauri Mocking

A shared test utility (`src/shared/test-utils/tauri-mock.ts`) mocks `window.__TAURI__` with typed stubs for `invoke` and `listen`, so components can be tested without a Tauri runtime.

## CONTRIBUTING.md

A `CONTRIBUTING.md` at the repo root covering:

- **Prerequisites**: Rust toolchain, Tauri CLI v2, Node.js 20+, pnpm
- **Dev setup**: clone, `pnpm install` in `ui/`, `./scripts/download-models.sh`, `cargo tauri dev`
- **Running tests**: `cargo test` (Rust, from `src-tauri/`), `pnpm test` (frontend, from `ui/`)
- **Project structure**: overview of `src-tauri/` and `ui/` layout
- **Code conventions**: kebab-case folders, PascalCase components, co-located tests and CSS Modules, unique filenames

The README is slimmed to: what the app does, features, download link, license, and a pointer to `CONTRIBUTING.md` for development setup.

## Migration Notes

- The existing `ui/` folder contents (`index.html`, `main.js`, `styles.css`, `settings.html`, `history.html`, `setup.html`) are replaced entirely by the new React app
- All existing UI behavior is preserved — this is a 1:1 port to React + TypeScript, not a redesign
- The Tauri IPC contract (command names, event names, payload shapes) does not change
- `docs/design/system.md` remains as reference documentation but is no longer the source of truth for tokens — `tokens.json` is
