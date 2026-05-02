# Frontend Modernization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the raw static HTML/CSS/JS frontend with Vite 8 + React + TypeScript + PostCSS, add semantic design tokens with automatic light/dark theming, and create contribution documentation.

**Architecture:** Multi-page Vite app where each Tauri window (pill, settings, history, setup) is a separate React entry point. A single `tokens.json` file drives all design tokens, with a build-time script generating CSS custom properties for light and dark themes via `@media (prefers-color-scheme: dark)`. Jotai manages per-page state. CSS Modules scope component styles.

**Tech Stack:** Vite 8 (Rolldown), React 19, TypeScript, PostCSS, Vitest, Jotai, pnpm

**Spec:** `docs/superpowers/specs/2026-05-02-frontend-modernization-design.md`

---

## File Map

### New files to create

```
ui/
  package.json                                    — dependencies, scripts (dev, build, test, generate-tokens)
  tsconfig.json                                   — TS config with paths, JSX, strict mode
  vite.config.ts                                  — multi-page config, Vitest, token generation plugin
  src/
    tokens/
      tokens.json                                 — semantic design tokens (light + dark values)
      generate-variables.ts                       — reads tokens.json, writes generated/variables.css
      generate-variables.test.ts                  — tests for token generation
      generated/
        variables.css                             — auto-generated CSS custom properties (do not edit)
    shared/
      theme-atoms.ts                              — Jotai atom for current theme
      theme-hook.ts                               — useTheme hook: watches prefers-color-scheme
      theme-hook.test.ts                          — tests for useTheme
      tauri-hook.ts                               — typed wrappers for invoke/listen
      tauri-types.ts                              — TypeScript types for all Tauri IPC commands/events
      test-utils/
        tauri-mock.ts                             — mock window.__TAURI__ for Vitest
      reset.css                                   — minimal CSS reset
      global.css                                  — imports variables.css + reset.css + base styles
    pages/
      pill/
        index.html                                — HTML entry for pill window
        pill-entry.tsx                             — React root mount
        pill-app.tsx                               — Pill root component, Tauri event wiring
        pill-atoms.ts                              — pillStateAtom, recordingLockedAtom, barLevelsAtom
        components/
          pill-states/
            PillStates.tsx                         — state machine UI (idle/recording/processing/done/setup)
            PillStates.module.css                  — pill state styles
            PillStates.test.tsx                    — state rendering tests
          waveform/
            Waveform.tsx                           — canvas audio level visualizer
            Waveform.module.css                    — waveform styles
      settings/
        index.html                                — HTML entry for settings window
        settings-entry.tsx                         — React root mount
        settings-app.tsx                           — Settings root, tab navigation, Tauri event wiring
        settings-atoms.ts                          — settingsAtom, model atoms
        components/
          style-cards/
            StyleCards.tsx                          — writing style selection grid
            StyleCards.module.css
          cleanup-cards/
            CleanupCards.tsx                        — cleanup level selection grid
            CleanupCards.module.css
          model-list/
            ModelList.tsx                           — model category list, renders ModelCards
            ModelList.module.css
          model-card/
            ModelCard.tsx                           — individual model with download/select/active
            ModelCard.module.css
          toggle/
            Toggle.tsx                             — reusable toggle switch
            Toggle.module.css
            Toggle.test.tsx                        — toggle interaction tests
          custom-prompt/
            CustomPrompt.tsx                       — debounced textarea for custom instructions
            CustomPrompt.module.css
      history/
        index.html                                — HTML entry for history window
        history-entry.tsx                          — React root mount
        history-app.tsx                            — History root, Tauri event wiring
        history-atoms.ts                           — historyItemsAtom
        components/
          history-item/
            HistoryItem.tsx                        — single entry, click-to-copy, expandable
            HistoryItem.module.css
            HistoryItem.test.tsx                   — interaction tests
          history-detail/
            HistoryDetail.tsx                      — raw vs cleaned comparison
            HistoryDetail.module.css
      setup/
        index.html                                — HTML entry for setup window
        setup-entry.tsx                            — React root mount
        setup-app.tsx                              — Setup root, sequential flow, Tauri events
        setup-atoms.ts                             — setupStateAtom
        components/
          setup-step/
            SetupStep.tsx                          — step row (pending/active/done)
            SetupStep.module.css
            SetupStep.test.tsx                     — step state rendering tests
          action-button/
            ActionButton.tsx                       — context-aware primary button
            ActionButton.module.css
```

### Files to modify

```
src-tauri/tauri.conf.json                         — frontendDist, beforeDevCommand, beforeBuildCommand, window URLs
CONTRIBUTING.md                                   — new file at repo root
README.md                                         — slim down, point to CONTRIBUTING.md
```

### Files to remove (final task)

```
ui/index.html
ui/main.js
ui/styles.css
ui/settings.html
ui/history.html
ui/setup.html
```

---

## Task 1: Scaffold Vite + React + TypeScript Project

**Files:**
- Create: `ui/package.json`
- Create: `ui/tsconfig.json`
- Create: `ui/vite.config.ts`

- [ ] **Step 1: Create package.json**

```json
{
  "name": "zecho-ui",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "test": "vitest run",
    "test:watch": "vitest",
    "generate-tokens": "tsx src/tokens/generate-variables.ts"
  },
  "dependencies": {
    "jotai": "^2.12.0",
    "react": "^19.1.0",
    "react-dom": "^19.1.0"
  },
  "devDependencies": {
    "@testing-library/jest-dom": "^6.6.0",
    "@testing-library/react": "^16.3.0",
    "@types/react": "^19.1.0",
    "@types/react-dom": "^19.1.0",
    "jsdom": "^26.1.0",
    "tsx": "^4.19.0",
    "typescript": "^5.8.0",
    "vite": "^8.0.0",
    "vitest": "^3.2.0"
  }
}
```

- [ ] **Step 2: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "skipLibCheck": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "esModuleInterop": true,
    "paths": {
      "@shared/*": ["./src/shared/*"],
      "@tokens/*": ["./src/tokens/*"]
    },
    "baseUrl": "."
  },
  "include": ["src"]
}
```

- [ ] **Step 3: Create vite.config.ts**

```ts
import { resolve } from "path";
import { defineConfig } from "vite";

export default defineConfig({
  resolve: {
    alias: {
      "@shared": resolve(__dirname, "src/shared"),
      "@tokens": resolve(__dirname, "src/tokens"),
    },
  },
  build: {
    rollupOptions: {
      input: {
        pill: resolve(__dirname, "src/pages/pill/index.html"),
        settings: resolve(__dirname, "src/pages/settings/index.html"),
        history: resolve(__dirname, "src/pages/history/index.html"),
        setup: resolve(__dirname, "src/pages/setup/index.html"),
      },
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: [],
  },
});
```

- [ ] **Step 4: Install dependencies**

Run: `cd ui && pnpm install`
Expected: `node_modules` created, lockfile generated

- [ ] **Step 5: Verify Vite starts without errors**

Run: `cd ui && pnpm exec vite --version`
Expected: prints Vite version 8.x

- [ ] **Step 6: Commit**

```bash
git add ui/package.json ui/pnpm-lock.yaml ui/tsconfig.json ui/vite.config.ts ui/node_modules/.modules.yaml
git commit -m "feat(ui): scaffold Vite 8 + React + TypeScript project"
```

Note: `ui/node_modules` should be in `.gitignore`. Check that `.gitignore` covers it; if not, add `ui/node_modules/` to `.gitignore` before committing.

---

## Task 2: Design Token System

**Files:**
- Create: `ui/src/tokens/tokens.json`
- Create: `ui/src/tokens/generate-variables.ts`
- Create: `ui/src/tokens/generate-variables.test.ts`
- Create: `ui/src/tokens/generated/variables.css`

- [ ] **Step 1: Create tokens.json**

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

- [ ] **Step 2: Write the failing test for generate-variables**

```ts
// ui/src/tokens/generate-variables.test.ts
import { describe, it, expect } from "vitest";
import { generateCSS } from "./generate-variables";
import tokens from "./tokens.json";

describe("generateCSS", () => {
  const css = generateCSS(tokens);

  it("generates :root block with light values", () => {
    expect(css).toContain(":root {");
    expect(css).toContain("--color-bg: #ffffff;");
    expect(css).toContain("--color-text: #1c1c1e;");
    expect(css).toContain("--radius-sm: 8px;");
    expect(css).toContain("--shadow-pill: 0 4px 20px rgba(0,0,0,0.12);");
  });

  it("generates dark media query with dark values", () => {
    expect(css).toContain("@media (prefers-color-scheme: dark)");
    expect(css).toContain("--color-bg: #1c1c1e;");
    expect(css).toContain("--color-text: #f5f5f7;");
    expect(css).toContain("--shadow-pill: 0 4px 20px rgba(0,0,0,0.5);");
  });

  it("includes non-themed tokens only in :root", () => {
    expect(css).toContain("--radius-md: 14px;");
    const darkBlock = css.split("@media")[1];
    expect(darkBlock).not.toContain("--radius-md:");
  });

  it("uses category-name format for variable names", () => {
    expect(css).toContain("--color-surface-hover:");
    expect(css).toContain("--color-accent-light:");
    expect(css).toContain("--color-pill-bg:");
    expect(css).toContain("--shadow-card-hover:");
  });
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd ui && pnpm test -- src/tokens/generate-variables.test.ts`
Expected: FAIL — `generateCSS` not found

- [ ] **Step 4: Implement generate-variables.ts**

```ts
// ui/src/tokens/generate-variables.ts
import { readFileSync, writeFileSync, mkdirSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

interface ThemedValue {
  light: string;
  dark: string;
}

type TokenValue = string | ThemedValue;
type TokenCategory = Record<string, TokenValue>;
type Tokens = Record<string, TokenCategory>;

function isThemed(value: TokenValue): value is ThemedValue {
  return typeof value === "object" && "light" in value && "dark" in value;
}

export function generateCSS(tokens: Tokens): string {
  const lightVars: string[] = [];
  const darkVars: string[] = [];
  const staticVars: string[] = [];

  for (const [category, entries] of Object.entries(tokens)) {
    for (const [name, value] of Object.entries(entries)) {
      const varName = `--${category}-${name}`;
      if (isThemed(value)) {
        lightVars.push(`  ${varName}: ${value.light};`);
        darkVars.push(`  ${varName}: ${value.dark};`);
      } else {
        staticVars.push(`  ${varName}: ${value};`);
      }
    }
  }

  const lines: string[] = [
    "/* Auto-generated from tokens.json — do not edit */",
    ":root {",
    ...lightVars,
    ...staticVars,
    "}",
    "",
    "@media (prefers-color-scheme: dark) {",
    "  :root {",
    ...darkVars.map((v) => `  ${v}`),
    "  }",
    "}",
    "",
  ];

  return lines.join("\n");
}

if (typeof process !== "undefined" && process.argv[1]) {
  const currentFile = fileURLToPath(import.meta.url);
  if (process.argv[1] === currentFile || process.argv[1].endsWith("/generate-variables.ts")) {
    const tokensPath = resolve(dirname(currentFile), "tokens.json");
    const outPath = resolve(dirname(currentFile), "generated", "variables.css");
    const tokens = JSON.parse(readFileSync(tokensPath, "utf-8"));
    mkdirSync(dirname(outPath), { recursive: true });
    writeFileSync(outPath, generateCSS(tokens));
    console.log(`Generated ${outPath}`);
  }
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cd ui && pnpm test -- src/tokens/generate-variables.test.ts`
Expected: PASS (all 4 tests)

- [ ] **Step 6: Generate the initial variables.css**

Run: `cd ui && pnpm generate-tokens`
Expected: prints path to generated file, `src/tokens/generated/variables.css` is created

- [ ] **Step 7: Commit**

```bash
git add ui/src/tokens/
git commit -m "feat(ui): add design token system with light/dark CSS generation"
```

---

## Task 3: Shared Styles and Theme Infrastructure

**Files:**
- Create: `ui/src/shared/reset.css`
- Create: `ui/src/shared/global.css`
- Create: `ui/src/shared/tauri-types.ts`
- Create: `ui/src/shared/tauri-hook.ts`
- Create: `ui/src/shared/test-utils/tauri-mock.ts`
- Create: `ui/src/shared/theme-atoms.ts`
- Create: `ui/src/shared/theme-hook.ts`
- Create: `ui/src/shared/theme-hook.test.ts`

- [ ] **Step 1: Create reset.css**

```css
/* ui/src/shared/reset.css */
*,
*::before,
*::after {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}
```

- [ ] **Step 2: Create global.css**

```css
/* ui/src/shared/global.css */
@import "../tokens/generated/variables.css";
@import "./reset.css";

html, body {
  font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display", "Helvetica Neue", system-ui, sans-serif;
  color: var(--color-text);
  -webkit-user-select: none;
  user-select: none;
  cursor: default;
  height: 100%;
  overflow: hidden;
}

.hidden {
  display: none !important;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
```

- [ ] **Step 3: Create tauri-types.ts**

These types are derived from the existing Rust backend commands and events.

```ts
// ui/src/shared/tauri-types.ts

// --- Invoke command return types ---

export interface SetupStatus {
  whisper_ready: boolean;
  cleanup_ready: boolean;
}

export interface Settings {
  writing_style: "Formal" | "Casual" | "VeryCasual";
  cleanup_level: "None" | "Light" | "Medium" | "High";
  auto_paste: boolean;
  custom_prompt: string | null;
  active_whisper_model: string;
  active_cleanup_model: string;
}

export interface Model {
  id: string;
  name: string;
  description: string;
  model_type: "Whisper" | "Cleanup";
  downloaded: boolean;
  size_mb: number;
  quality_score: number;
  speed_score: number;
}

export interface HistoryEntry {
  id: string;
  text: string;
  raw_text: string;
  created_at: string;
  transcribe_ms: number;
  cleanup_ms: number;
}

// --- Invoke command signatures ---

export type InvokeCommands = {
  start_recording: { args: void; return: void };
  stop_recording: { args: void; return: void };
  cancel_recording: { args: void; return: void };
  get_audio_level: { args: void; return: number };
  check_setup: { args: void; return: SetupStatus };
  setup_download_models: { args: void; return: void };
  get_settings: { args: void; return: Settings };
  update_settings: { args: { newSettings: Settings }; return: void };
  list_models: { args: void; return: Model[] };
  download_model: { args: { modelId: string }; return: void };
  load_whisper_model_cmd: { args: { modelId: string }; return: void };
  load_cleanup_model: { args: { modelId: string }; return: void };
  get_history: { args: void; return: HistoryEntry[] };
  copy_history_item: { args: { id: string }; return: void };
  clear_history: { args: void; return: void };
  toggle_history: { args: void; return: void };
  open_settings: { args: void; return: void };
  start_drag: { args: void; return: void };
  persist_pill_position: { args: void; return: void };
  check_accessibility: { args: void; return: boolean };
  open_accessibility_settings: { args: void; return: void };
  start_fn_listener: { args: void; return: void };
  request_microphone: { args: void; return: void };
  complete_setup: { args: void; return: void };
  hide_setup: { args: void; return: void };
};

// --- Event payload types ---

export type EventPayloads = {
  "pill-hover": boolean;
  "fn-key-down": void;
  "fn-key-up": void;
  "toggle-recording": void;
  "cancel-recording": void;
  "transcription-complete": void;
  "transcription-error": string;
  "setup-progress": string;
  "setup-complete": void;
  "setup-error": string;
  "model-download-complete": void;
  "model-download-error": string;
};
```

- [ ] **Step 4: Create tauri-hook.ts**

```ts
// ui/src/shared/tauri-hook.ts
import type { InvokeCommands, EventPayloads } from "./tauri-types";

type TauriCore = {
  invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
};

type TauriEvent = {
  listen: (
    event: string,
    handler: (event: { payload: unknown }) => void
  ) => Promise<() => void>;
};

function getCore(): TauriCore {
  return (window as any).__TAURI__.core;
}

function getEvent(): TauriEvent {
  return (window as any).__TAURI__.event;
}

export async function invoke<K extends keyof InvokeCommands>(
  cmd: K,
  ...args: InvokeCommands[K]["args"] extends void ? [] : [InvokeCommands[K]["args"]]
): Promise<InvokeCommands[K]["return"]> {
  return getCore().invoke(cmd, args[0] as any) as any;
}

export function listen<K extends keyof EventPayloads>(
  event: K,
  handler: EventPayloads[K] extends void
    ? () => void
    : (payload: EventPayloads[K]) => void
): Promise<() => void> {
  return getEvent().listen(event, (e) => {
    (handler as any)(e.payload);
  });
}
```

- [ ] **Step 5: Create tauri-mock.ts**

```ts
// ui/src/shared/test-utils/tauri-mock.ts
import { vi } from "vitest";

type MockInvokeHandler = (cmd: string, args?: any) => any;

let invokeHandler: MockInvokeHandler = () => undefined;

export function mockInvoke(handler: MockInvokeHandler) {
  invokeHandler = handler;
}

export function mockListen() {
  return vi.fn(() => Promise.resolve(() => {}));
}

export function installTauriMock() {
  const listenMock = mockListen();

  (globalThis as any).__TAURI__ = {
    core: {
      invoke: vi.fn((cmd: string, args?: any) =>
        Promise.resolve(invokeHandler(cmd, args))
      ),
    },
    event: {
      listen: listenMock,
    },
    window: {
      getCurrentWindow: vi.fn(() => ({
        hide: vi.fn(() => Promise.resolve()),
      })),
    },
  };

  return {
    invoke: (globalThis as any).__TAURI__.core.invoke,
    listen: listenMock,
  };
}

export function clearTauriMock() {
  delete (globalThis as any).__TAURI__;
  invokeHandler = () => undefined;
}
```

- [ ] **Step 6: Write the failing test for useTheme**

```ts
// ui/src/shared/theme-hook.test.ts
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { Provider } from "jotai";
import { useHydrateAtoms } from "jotai/utils";
import { useTheme } from "./theme-hook";
import { themeAtom } from "./theme-atoms";
import type { ReactNode } from "react";
import React from "react";

let listeners: Array<(e: { matches: boolean }) => void> = [];
let currentMatches = false;

beforeEach(() => {
  listeners = [];
  currentMatches = false;
  vi.stubGlobal(
    "matchMedia",
    vi.fn((query: string) => ({
      matches: currentMatches,
      media: query,
      addEventListener: (_: string, cb: (e: { matches: boolean }) => void) => {
        listeners.push(cb);
      },
      removeEventListener: (_: string, cb: (e: { matches: boolean }) => void) => {
        listeners = listeners.filter((l) => l !== cb);
      },
    }))
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
});

function wrapper({ children }: { children: ReactNode }) {
  return React.createElement(Provider, null, children);
}

describe("useTheme", () => {
  it("returns 'light' when system is light mode", () => {
    currentMatches = false;
    const { result } = renderHook(() => useTheme(), { wrapper });
    expect(result.current).toBe("light");
  });

  it("returns 'dark' when system is dark mode", () => {
    currentMatches = true;
    const { result } = renderHook(() => useTheme(), { wrapper });
    expect(result.current).toBe("dark");
  });

  it("reacts to system preference changes", () => {
    currentMatches = false;
    const { result } = renderHook(() => useTheme(), { wrapper });
    expect(result.current).toBe("light");

    act(() => {
      listeners.forEach((cb) => cb({ matches: true }));
    });
    expect(result.current).toBe("dark");
  });
});
```

- [ ] **Step 7: Run test to verify it fails**

Run: `cd ui && pnpm test -- src/shared/theme-hook.test.ts`
Expected: FAIL — modules not found

- [ ] **Step 8: Create theme-atoms.ts and theme-hook.ts**

```ts
// ui/src/shared/theme-atoms.ts
import { atom } from "jotai";

export const themeAtom = atom<"light" | "dark">("light");
```

```ts
// ui/src/shared/theme-hook.ts
import { useEffect } from "react";
import { useAtom } from "jotai";
import { themeAtom } from "./theme-atoms";

const DARK_QUERY = "(prefers-color-scheme: dark)";

export function useTheme(): "light" | "dark" {
  const [theme, setTheme] = useAtom(themeAtom);

  useEffect(() => {
    const mql = window.matchMedia(DARK_QUERY);

    function update(matches: boolean) {
      const next = matches ? "dark" : "light";
      setTheme(next);
      document.documentElement.setAttribute("data-theme", next);
    }

    update(mql.matches);

    function onChange(e: MediaQueryListEvent | { matches: boolean }) {
      update(e.matches);
    }

    mql.addEventListener("change", onChange);
    return () => mql.removeEventListener("change", onChange);
  }, [setTheme]);

  return theme;
}
```

- [ ] **Step 9: Run test to verify it passes**

Run: `cd ui && pnpm test -- src/shared/theme-hook.test.ts`
Expected: PASS (all 3 tests)

Note: if `jotai/utils` or `useHydrateAtoms` import fails in tests, remove the unused import — the test uses `Provider` from `jotai` directly.

- [ ] **Step 10: Commit**

```bash
git add ui/src/shared/ ui/src/tokens/generated/
git commit -m "feat(ui): add shared infrastructure — theme, Tauri hooks, test utils, global styles"
```

---

## Task 4: Pill Page

**Files:**
- Create: `ui/src/pages/pill/index.html`
- Create: `ui/src/pages/pill/pill-entry.tsx`
- Create: `ui/src/pages/pill/pill-app.tsx`
- Create: `ui/src/pages/pill/pill-atoms.ts`
- Create: `ui/src/pages/pill/components/pill-states/PillStates.tsx`
- Create: `ui/src/pages/pill/components/pill-states/PillStates.module.css`
- Create: `ui/src/pages/pill/components/pill-states/PillStates.test.tsx`
- Create: `ui/src/pages/pill/components/waveform/Waveform.tsx`
- Create: `ui/src/pages/pill/components/waveform/Waveform.module.css`

- [ ] **Step 1: Create pill-atoms.ts**

```tsx
// ui/src/pages/pill/pill-atoms.ts
import { atom } from "jotai";

export type PillState = "idle" | "recording" | "processing" | "done" | "setup";

export const pillStateAtom = atom<PillState>("idle");
export const recordingLockedAtom = atom(false);
export const barLevelsAtom = atom<number[]>(new Array(16).fill(0));
```

- [ ] **Step 2: Write the failing test for PillStates**

```tsx
// ui/src/pages/pill/components/pill-states/PillStates.test.tsx
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { Provider, createStore } from "jotai";
import React from "react";
import { PillStates } from "./PillStates";
import { pillStateAtom } from "../../pill-atoms";

function renderWithState(state: string) {
  const store = createStore();
  store.set(pillStateAtom, state as any);
  return render(
    React.createElement(
      Provider,
      { store },
      React.createElement(PillStates, {
        onToggleHistory: () => {},
        onOpenSettings: () => {},
        onCancel: () => {},
        onStop: () => {},
      })
    )
  );
}

describe("PillStates", () => {
  it("renders idle state with history and settings buttons", () => {
    renderWithState("idle");
    expect(screen.getByLabelText("History")).toBeDefined();
    expect(screen.getByLabelText("Settings")).toBeDefined();
  });

  it("renders recording state with cancel and stop buttons", () => {
    renderWithState("recording");
    expect(screen.getByLabelText("Cancel")).toBeDefined();
    expect(screen.getByLabelText("Stop")).toBeDefined();
  });

  it("renders processing state with processing label", () => {
    renderWithState("processing");
    expect(screen.getByText("Processing")).toBeDefined();
  });

  it("renders done state with copied label", () => {
    renderWithState("done");
    expect(screen.getByText("Copied")).toBeDefined();
  });
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd ui && pnpm test -- src/pages/pill/components/pill-states/PillStates.test.tsx`
Expected: FAIL — PillStates module not found

- [ ] **Step 4: Create PillStates.tsx**

```tsx
// ui/src/pages/pill/components/pill-states/PillStates.tsx
import { useAtomValue } from "jotai";
import { pillStateAtom } from "../../pill-atoms";
import { Waveform } from "../waveform/Waveform";
import styles from "./PillStates.module.css";

interface PillStatesProps {
  onToggleHistory: () => void;
  onOpenSettings: () => void;
  onCancel: () => void;
  onStop: () => void;
}

export function PillStates({
  onToggleHistory,
  onOpenSettings,
  onCancel,
  onStop,
}: PillStatesProps) {
  const state = useAtomValue(pillStateAtom);

  if (state === "idle") {
    return (
      <div className={styles.idle}>
        <button
          className={styles.pillAction}
          aria-label="History"
          onClick={onToggleHistory}
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path
              d="M8 3.5V8L10.5 10.5"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
            />
            <circle
              cx="8"
              cy="8"
              r="6"
              stroke="currentColor"
              strokeWidth="1.5"
              fill="none"
            />
          </svg>
        </button>
        <div className={styles.idleDot} />
        <button
          className={styles.pillAction}
          aria-label="Settings"
          onClick={onOpenSettings}
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path
              d="M6.6 1.2h2.8l.4 1.9.7.3 1.7-1 2 2-1 1.7.3.7 1.9.4v2.8l-1.9.4-.3.7 1 1.7-2 2-1.7-1-.7.3-.4 1.9H6.6l-.4-1.9-.7-.3-1.7 1-2-2 1-1.7-.3-.7L.6 9.8V7l1.9-.4.3-.7-1-1.7 2-2 1.7 1 .7-.3.4-1.9z"
              stroke="currentColor"
              strokeWidth="1.2"
              fill="none"
              strokeLinejoin="round"
            />
            <circle
              cx="8"
              cy="8.4"
              r="2"
              stroke="currentColor"
              strokeWidth="1.2"
              fill="none"
            />
          </svg>
        </button>
      </div>
    );
  }

  if (state === "recording") {
    return (
      <div className={styles.recording}>
        <button
          className={`${styles.recBtn} ${styles.cancelBtn}`}
          aria-label="Cancel"
          onClick={onCancel}
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path
              d="M3.5 3.5L10.5 10.5M10.5 3.5L3.5 10.5"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
            />
          </svg>
        </button>
        <Waveform />
        <button
          className={`${styles.recBtn} ${styles.stopBtn}`}
          aria-label="Stop"
          onClick={onStop}
        >
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <rect x="1" y="1" width="10" height="10" rx="2" fill="currentColor" />
          </svg>
        </button>
      </div>
    );
  }

  if (state === "processing") {
    return (
      <div className={styles.processing}>
        <div className={styles.spinner} />
        <span className={styles.pillLabel}>Processing</span>
      </div>
    );
  }

  if (state === "done") {
    return (
      <div className={styles.done}>
        <svg width="16" height="16" viewBox="0 0 18 18" fill="none">
          <path
            d="M4 9L7.5 12.5L14 5.5"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
        <span className={styles.pillLabel}>Copied</span>
      </div>
    );
  }

  if (state === "setup") {
    return (
      <div className={styles.setup}>
        <div className={styles.setupSpinner} />
        <span className={styles.pillLabel}>Setting up...</span>
      </div>
    );
  }

  return null;
}
```

- [ ] **Step 5: Create PillStates.module.css**

Port the existing pill styles from `ui/styles.css`, replacing hardcoded colors with token variables. This file contains styles for all pill states: idle, recording, processing, done, setup.

```css
/* ui/src/pages/pill/components/pill-states/PillStates.module.css */

.idle {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.idleDot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--color-text-dim);
  flex-shrink: 0;
}

.pillAction {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: none;
  background: transparent;
  color: var(--color-text-dim);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.12s ease;
  padding: 0;
  flex-shrink: 0;
}

.pillAction:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.recording {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  gap: 6px;
  animation: fadeIn 0.15s ease;
}

.recBtn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.12s ease;
  flex-shrink: 0;
  padding: 0;
}

.cancelBtn {
  background: rgba(255, 255, 255, 0.1);
  color: var(--color-text-dim);
}

.cancelBtn:hover {
  background: rgba(255, 255, 255, 0.18);
  color: var(--color-text);
}

.stopBtn {
  background: rgba(255, 69, 58, 0.15);
  color: var(--color-red);
}

.stopBtn:hover {
  background: rgba(255, 69, 58, 0.25);
}

.processing,
.done,
.setup {
  display: flex;
  align-items: center;
  gap: 8px;
  animation: fadeIn 0.15s ease;
}

.done {
  color: var(--color-green);
}

.pillLabel {
  font-size: 12px;
  font-weight: 500;
  letter-spacing: -0.01em;
  white-space: nowrap;
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--color-text-dim);
  border-top-color: var(--color-text);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  flex-shrink: 0;
}

.setupSpinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--color-accent-light);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  flex-shrink: 0;
}
```

- [ ] **Step 6: Create Waveform.tsx and Waveform.module.css**

```tsx
// ui/src/pages/pill/components/waveform/Waveform.tsx
import { useRef, useEffect } from "react";
import { useAtomValue } from "jotai";
import { barLevelsAtom } from "../../pill-atoms";
import { themeAtom } from "@shared/theme-atoms";
import styles from "./Waveform.module.css";

export function Waveform() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const barLevels = useAtomValue(barLevelsAtom);
  const theme = useAtomValue(themeAtom);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const w = canvas.width;
    const h = canvas.height;
    const bars = 16;
    const barW = 3;
    const totalWidth = bars * barW + (bars - 1) * 2;
    const offsetX = (w - totalWidth) / 2;

    ctx.clearRect(0, 0, w, h);
    for (let i = 0; i < bars; i++) {
      const amp = 0.08 + barLevels[i] * 0.92;
      const barH = amp * h * 0.9;
      const x = offsetX + i * (barW + 2);
      const y = (h - barH) / 2;
      const alpha = 0.5 + barLevels[i] * 0.5;
      const r = Math.round(200 + barLevels[i] * 55);
      const g = Math.round(200 + barLevels[i] * 30);
      const b = 255;
      ctx.fillStyle =
        theme === "dark"
          ? `rgba(${r}, ${g}, ${b}, ${alpha})`
          : `rgba(${80 - barLevels[i] * 30}, ${70 - barLevels[i] * 20}, ${180 + barLevels[i] * 40}, ${alpha})`;
      ctx.beginPath();
      ctx.roundRect(x, y, barW, barH, 1.5);
      ctx.fill();
    }
  }, [barLevels, theme]);

  return (
    <canvas
      ref={canvasRef}
      className={styles.waveform}
      width={120}
      height={22}
    />
  );
}
```

```css
/* ui/src/pages/pill/components/waveform/Waveform.module.css */

.waveform {
  flex: 1;
  height: 22px;
  min-width: 0;
}
```

- [ ] **Step 7: Run PillStates test to verify it passes**

Run: `cd ui && pnpm test -- src/pages/pill/components/pill-states/PillStates.test.tsx`
Expected: PASS (all 4 tests)

- [ ] **Step 8: Create pill-app.tsx**

```tsx
// ui/src/pages/pill/pill-app.tsx
import { useEffect, useCallback } from "react";
import { useSetAtom, useAtomValue } from "jotai";
import { pillStateAtom, recordingLockedAtom, barLevelsAtom } from "./pill-atoms";
import { useTheme } from "@shared/theme-hook";
import { invoke, listen } from "@shared/tauri-hook";
import { PillStates } from "./components/pill-states/PillStates";
import type { PillState } from "./pill-atoms";

const DOUBLE_TAP_MS = 400;

export function PillApp() {
  useTheme();

  const setPillState = useSetAtom(pillStateAtom);
  const setRecordingLocked = useSetAtom(recordingLockedAtom);
  const setBarLevels = useSetAtom(barLevelsAtom);
  const pillState = useAtomValue(pillStateAtom);
  const recordingLocked = useAtomValue(recordingLockedAtom);

  const isRecording = pillState === "recording";

  const startRecording = useCallback(async () => {
    if (isRecording) return;
    try {
      await invoke("start_recording");
      setPillState("recording");
    } catch (err) {
      console.error("Start error:", err);
    }
  }, [isRecording, setPillState]);

  const stopRecording = useCallback(async () => {
    if (!isRecording) return;
    setRecordingLocked(false);
    setPillState("processing");
    try {
      await invoke("stop_recording");
    } catch (err) {
      console.error("Stop error:", err);
      setPillState("idle");
    }
  }, [isRecording, setPillState, setRecordingLocked]);

  const cancelRecording = useCallback(async () => {
    if (!isRecording) return;
    setRecordingLocked(false);
    try {
      await invoke("cancel_recording");
    } catch (err) {
      console.error("Cancel error:", err);
    }
    setPillState("idle");
  }, [isRecording, setPillState, setRecordingLocked]);

  // Audio level polling during recording
  useEffect(() => {
    if (!isRecording) return;
    const interval = setInterval(async () => {
      try {
        const level = await invoke("get_audio_level");
        const normalized = Math.min(1, Math.pow(level * 15, 0.7));
        setBarLevels((prev) => [...prev.slice(1), normalized]);
      } catch {}
    }, 50);
    return () => clearInterval(interval);
  }, [isRecording, setBarLevels]);

  // FN key handling
  useEffect(() => {
    let lastFnDown = 0;
    const unsubs: Array<() => void> = [];

    listen("fn-key-down", () => {
      const now = Date.now();
      const currentState = pillState;
      const currentLocked = recordingLocked;

      if (currentState === "recording" && currentLocked) {
        stopRecording();
        return;
      }

      if (currentState !== "recording") {
        if (now - lastFnDown < DOUBLE_TAP_MS) {
          setRecordingLocked(true);
          startRecording();
        } else {
          setRecordingLocked(false);
          startRecording();
        }
      }

      lastFnDown = now;
    }).then((unsub) => unsubs.push(unsub));

    listen("fn-key-up", () => {
      if (pillState === "recording" && !recordingLocked) {
        stopRecording();
      }
    }).then((unsub) => unsubs.push(unsub));

    listen("toggle-recording", () => {
      if (pillState === "recording") {
        stopRecording();
      } else {
        startRecording();
      }
    }).then((unsub) => unsubs.push(unsub));

    listen("cancel-recording", () => {
      if (pillState === "recording") cancelRecording();
    }).then((unsub) => unsubs.push(unsub));

    listen("transcription-complete", () => {
      setPillState("done");
      setTimeout(() => setPillState("idle"), 1200);
    }).then((unsub) => unsubs.push(unsub));

    listen("transcription-error", (payload) => {
      console.error("Transcription error:", payload);
      setPillState("idle");
    }).then((unsub) => unsubs.push(unsub));

    listen("pill-hover", (hovered) => {
      const pill = document.getElementById("pill");
      if (pill) {
        pill.classList.toggle("hover", hovered);
      }
    }).then((unsub) => unsubs.push(unsub));

    return () => unsubs.forEach((fn) => fn());
  }, [pillState, recordingLocked, startRecording, stopRecording, cancelRecording, setPillState, setRecordingLocked]);

  // Keyboard escape to cancel
  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape" && isRecording) {
        cancelRecording();
      }
    }
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [isRecording, cancelRecording]);

  // First-run setup check
  useEffect(() => {
    async function checkSetup() {
      try {
        const status = await invoke("check_setup");
        if (!status.whisper_ready || !status.cleanup_ready) {
          setPillState("setup");
          await invoke("setup_download_models");
        } else {
          setPillState("idle");
        }
      } catch {
        setPillState("idle");
      }
    }
    checkSetup();

    const unsubs: Array<() => void> = [];
    listen("setup-complete", () => setPillState("idle")).then((u) => unsubs.push(u));
    listen("setup-error", () => {
      setTimeout(() => setPillState("idle"), 3000);
    }).then((u) => unsubs.push(u));

    return () => unsubs.forEach((fn) => fn());
  }, [setPillState]);

  const handleToggleHistory = useCallback(() => {
    invoke("toggle_history").catch(() => {});
  }, []);

  const handleOpenSettings = useCallback(async () => {
    try {
      await invoke("open_settings");
    } catch (err) {
      console.error("Settings error:", err);
    }
  }, []);

  const handleMouseDown = useCallback(
    async (e: React.MouseEvent) => {
      if ((e.target as HTMLElement).closest("button, canvas")) return;
      try {
        await invoke("start_drag");
      } catch {}
      invoke("persist_pill_position").catch(() => {});
    },
    []
  );

  return (
    <div id="pill" className={pillState !== "idle" ? pillState : undefined} onMouseDown={handleMouseDown}>
      <PillStates
        onToggleHistory={handleToggleHistory}
        onOpenSettings={handleOpenSettings}
        onCancel={cancelRecording}
        onStop={stopRecording}
      />
    </div>
  );
}
```

- [ ] **Step 9: Create pill-entry.tsx and index.html**

```tsx
// ui/src/pages/pill/pill-entry.tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Provider } from "jotai";
import { PillApp } from "./pill-app";
import "@shared/global.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Provider>
      <PillApp />
    </Provider>
  </StrictMode>
);
```

```html
<!-- ui/src/pages/pill/index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Zecho</title>
</head>
<body>
  <div id="app">
    <div id="root"></div>
  </div>
  <script type="module" src="./pill-entry.tsx"></script>
</body>
</html>
```

Note: The `#app` wrapper and `#pill` element need the same structural CSS as the original. Add a `pill-app.module.css` or put the app/pill container styles in `PillStates.module.css` — the pill sizing/transitions depend on the parent `#pill` element having the right base styles. Check the original `styles.css` lines 43-71 for the `#pill` base styles and lines 29-41 for `#app` styles. These should be added either as a page-level CSS file imported in `pill-entry.tsx` or integrated into the component styles.

- [ ] **Step 10: Commit**

```bash
git add ui/src/pages/pill/
git commit -m "feat(ui): add pill page — React entry, state machine, waveform, PillStates component"
```

---

## Task 5: History Page

**Files:**
- Create: `ui/src/pages/history/index.html`
- Create: `ui/src/pages/history/history-entry.tsx`
- Create: `ui/src/pages/history/history-app.tsx`
- Create: `ui/src/pages/history/history-atoms.ts`
- Create: `ui/src/pages/history/components/history-item/HistoryItem.tsx`
- Create: `ui/src/pages/history/components/history-item/HistoryItem.module.css`
- Create: `ui/src/pages/history/components/history-item/HistoryItem.test.tsx`
- Create: `ui/src/pages/history/components/history-detail/HistoryDetail.tsx`
- Create: `ui/src/pages/history/components/history-detail/HistoryDetail.module.css`

- [ ] **Step 1: Create history-atoms.ts**

```ts
// ui/src/pages/history/history-atoms.ts
import { atom } from "jotai";
import type { HistoryEntry } from "@shared/tauri-types";

export const historyItemsAtom = atom<HistoryEntry[]>([]);
export const expandedItemIdAtom = atom<string | null>(null);
```

- [ ] **Step 2: Write the failing test for HistoryItem**

```tsx
// ui/src/pages/history/components/history-item/HistoryItem.test.tsx
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";
import { HistoryItem } from "./HistoryItem";
import type { HistoryEntry } from "@shared/tauri-types";

const mockEntry: HistoryEntry = {
  id: "abc-123",
  text: "Hello world",
  raw_text: "Hello um world",
  created_at: new Date().toISOString(),
  transcribe_ms: 1200,
  cleanup_ms: 800,
};

describe("HistoryItem", () => {
  it("renders the cleaned text", () => {
    render(
      React.createElement(HistoryItem, {
        entry: mockEntry,
        isExpanded: false,
        onCopy: vi.fn(),
        onToggleDetail: vi.fn(),
      })
    );
    expect(screen.getByText("Hello world")).toBeDefined();
  });

  it("calls onCopy when clicked", () => {
    const onCopy = vi.fn();
    render(
      React.createElement(HistoryItem, {
        entry: mockEntry,
        isExpanded: false,
        onCopy,
        onToggleDetail: vi.fn(),
      })
    );
    fireEvent.click(screen.getByText("Hello world"));
    expect(onCopy).toHaveBeenCalledWith("abc-123");
  });

  it("shows timing info when available", () => {
    render(
      React.createElement(HistoryItem, {
        entry: mockEntry,
        isExpanded: false,
        onCopy: vi.fn(),
        onToggleDetail: vi.fn(),
      })
    );
    expect(screen.getByText(/STT 1\.2s/)).toBeDefined();
  });
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd ui && pnpm test -- src/pages/history/components/history-item/HistoryItem.test.tsx`
Expected: FAIL — HistoryItem not found

- [ ] **Step 4: Create HistoryItem.tsx**

```tsx
// ui/src/pages/history/components/history-item/HistoryItem.tsx
import type { HistoryEntry } from "@shared/tauri-types";
import { HistoryDetail } from "../history-detail/HistoryDetail";
import styles from "./HistoryItem.module.css";

interface HistoryItemProps {
  entry: HistoryEntry;
  isExpanded: boolean;
  onCopy: (id: string) => void;
  onToggleDetail: (id: string) => void;
}

function formatTime(isoString: string): string {
  const d = new Date(isoString);
  const now = new Date();
  const diff = now.getTime() - d.getTime();
  if (diff < 60000) return "Just now";
  if (diff < 3600000) return Math.floor(diff / 60000) + "m ago";
  if (diff < 86400000) return Math.floor(diff / 3600000) + "h ago";
  return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}

export function HistoryItem({
  entry,
  isExpanded,
  onCopy,
  onToggleDetail,
}: HistoryItemProps) {
  const hasTimings = entry.transcribe_ms > 0 || entry.cleanup_ms > 0;

  return (
    <>
      <div className={styles.item}>
        <div className={styles.content} onClick={() => onCopy(entry.id)}>
          <div className={styles.text}>{entry.text}</div>
          <div className={styles.meta}>
            <span className={styles.time}>{formatTime(entry.created_at)}</span>
            {hasTimings && (
              <span className={styles.timing}>
                STT {(entry.transcribe_ms / 1000).toFixed(1)}s + AI{" "}
                {(entry.cleanup_ms / 1000).toFixed(1)}s
              </span>
            )}
          </div>
        </div>
        <button
          className={styles.infoBtn}
          onClick={(e) => {
            e.stopPropagation();
            onToggleDetail(entry.id);
          }}
          aria-label="Details"
        >
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
            <circle cx="8" cy="8" r="6.5" stroke="currentColor" strokeWidth="1.2" />
            <path
              d="M8 7v4M8 5.5v.5"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
            />
          </svg>
        </button>
      </div>
      {isExpanded && <HistoryDetail entry={entry} />}
    </>
  );
}
```

- [ ] **Step 5: Create HistoryItem.module.css**

Port from the existing `history.html` inline styles, replacing hardcoded colors with token variables.

```css
/* ui/src/pages/history/components/history-item/HistoryItem.module.css */

.item {
  padding: 10px 14px;
  border-bottom: 1px solid var(--color-border);
  cursor: pointer;
  transition: background 0.1s ease;
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.item:hover {
  background: var(--color-hover);
}

.item:last-child {
  border-bottom: none;
}

.content {
  flex: 1;
  min-width: 0;
}

.text {
  font-size: 12px;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  color: var(--color-text);
  word-break: break-word;
}

.meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.time {
  font-size: 10px;
  color: var(--color-text-dim);
}

.timing {
  font-size: 9px;
  color: var(--color-accent-light);
  font-variant-numeric: tabular-nums;
}

.infoBtn {
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.1s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: none;
  background: transparent;
  color: var(--color-text-dim);
  border-radius: 50%;
  cursor: pointer;
  padding: 0;
}

.item:hover .infoBtn {
  opacity: 1;
}

.infoBtn:hover {
  background: var(--color-hover);
  color: var(--color-text);
}
```

- [ ] **Step 6: Create HistoryDetail.tsx and HistoryDetail.module.css**

The standalone `HistoryDetail` component (for use outside `HistoryItem` if needed):

```tsx
// ui/src/pages/history/components/history-detail/HistoryDetail.tsx
import type { HistoryEntry } from "@shared/tauri-types";
import styles from "./HistoryDetail.module.css";

interface HistoryDetailProps {
  entry: HistoryEntry;
}

export function HistoryDetail({ entry }: HistoryDetailProps) {
  const hasTimings = entry.transcribe_ms > 0 || entry.cleanup_ms > 0;

  return (
    <div className={styles.detail}>
      <div className={styles.col}>
        <div className={styles.label}>Original</div>
        <div className={styles.text}>{entry.raw_text}</div>
      </div>
      <div className={styles.col}>
        <div className={styles.label}>Cleaned</div>
        <div className={styles.text}>{entry.text}</div>
      </div>
      {hasTimings && (
        <div className={styles.timing}>
          STT {(entry.transcribe_ms / 1000).toFixed(1)}s · AI{" "}
          {(entry.cleanup_ms / 1000).toFixed(1)}s · Total{" "}
          {((entry.transcribe_ms + entry.cleanup_ms) / 1000).toFixed(1)}s
        </div>
      )}
    </div>
  );
}
```

```css
/* ui/src/pages/history/components/history-detail/HistoryDetail.module.css */

.detail {
  padding: 10px 14px;
  border-bottom: 1px solid var(--color-border);
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  background: var(--color-accent-light);
}

.col {
  min-width: 0;
}

.label {
  font-size: 9px;
  font-weight: 600;
  color: var(--color-text-dim);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin-bottom: 4px;
}

.text {
  font-size: 11px;
  line-height: 1.5;
  color: var(--color-text);
  word-break: break-word;
}

.timing {
  grid-column: 1 / -1;
  font-size: 9px;
  color: var(--color-accent-light);
  font-variant-numeric: tabular-nums;
}
```

- [ ] **Step 7: Create history-app.tsx**

```tsx
// ui/src/pages/history/history-app.tsx
import { useEffect, useState, useCallback } from "react";
import { useAtom } from "jotai";
import { historyItemsAtom, expandedItemIdAtom } from "./history-atoms";
import { useTheme } from "@shared/theme-hook";
import { invoke, listen } from "@shared/tauri-hook";
import { HistoryItem } from "./components/history-item/HistoryItem";
import type { HistoryEntry } from "@shared/tauri-types";

export function HistoryApp() {
  useTheme();
  const [items, setItems] = useAtom(historyItemsAtom);
  const [expandedId, setExpandedId] = useAtom(expandedItemIdAtom);

  const loadHistory = useCallback(async () => {
    try {
      const entries = await invoke("get_history");
      setItems(entries);
    } catch {}
  }, [setItems]);

  useEffect(() => {
    loadHistory();
    const unsubs: Array<() => void> = [];
    listen("transcription-complete", () => loadHistory()).then((u) =>
      unsubs.push(u)
    );
    return () => unsubs.forEach((fn) => fn());
  }, [loadHistory]);

  const handleCopy = useCallback(
    async (id: string) => {
      try {
        await invoke("copy_history_item", { id });
        setTimeout(() => loadHistory(), 800);
      } catch {}
    },
    [loadHistory]
  );

  const handleToggleDetail = useCallback(
    (id: string) => {
      setExpandedId((prev) => (prev === id ? null : id));
    },
    [setExpandedId]
  );

  const handleClear = useCallback(async () => {
    try {
      await invoke("clear_history");
      loadHistory();
    } catch {}
  }, [loadHistory]);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "10px 14px",
          borderBottom: "1px solid var(--color-border)",
          fontSize: "11px",
          fontWeight: 600,
          color: "var(--color-text-dim)",
          textTransform: "uppercase",
          letterSpacing: "0.06em",
          flexShrink: 0,
        }}
      >
        <span>History</span>
        <button
          onClick={handleClear}
          style={{
            fontSize: "11px",
            fontWeight: 500,
            fontFamily: "inherit",
            color: "var(--color-text-dim)",
            background: "none",
            border: "none",
            cursor: "pointer",
          }}
        >
          Clear
        </button>
      </div>
      <div style={{ overflowY: "auto", flex: 1 }}>
        {items.length === 0 ? (
          <div
            style={{
              padding: "24px 14px",
              textAlign: "center",
              fontSize: "12px",
              color: "var(--color-text-dim)",
            }}
          >
            No recordings yet
          </div>
        ) : (
          items.map((entry) => (
            <HistoryItem
              key={entry.id}
              entry={entry}
              isExpanded={expandedId === entry.id}
              onCopy={handleCopy}
              onToggleDetail={handleToggleDetail}
            />
          ))
        )}
      </div>
    </div>
  );
}
```

Note: The header/container styles above are inline for now since they're page-level layout, not component styles. If desired, extract to a `history-app.module.css` during implementation.

- [ ] **Step 8: Create history-entry.tsx and index.html**

```tsx
// ui/src/pages/history/history-entry.tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Provider } from "jotai";
import { HistoryApp } from "./history-app";
import "@shared/global.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Provider>
      <HistoryApp />
    </Provider>
  </StrictMode>
);
```

```html
<!-- ui/src/pages/history/index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Zecho History</title>
</head>
<body>
  <div id="root"></div>
  <script type="module" src="./history-entry.tsx"></script>
</body>
</html>
```

- [ ] **Step 9: Run HistoryItem test to verify it passes**

Run: `cd ui && pnpm test -- src/pages/history/components/history-item/HistoryItem.test.tsx`
Expected: PASS (all 3 tests)

- [ ] **Step 10: Commit**

```bash
git add ui/src/pages/history/
git commit -m "feat(ui): add history page — React entry, HistoryItem, HistoryDetail"
```

---

## Task 6: Settings Page

**Files:**
- Create: `ui/src/pages/settings/index.html`
- Create: `ui/src/pages/settings/settings-entry.tsx`
- Create: `ui/src/pages/settings/settings-app.tsx`
- Create: `ui/src/pages/settings/settings-atoms.ts`
- Create: 6 component folders with `.tsx` and `.module.css` files
- Create: `ui/src/pages/settings/components/toggle/Toggle.test.tsx`

- [ ] **Step 1: Create settings-atoms.ts**

```ts
// ui/src/pages/settings/settings-atoms.ts
import { atom } from "jotai";
import type { Settings, Model } from "@shared/tauri-types";

export const settingsAtom = atom<Settings>({
  writing_style: "Casual",
  cleanup_level: "Light",
  auto_paste: true,
  custom_prompt: null,
  active_whisper_model: "whisper-base-en",
  active_cleanup_model: "qwen25-1.5b",
});

export const modelsAtom = atom<Model[]>([]);
export const activeTabAtom = atom<"style" | "models" | "general">("style");
```

- [ ] **Step 2: Write the failing test for Toggle**

```tsx
// ui/src/pages/settings/components/toggle/Toggle.test.tsx
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";
import { Toggle } from "./Toggle";

describe("Toggle", () => {
  it("renders checked state", () => {
    render(
      React.createElement(Toggle, { checked: true, onChange: vi.fn() })
    );
    const input = screen.getByRole("checkbox") as HTMLInputElement;
    expect(input.checked).toBe(true);
  });

  it("renders unchecked state", () => {
    render(
      React.createElement(Toggle, { checked: false, onChange: vi.fn() })
    );
    const input = screen.getByRole("checkbox") as HTMLInputElement;
    expect(input.checked).toBe(false);
  });

  it("calls onChange when toggled", () => {
    const onChange = vi.fn();
    render(
      React.createElement(Toggle, { checked: false, onChange })
    );
    fireEvent.click(screen.getByRole("checkbox"));
    expect(onChange).toHaveBeenCalledWith(true);
  });
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd ui && pnpm test -- src/pages/settings/components/toggle/Toggle.test.tsx`
Expected: FAIL — Toggle not found

- [ ] **Step 4: Create Toggle.tsx and Toggle.module.css**

```tsx
// ui/src/pages/settings/components/toggle/Toggle.tsx
import styles from "./Toggle.module.css";

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export function Toggle({ checked, onChange }: ToggleProps) {
  return (
    <label className={styles.toggle}>
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
      />
      <span className={styles.track} />
      <span className={styles.knob} />
    </label>
  );
}
```

```css
/* ui/src/pages/settings/components/toggle/Toggle.module.css */

.toggle {
  position: relative;
  width: 46px;
  height: 28px;
  cursor: pointer;
  display: inline-block;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
  position: absolute;
}

.track {
  position: absolute;
  inset: 0;
  background: #48484a;
  border-radius: 14px;
  transition: background 0.2s ease;
}

.toggle input:checked + .track {
  background: var(--color-accent);
}

.knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 24px;
  height: 24px;
  background: #fff;
  border-radius: 50%;
  transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

.toggle input:checked ~ .knob {
  transform: translateX(18px);
}
```

- [ ] **Step 5: Run Toggle test to verify it passes**

Run: `cd ui && pnpm test -- src/pages/settings/components/toggle/Toggle.test.tsx`
Expected: PASS (all 3 tests)

- [ ] **Step 6: Create CustomPrompt.tsx and CustomPrompt.module.css**

```tsx
// ui/src/pages/settings/components/custom-prompt/CustomPrompt.tsx
import { useRef, useEffect, useCallback } from "react";
import styles from "./CustomPrompt.module.css";

interface CustomPromptProps {
  value: string;
  onChange: (value: string | null) => void;
}

export function CustomPrompt({ value, onChange }: CustomPromptProps) {
  const timeoutRef = useRef<ReturnType<typeof setTimeout>>();

  const handleInput = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      clearTimeout(timeoutRef.current);
      const newValue = e.target.value;
      timeoutRef.current = setTimeout(() => {
        onChange(newValue || null);
      }, 500);
    },
    [onChange]
  );

  useEffect(() => {
    return () => clearTimeout(timeoutRef.current);
  }, []);

  return (
    <div className={styles.wrapper}>
      <textarea
        className={styles.textarea}
        defaultValue={value}
        onChange={handleInput}
        placeholder='e.g. "Always use Oxford commas" or "Keep technical jargon as-is"'
      />
    </div>
  );
}
```

```css
/* ui/src/pages/settings/components/custom-prompt/CustomPrompt.module.css */

.wrapper {
  margin-top: 8px;
}

.textarea {
  width: 100%;
  min-height: 72px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  color: var(--color-text);
  font-family: inherit;
  font-size: 13px;
  padding: 10px 12px;
  resize: vertical;
  outline: none;
  transition: border-color 0.15s ease;
}

.textarea:focus {
  border-color: var(--color-accent);
}

.textarea::placeholder {
  color: var(--color-text-dim);
}
```

- [ ] **Step 7: Create StyleCards.tsx and StyleCards.module.css**

```tsx
// ui/src/pages/settings/components/style-cards/StyleCards.tsx
import styles from "./StyleCards.module.css";

const STYLES = [
  {
    key: "Formal" as const,
    title: "Formal.",
    desc: "Caps + Punctuation",
    preview:
      "Hey, are you free for lunch tomorrow? Let's do 12 if that works for you.",
  },
  {
    key: "Casual" as const,
    title: "Casual",
    desc: "Caps + Less punctuation",
    preview:
      "Hey are you free for lunch tomorrow? Let's do 12 if that works for you",
  },
  {
    key: "VeryCasual" as const,
    title: "very casual",
    desc: "No caps + Less punctuation",
    preview:
      "hey are you free for lunch tomorrow? let's do 12 if that works for you",
  },
];

interface StyleCardsProps {
  selected: "Formal" | "Casual" | "VeryCasual";
  onSelect: (style: "Formal" | "Casual" | "VeryCasual") => void;
}

export function StyleCards({ selected, onSelect }: StyleCardsProps) {
  return (
    <div className={styles.grid}>
      {STYLES.map((s) => (
        <div
          key={s.key}
          className={`${styles.card} ${selected === s.key ? styles.selected : ""}`}
          onClick={() => onSelect(s.key)}
        >
          <div className={styles.cardTitle}>{s.title}</div>
          <div className={styles.cardDesc}>{s.desc}</div>
          <div className={styles.cardPreview}>{s.preview}</div>
        </div>
      ))}
    </div>
  );
}
```

```css
/* ui/src/pages/settings/components/style-cards/StyleCards.module.css */

.grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}

.card {
  background: var(--color-surface);
  border: 2px solid transparent;
  border-radius: var(--radius-md);
  padding: 16px;
  cursor: pointer;
  transition: all 0.18s ease;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.card:hover {
  background: var(--color-surface-hover);
}

.card.selected {
  border-color: var(--color-accent);
  background: var(--color-accent-light);
}

.cardTitle {
  font-size: 16px;
  font-weight: 650;
  letter-spacing: -0.01em;
}

.cardDesc {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.cardPreview {
  margin-top: auto;
  padding: 10px 12px;
  background: var(--color-accent-light);
  border-radius: var(--radius-sm);
  font-size: 12px;
  line-height: 1.5;
  color: var(--color-accent);
}
```

- [ ] **Step 8: Create CleanupCards.tsx and CleanupCards.module.css**

```tsx
// ui/src/pages/settings/components/cleanup-cards/CleanupCards.tsx
import styles from "./CleanupCards.module.css";

const LEVELS = [
  {
    key: "None" as const,
    title: "None",
    desc: "Transcribes exactly what you said",
    preview:
      "hey we still on for coffee or? I think we maybe should leave earlier um to make it there in time",
  },
  {
    key: "Light" as const,
    title: "Light",
    desc: "Cleans up filler words and grammar",
    preview:
      "Hey, are we still on for coffee? I think we should leave earlier to make it there in time.",
  },
  {
    key: "Medium" as const,
    title: "Medium",
    desc: "Edits for clarity and conciseness",
    preview:
      "Hey, are we still on for coffee? We should leave earlier; there might be traffic.",
  },
  {
    key: "High" as const,
    title: "High",
    desc: "Rewrites for brevity and polish",
    preview:
      "Hey, still on for coffee? Let's leave early to beat traffic.",
  },
];

interface CleanupCardsProps {
  selected: "None" | "Light" | "Medium" | "High";
  onSelect: (level: "None" | "Light" | "Medium" | "High") => void;
}

export function CleanupCards({ selected, onSelect }: CleanupCardsProps) {
  return (
    <div className={styles.grid}>
      {LEVELS.map((l) => (
        <div
          key={l.key}
          className={`${styles.card} ${selected === l.key ? styles.selected : ""}`}
          onClick={() => onSelect(l.key)}
        >
          <div className={styles.cardTitle}>{l.title}</div>
          <div className={styles.cardDesc}>{l.desc}</div>
          <div className={styles.cardPreview}>{l.preview}</div>
        </div>
      ))}
    </div>
  );
}
```

```css
/* ui/src/pages/settings/components/cleanup-cards/CleanupCards.module.css */

.grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

.card {
  background: var(--color-surface);
  border: 2px solid transparent;
  border-radius: var(--radius-md);
  padding: 16px;
  cursor: pointer;
  transition: all 0.18s ease;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.card:hover {
  background: var(--color-surface-hover);
}

.card.selected {
  border-color: var(--color-accent);
  background: var(--color-accent-light);
}

.cardTitle {
  font-size: 16px;
  font-weight: 650;
  letter-spacing: -0.01em;
}

.cardDesc {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.cardPreview {
  margin-top: auto;
  padding: 10px 12px;
  background: var(--color-accent-light);
  border-radius: var(--radius-sm);
  font-size: 12px;
  line-height: 1.5;
  color: var(--color-accent);
}
```

- [ ] **Step 9: Create ModelCard.tsx and ModelCard.module.css**

```tsx
// ui/src/pages/settings/components/model-card/ModelCard.tsx
import type { Model } from "@shared/tauri-types";
import styles from "./ModelCard.module.css";

interface ModelCardProps {
  model: Model;
  isActive: boolean;
  onDownload: (modelId: string) => void;
  onSelect: (modelId: string, modelType: string) => void;
}

function ScoreDots({
  score,
  max,
  colorClass,
}: {
  score: number;
  max: number;
  colorClass?: string;
}) {
  return (
    <div className={styles.statBar}>
      {Array.from({ length: max }, (_, i) => (
        <div
          key={i}
          className={`${styles.statDot} ${i < score ? styles.filled : ""} ${
            i < score && colorClass ? styles[colorClass] : ""
          }`}
        />
      ))}
    </div>
  );
}

export function ModelCard({
  model,
  isActive,
  onDownload,
  onSelect,
}: ModelCardProps) {
  const recommended = model.description.includes("Recommended");
  const sizeLabel =
    model.size_mb >= 1000
      ? (model.size_mb / 1024).toFixed(1) + " GB"
      : model.size_mb + " MB";

  return (
    <div className={`${styles.card} ${isActive ? styles.active : ""}`}>
      <div className={styles.info}>
        <div className={styles.name}>
          {model.name}
          {recommended && <span className={styles.badge}>Rec</span>}
        </div>
        <div className={styles.desc}>{model.description}</div>
        <div className={styles.meta}>
          <div className={styles.stat}>
            Quality <ScoreDots score={model.quality_score} max={10} />
          </div>
          <div className={styles.stat}>
            Speed{" "}
            <ScoreDots score={model.speed_score} max={10} colorClass="green" />
          </div>
        </div>
        <div className={styles.size}>{sizeLabel}</div>
      </div>
      <div className={styles.action}>
        {!model.downloaded ? (
          <button
            className={`${styles.btn} ${styles.download}`}
            onClick={() => onDownload(model.id)}
          >
            Download
          </button>
        ) : isActive ? (
          <button className={`${styles.btn} ${styles.activeBtn}`}>
            Active
          </button>
        ) : (
          <button
            className={`${styles.btn} ${styles.selectBtn}`}
            onClick={() => onSelect(model.id, model.model_type)}
          >
            Use
          </button>
        )}
      </div>
    </div>
  );
}
```

```css
/* ui/src/pages/settings/components/model-card/ModelCard.module.css */

.card {
  background: var(--color-surface);
  border: 2px solid transparent;
  border-radius: var(--radius-md);
  padding: 14px 16px;
  display: flex;
  align-items: center;
  gap: 14px;
  cursor: pointer;
  transition: all 0.18s ease;
}

.card:hover { background: var(--color-surface-hover); }
.card.active { border-color: var(--color-accent); background: var(--color-accent-light); }

.info { flex: 1; min-width: 0; }

.name {
  font-size: 14px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.badge {
  font-size: 10px;
  font-weight: 500;
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--color-accent-light);
  color: var(--color-accent);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.desc { font-size: 12px; color: var(--color-text-secondary); margin-top: 2px; }
.meta { display: flex; gap: 12px; margin-top: 6px; }

.stat {
  font-size: 11px;
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
  gap: 4px;
}

.statBar { display: flex; gap: 2px; }

.statDot {
  width: 6px;
  height: 6px;
  border-radius: 2px;
  background: #48484a;
}

.statDot.filled { background: var(--color-accent); }
.statDot.green { background: var(--color-green); }

.size { font-size: 11px; color: var(--color-text-dim); margin-top: 2px; }

.action { flex-shrink: 0; }

.btn {
  padding: 6px 14px;
  border-radius: var(--radius-sm);
  border: none;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  font-family: inherit;
}

.download { background: var(--color-accent); color: #fff; }
.download:hover { filter: brightness(1.1); }
.activeBtn { background: rgba(48, 209, 88, 0.12); color: var(--color-green); cursor: default; }
.selectBtn { background: var(--color-bg); color: var(--color-text-secondary); border: 1px solid var(--color-border); }
.selectBtn:hover { color: var(--color-text); border-color: var(--color-hover); }
```

- [ ] **Step 10: Create ModelList.tsx and ModelList.module.css**

```tsx
// ui/src/pages/settings/components/model-list/ModelList.tsx
import type { Model } from "@shared/tauri-types";
import { ModelCard } from "../model-card/ModelCard";
import styles from "./ModelList.module.css";

interface ModelListProps {
  models: Model[];
  activeModelId: string;
  onDownload: (modelId: string) => void;
  onSelect: (modelId: string, modelType: string) => void;
}

export function ModelList({
  models,
  activeModelId,
  onDownload,
  onSelect,
}: ModelListProps) {
  return (
    <div className={styles.list}>
      {models.map((model) => (
        <ModelCard
          key={model.id}
          model={model}
          isActive={model.id === activeModelId}
          onDownload={onDownload}
          onSelect={onSelect}
        />
      ))}
    </div>
  );
}
```

```css
/* ui/src/pages/settings/components/model-list/ModelList.module.css */

.list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
```

- [ ] **Step 11: Create settings-app.tsx**

This is the largest component — it manages tab navigation, settings load/save, and model operations. Port the behavior from the existing `settings.html` inline script.

```tsx
// ui/src/pages/settings/settings-app.tsx
import { useEffect, useCallback, useState } from "react";
import { useAtom } from "jotai";
import { settingsAtom, modelsAtom, activeTabAtom } from "./settings-atoms";
import { useTheme } from "@shared/theme-hook";
import { invoke, listen } from "@shared/tauri-hook";
import { StyleCards } from "./components/style-cards/StyleCards";
import { CleanupCards } from "./components/cleanup-cards/CleanupCards";
import { ModelList } from "./components/model-list/ModelList";
import { Toggle } from "./components/toggle/Toggle";
import { CustomPrompt } from "./components/custom-prompt/CustomPrompt";
import type { Settings } from "@shared/tauri-types";

export function SettingsApp() {
  useTheme();
  const [settings, setSettings] = useAtom(settingsAtom);
  const [models, setModels] = useAtom(modelsAtom);
  const [activeTab, setActiveTab] = useAtom(activeTabAtom);
  const [previousCleanupLevel, setPreviousCleanupLevel] = useState<string>("Light");

  const saveSettings = useCallback(
    async (updated: Settings) => {
      setSettings(updated);
      await invoke("update_settings", { newSettings: updated });
    },
    [setSettings]
  );

  const loadModels = useCallback(async () => {
    try {
      const list = await invoke("list_models");
      setModels(list);
    } catch (err) {
      console.error("Failed to load models:", err);
    }
  }, [setModels]);

  useEffect(() => {
    async function load() {
      const s = await invoke("get_settings");
      setSettings(s);
      if (s.cleanup_level !== "None") setPreviousCleanupLevel(s.cleanup_level);
    }
    load();
    loadModels();

    const unsubs: Array<() => void> = [];
    listen("model-download-complete", () => loadModels()).then((u) => unsubs.push(u));
    listen("model-download-error", (payload) => {
      console.error("Download failed:", payload);
      loadModels();
    }).then((u) => unsubs.push(u));

    return () => unsubs.forEach((fn) => fn());
  }, [setSettings, loadModels]);

  const handleDownload = useCallback(async (modelId: string) => {
    try {
      await invoke("download_model", { modelId });
    } catch (err) {
      console.error("Download error:", err);
    }
  }, []);

  const handleSelectModel = useCallback(
    async (modelId: string, modelType: string) => {
      try {
        if (modelType === "Whisper") {
          await invoke("load_whisper_model_cmd", { modelId });
          saveSettings({ ...settings, active_whisper_model: modelId });
        } else {
          await invoke("load_cleanup_model", { modelId });
          saveSettings({ ...settings, active_cleanup_model: modelId });
        }
        loadModels();
      } catch (err) {
        console.error("Model load error:", err);
      }
    },
    [settings, saveSettings, loadModels]
  );

  const cleanupEnabled = settings.cleanup_level !== "None";

  const handleCleanupToggle = useCallback(
    (checked: boolean) => {
      if (checked) {
        saveSettings({ ...settings, cleanup_level: previousCleanupLevel as Settings["cleanup_level"] });
      } else {
        setPreviousCleanupLevel(settings.cleanup_level);
        saveSettings({ ...settings, cleanup_level: "None" });
      }
    },
    [settings, previousCleanupLevel, saveSettings]
  );

  const tabs = ["style", "models", "general"] as const;

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100vh", background: "var(--color-bg)" }}>
      <div style={{ padding: "20px 28px 0", flexShrink: 0 }}>
        <h1 style={{ fontSize: "22px", fontWeight: 700, letterSpacing: "-0.02em" }}>Settings</h1>
      </div>

      <div style={{ display: "flex", gap: 0, padding: "16px 28px 0", borderBottom: "1px solid var(--color-border)", flexShrink: 0 }}>
        {tabs.map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            style={{
              padding: "8px 18px 12px",
              fontSize: "13px",
              fontWeight: activeTab === tab ? 600 : 500,
              color: activeTab === tab ? "var(--color-text)" : "var(--color-text-secondary)",
              cursor: "pointer",
              borderBottom: activeTab === tab ? "2px solid var(--color-accent)" : "2px solid transparent",
              background: "none",
              border: "none",
              borderBottomStyle: "solid",
              borderBottomWidth: "2px",
              borderBottomColor: activeTab === tab ? "var(--color-accent)" : "transparent",
              fontFamily: "inherit",
              transition: "all 0.15s ease",
            }}
          >
            {tab.charAt(0).toUpperCase() + tab.slice(1)}
          </button>
        ))}
      </div>

      <div style={{ flex: 1, overflowY: "auto", padding: "24px 28px" }}>
        {activeTab === "style" && (
          <>
            <Section label="Writing Style">
              <StyleCards
                selected={settings.writing_style}
                onSelect={(style) => saveSettings({ ...settings, writing_style: style })}
              />
            </Section>
            <Section label="Auto Cleanup">
              <CleanupCards
                selected={settings.cleanup_level}
                onSelect={(level) => saveSettings({ ...settings, cleanup_level: level })}
              />
              <div style={{ fontSize: "12px", color: "var(--color-text-secondary)", marginTop: "8px", lineHeight: 1.5 }}>
                Your original dictation is always saved in History.
              </div>
            </Section>
            <Section label="Custom Instructions">
              <CustomPrompt
                value={settings.custom_prompt || ""}
                onChange={(v) => saveSettings({ ...settings, custom_prompt: v })}
              />
              <div style={{ fontSize: "12px", color: "var(--color-text-secondary)", marginTop: "8px", lineHeight: 1.5 }}>
                Appended to the cleanup prompt. Leave blank for defaults.
              </div>
            </Section>
          </>
        )}

        {activeTab === "models" && (
          <>
            <Section label="Speech-to-Text">
              <ModelList
                models={models.filter((m) => m.model_type === "Whisper")}
                activeModelId={settings.active_whisper_model}
                onDownload={handleDownload}
                onSelect={handleSelectModel}
              />
            </Section>
            <Section label="Text Cleanup">
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "14px 16px", background: "var(--color-surface)", borderRadius: "var(--radius-md)", marginBottom: "10px" }}>
                <span style={{ fontSize: "14px" }}>Enable AI cleanup</span>
                <Toggle checked={cleanupEnabled} onChange={handleCleanupToggle} />
              </div>
              <div style={{ opacity: cleanupEnabled ? 1 : 0.4, pointerEvents: cleanupEnabled ? "auto" : "none" }}>
                <ModelList
                  models={models.filter((m) => m.model_type === "Cleanup")}
                  activeModelId={settings.active_cleanup_model}
                  onDownload={handleDownload}
                  onSelect={handleSelectModel}
                />
              </div>
              <div style={{ fontSize: "12px", color: "var(--color-text-secondary)", marginTop: "8px", lineHeight: 1.5 }}>
                Larger models produce better results but use more memory.
              </div>
            </Section>
          </>
        )}

        {activeTab === "general" && (
          <>
            <Section label="Behavior">
              <div style={{ background: "var(--color-surface)", borderRadius: "var(--radius-md)", overflow: "hidden" }}>
                <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "14px 16px", minHeight: "48px" }}>
                  <span style={{ fontSize: "14px" }}>Auto-paste after recording</span>
                  <Toggle
                    checked={settings.auto_paste}
                    onChange={(v) => saveSettings({ ...settings, auto_paste: v })}
                  />
                </div>
              </div>
            </Section>
            <Section label="Hotkey">
              <div style={{ background: "var(--color-surface)", borderRadius: "var(--radius-md)", overflow: "hidden" }}>
                <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "14px 16px", minHeight: "48px" }}>
                  <span style={{ fontSize: "14px" }}>Record key</span>
                  <span><kbd style={{ padding: "3px 7px", fontSize: "12px", background: "var(--color-bg)", border: "1px solid var(--color-border)", borderRadius: "6px", color: "var(--color-text-secondary)" }}>Fn</kbd></span>
                </div>
                <div style={{ borderTop: "1px solid var(--color-border)", display: "flex", alignItems: "center", justifyContent: "space-between", padding: "14px 16px", minHeight: "48px" }}>
                  <span style={{ fontSize: "14px" }}>Fallback shortcut</span>
                  <span style={{ display: "flex", gap: "4px" }}>
                    <kbd style={{ padding: "3px 7px", fontSize: "12px", background: "var(--color-bg)", border: "1px solid var(--color-border)", borderRadius: "6px", color: "var(--color-text-secondary)" }}>Option</kbd>
                    <kbd style={{ padding: "3px 7px", fontSize: "12px", background: "var(--color-bg)", border: "1px solid var(--color-border)", borderRadius: "6px", color: "var(--color-text-secondary)" }}>Space</kbd>
                  </span>
                </div>
              </div>
              <div style={{ fontSize: "12px", color: "var(--color-text-secondary)", marginTop: "8px", lineHeight: 1.5 }}>
                Hold Fn to record, release to stop. Double-tap Fn to lock recording. Press Escape to cancel.
              </div>
            </Section>
            <Section label="About">
              <div style={{ background: "var(--color-surface)", borderRadius: "var(--radius-md)", overflow: "hidden" }}>
                <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "14px 16px" }}>
                  <span style={{ fontSize: "14px" }}>Version</span>
                  <span style={{ fontSize: "13px", color: "var(--color-text-secondary)" }}>0.2.0</span>
                </div>
              </div>
            </Section>
          </>
        )}
      </div>

      <div style={{ padding: "12px 28px", textAlign: "center", fontSize: "11px", color: "var(--color-text-dim)", flexShrink: 0, borderTop: "1px solid var(--color-border)" }}>
        Zecho
      </div>
    </div>
  );
}

function Section({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div style={{ marginBottom: "28px" }}>
      <div style={{ fontSize: "11px", fontWeight: 600, color: "var(--color-text-secondary)", textTransform: "uppercase", letterSpacing: "0.06em", marginBottom: "10px" }}>
        {label}
      </div>
      {children}
    </div>
  );
}
```

- [ ] **Step 12: Create settings-entry.tsx and index.html**

```tsx
// ui/src/pages/settings/settings-entry.tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Provider } from "jotai";
import { SettingsApp } from "./settings-app";
import "@shared/global.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Provider>
      <SettingsApp />
    </Provider>
  </StrictMode>
);
```

```html
<!-- ui/src/pages/settings/index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Zecho Settings</title>
</head>
<body>
  <div id="root"></div>
  <script type="module" src="./settings-entry.tsx"></script>
</body>
</html>
```

- [ ] **Step 13: Run all tests**

Run: `cd ui && pnpm test`
Expected: All tests pass (token gen, theme hook, PillStates, HistoryItem, Toggle)

- [ ] **Step 14: Commit**

```bash
git add ui/src/pages/settings/
git commit -m "feat(ui): add settings page — tabs, style/cleanup cards, model list, toggle"
```

---

## Task 7: Setup Page

**Files:**
- Create: `ui/src/pages/setup/index.html`
- Create: `ui/src/pages/setup/setup-entry.tsx`
- Create: `ui/src/pages/setup/setup-app.tsx`
- Create: `ui/src/pages/setup/setup-atoms.ts`
- Create: `ui/src/pages/setup/components/setup-step/SetupStep.tsx`
- Create: `ui/src/pages/setup/components/setup-step/SetupStep.module.css`
- Create: `ui/src/pages/setup/components/setup-step/SetupStep.test.tsx`
- Create: `ui/src/pages/setup/components/action-button/ActionButton.tsx`
- Create: `ui/src/pages/setup/components/action-button/ActionButton.module.css`

- [ ] **Step 1: Create setup-atoms.ts**

```ts
// ui/src/pages/setup/setup-atoms.ts
import { atom } from "jotai";

export interface SetupState {
  accessibility: boolean;
  microphone: boolean;
  whisper: boolean;
  cleanup: boolean;
  downloading: boolean;
}

export const setupStateAtom = atom<SetupState>({
  accessibility: false,
  microphone: false,
  whisper: false,
  cleanup: false,
  downloading: false,
});
```

- [ ] **Step 2: Write the failing test for SetupStep**

```tsx
// ui/src/pages/setup/components/setup-step/SetupStep.test.tsx
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import React from "react";
import { SetupStep } from "./SetupStep";

describe("SetupStep", () => {
  it("renders with pending status", () => {
    render(
      React.createElement(SetupStep, {
        title: "Input Monitoring",
        description: "Required for FN key recording.",
        status: "pending",
        statusText: "",
        icon: React.createElement("span", null, "icon"),
      })
    );
    expect(screen.getByText("Input Monitoring")).toBeDefined();
    expect(screen.getByText("Required for FN key recording.")).toBeDefined();
  });

  it("renders with done status and shows status text", () => {
    render(
      React.createElement(SetupStep, {
        title: "Microphone Access",
        description: "Required to capture your voice.",
        status: "done",
        statusText: "Enabled",
        icon: React.createElement("span", null, "icon"),
      })
    );
    expect(screen.getByText("Enabled")).toBeDefined();
  });

  it("renders spinner when active", () => {
    const { container } = render(
      React.createElement(SetupStep, {
        title: "Speech-to-Text Model",
        description: "Downloading...",
        status: "active",
        statusText: "Downloading...",
        icon: React.createElement("span", null, "icon"),
      })
    );
    expect(container.querySelector("[class*='spinner']")).not.toBeNull();
  });
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd ui && pnpm test -- src/pages/setup/components/setup-step/SetupStep.test.tsx`
Expected: FAIL — SetupStep not found

- [ ] **Step 4: Create SetupStep.tsx and SetupStep.module.css**

```tsx
// ui/src/pages/setup/components/setup-step/SetupStep.tsx
import type { ReactNode } from "react";
import styles from "./SetupStep.module.css";

interface SetupStepProps {
  title: string;
  description: string;
  status: "pending" | "active" | "done";
  statusText: string;
  icon: ReactNode;
}

export function SetupStep({
  title,
  description,
  status,
  statusText,
  icon,
}: SetupStepProps) {
  return (
    <div className={`${styles.step} ${styles[status]}`}>
      <div className={`${styles.icon} ${styles[`icon_${status}`]}`}>
        {status === "done" ? (
          <svg width="16" height="16" viewBox="0 0 18 18" fill="none">
            <path
              d="M4 9L7.5 12.5L14 5.5"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        ) : status === "active" ? (
          <div className={styles.spinner} />
        ) : (
          icon
        )}
      </div>
      <div className={styles.content}>
        <div className={styles.title}>{title}</div>
        <div className={styles.desc}>{description}</div>
        {statusText && <div className={styles.statusText}>{statusText}</div>}
      </div>
    </div>
  );
}
```

```css
/* ui/src/pages/setup/components/setup-step/SetupStep.module.css */

.step {
  background: var(--color-surface);
  border-radius: var(--radius-md);
  padding: 16px 18px;
  display: flex;
  align-items: center;
  gap: 14px;
  transition: all 0.2s ease;
}

.step.active {
  border: 1px solid var(--color-accent);
  background: var(--color-accent-light);
}

.step.done {
  opacity: 0.6;
}

.icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-size: 16px;
}

.icon_pending {
  background: var(--color-hover);
  color: var(--color-text-dim);
}

.icon_active {
  background: var(--color-accent-light);
  color: var(--color-accent);
}

.icon_done {
  background: rgba(48, 209, 88, 0.12);
  color: var(--color-green);
}

.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid var(--color-accent-light);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

.content {
  flex: 1;
  min-width: 0;
}

.title {
  font-size: 14px;
  font-weight: 600;
}

.desc {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 2px;
}

.statusText {
  font-size: 11px;
  color: var(--color-accent);
  margin-top: 4px;
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cd ui && pnpm test -- src/pages/setup/components/setup-step/SetupStep.test.tsx`
Expected: PASS (all 3 tests)

- [ ] **Step 6: Create ActionButton.tsx and ActionButton.module.css**

```tsx
// ui/src/pages/setup/components/action-button/ActionButton.tsx
import styles from "./ActionButton.module.css";

interface ActionButtonProps {
  label: string;
  disabled?: boolean;
  variant?: "primary" | "secondary";
  onClick: () => void;
}

export function ActionButton({
  label,
  disabled = false,
  variant = "primary",
  onClick,
}: ActionButtonProps) {
  return (
    <button
      className={`${styles.btn} ${styles[variant]}`}
      disabled={disabled}
      onClick={onClick}
    >
      {label}
    </button>
  );
}
```

```css
/* ui/src/pages/setup/components/action-button/ActionButton.module.css */

.btn {
  width: 100%;
  padding: 12px;
  border: none;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: all 0.15s ease;
}

.primary {
  background: var(--color-accent);
  color: #fff;
}

.primary:hover:not(:disabled) {
  filter: brightness(1.1);
  transform: translateY(-1px);
}

.primary:active:not(:disabled) {
  transform: translateY(0);
}

.primary:disabled {
  background: #48484a;
  color: var(--color-text-dim);
  cursor: default;
}

.secondary {
  background: var(--color-surface);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
}

.secondary:hover {
  color: var(--color-text);
  border-color: var(--color-hover);
}
```

- [ ] **Step 7: Create setup-app.tsx**

Port the setup wizard flow from the existing `setup.html`. The component manages the sequential permission/download flow.

```tsx
// ui/src/pages/setup/setup-app.tsx
import { useEffect, useCallback } from "react";
import { useAtom } from "jotai";
import { setupStateAtom } from "./setup-atoms";
import { useTheme } from "@shared/theme-hook";
import { invoke, listen } from "@shared/tauri-hook";
import { SetupStep } from "./components/setup-step/SetupStep";
import { ActionButton } from "./components/action-button/ActionButton";
import React from "react";

export function SetupApp() {
  useTheme();
  const [state, setState] = useAtom(setupStateAtom);

  useEffect(() => {
    async function checkState() {
      try {
        const accessibility = await invoke("check_accessibility");
        setState((s) => ({ ...s, accessibility }));
      } catch {}
      try {
        const setup = await invoke("check_setup");
        setState((s) => ({
          ...s,
          whisper: setup.whisper_ready,
          cleanup: setup.cleanup_ready,
        }));
      } catch {}
    }
    checkState();

    const unsubs: Array<() => void> = [];

    listen("setup-progress", (msg) => {
      if (msg.includes("ready") || msg.includes("Ready")) {
        if (msg.toLowerCase().includes("speech")) {
          setState((s) => ({ ...s, whisper: true }));
        } else if (msg.toLowerCase().includes("cleanup")) {
          setState((s) => ({ ...s, cleanup: true }));
        }
      }
    }).then((u) => unsubs.push(u));

    listen("setup-complete", () => {
      setState((s) => ({ ...s, downloading: false, whisper: true, cleanup: true }));
    }).then((u) => unsubs.push(u));

    listen("setup-error", () => {
      setState((s) => ({ ...s, downloading: false }));
    }).then((u) => unsubs.push(u));

    return () => unsubs.forEach((fn) => fn());
  }, [setState]);

  const getNextAction = useCallback((): { label: string; action: string } => {
    if (state.downloading) return { label: "Downloading...", action: "wait" };
    if (!state.accessibility) return { label: "Enable Input Monitoring", action: "accessibility" };
    if (!state.microphone) return { label: "Enable Microphone", action: "microphone" };
    if (!state.whisper || !state.cleanup) return { label: "Download Models", action: "download" };
    return { label: "Get Started", action: "finish" };
  }, [state]);

  const handleAction = useCallback(async () => {
    const { action } = getNextAction();
    if (action === "accessibility") {
      await invoke("open_accessibility_settings");
      const poll = setInterval(async () => {
        const ok = await invoke("check_accessibility");
        if (ok) {
          clearInterval(poll);
          setState((s) => ({ ...s, accessibility: true }));
          await invoke("start_fn_listener");
        }
      }, 2000);
    } else if (action === "microphone") {
      await invoke("request_microphone");
      setTimeout(() => setState((s) => ({ ...s, microphone: true })), 3000);
    } else if (action === "download") {
      setState((s) => ({ ...s, downloading: true }));
      await invoke("setup_download_models");
    } else if (action === "finish") {
      try { await invoke("complete_setup"); } catch {}
      try {
        const win = (window as any).__TAURI__.window.getCurrentWindow();
        await win.hide();
      } catch {
        try { await invoke("hide_setup"); } catch {}
      }
    }
  }, [getNextAction, setState]);

  const handleSkip = useCallback(async () => {
    try { await invoke("complete_setup"); } catch {}
    try {
      const win = (window as any).__TAURI__.window.getCurrentWindow();
      await win.hide();
    } catch {
      try { await invoke("hide_setup"); } catch {}
    }
  }, []);

  const { label, action } = getNextAction();

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100vh", background: "var(--color-bg)" }}>
      <div style={{ flex: 1, display: "flex", flexDirection: "column", padding: "40px 36px 24px", overflow: "hidden" }}>
        <div style={{ textAlign: "center", marginBottom: "32px" }}>
          <h1 style={{ fontSize: "24px", fontWeight: 700, letterSpacing: "-0.02em", marginBottom: "8px" }}>
            Welcome to Zecho
          </h1>
          <p style={{ fontSize: "14px", color: "var(--color-text-secondary)", lineHeight: 1.5 }}>
            Let's get you set up. This takes about a minute.
          </p>
        </div>

        <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: "12px" }}>
          <SetupStep
            title="Input Monitoring"
            description="Required for FN key recording."
            status={state.accessibility ? "done" : "pending"}
            statusText={state.accessibility ? "Enabled" : ""}
            icon={<AccessibilityIcon />}
          />
          <SetupStep
            title="Microphone Access"
            description="Required to capture your voice."
            status={state.microphone ? "done" : "pending"}
            statusText={state.microphone ? "Enabled" : ""}
            icon={<MicrophoneIcon />}
          />
          <SetupStep
            title="Speech-to-Text Model"
            description="Whisper — converts your voice to text locally. (~141 MB)"
            status={state.whisper ? "done" : state.downloading && !state.whisper ? "active" : "pending"}
            statusText={state.whisper ? "Ready" : state.downloading ? "Downloading..." : ""}
            icon={<WhisperIcon />}
          />
          <SetupStep
            title="Text Cleanup Model"
            description="AI cleanup — removes filler words, fixes corrections. (~1 GB)"
            status={state.cleanup ? "done" : state.downloading && state.whisper && !state.cleanup ? "active" : "pending"}
            statusText={state.cleanup ? "Ready" : state.downloading && state.whisper ? "Downloading..." : ""}
            icon={<CleanupIcon />}
          />
        </div>
      </div>

      <div style={{ flexShrink: 0, padding: "16px 36px 0" }}>
        <ActionButton label={label} disabled={action === "wait"} onClick={handleAction} />
      </div>

      <div style={{ padding: "12px 36px 20px", textAlign: "center" }}>
        <button
          onClick={handleSkip}
          style={{
            fontSize: "12px",
            color: "var(--color-text-dim)",
            cursor: "pointer",
            border: "none",
            background: "none",
            fontFamily: "inherit",
          }}
        >
          Skip setup
        </button>
      </div>
    </div>
  );
}

function AccessibilityIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
      <path d="M12 15v2m0-8a1.5 1.5 0 110 3 1.5 1.5 0 010-3z" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      <rect x="3" y="3" width="18" height="18" rx="5" stroke="currentColor" strokeWidth="1.5" fill="none" />
    </svg>
  );
}

function MicrophoneIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
      <rect x="8" y="2" width="8" height="12" rx="4" stroke="currentColor" strokeWidth="1.5" fill="none" />
      <path d="M5 11c0 3.87 3.13 7 7 7s7-3.13 7-7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" fill="none" />
    </svg>
  );
}

function WhisperIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
      <rect x="7" y="2" width="10" height="14" rx="5" stroke="currentColor" strokeWidth="1.5" fill="none" />
      <path d="M4 12c0 4.42 3.58 8 8 8s8-3.58 8-8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" fill="none" />
    </svg>
  );
}

function CleanupIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
      <path d="M12 3L3 9v6l9 6 9-6V9l-9-6z" stroke="currentColor" strokeWidth="1.5" fill="none" />
      <path d="M12 15V9M9 12h6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
    </svg>
  );
}
```

- [ ] **Step 8: Create setup-entry.tsx and index.html**

```tsx
// ui/src/pages/setup/setup-entry.tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Provider } from "jotai";
import { SetupApp } from "./setup-app";
import "@shared/global.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Provider>
      <SetupApp />
    </Provider>
  </StrictMode>
);
```

```html
<!-- ui/src/pages/setup/index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Welcome to Zecho</title>
</head>
<body>
  <div id="root"></div>
  <script type="module" src="./setup-entry.tsx"></script>
</body>
</html>
```

- [ ] **Step 9: Run SetupStep test and all tests**

Run: `cd ui && pnpm test`
Expected: All tests pass

- [ ] **Step 10: Commit**

```bash
git add ui/src/pages/setup/
git commit -m "feat(ui): add setup page — wizard flow, permission steps, model download"
```

---

## Task 8: Tauri Config and Build Verification

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Update tauri.conf.json**

Change `frontendDist`, add `beforeDevCommand` and `beforeBuildCommand`, update window URLs to point to the new Vite output paths.

```json
{
  "build": {
    "frontendDist": "../ui/dist",
    "beforeDevCommand": "pnpm --dir ../ui dev",
    "beforeBuildCommand": "pnpm --dir ../ui build"
  }
}
```

Update window URLs — the pill window uses the default (`index.html`), which Vite will output as `dist/src/pages/pill/index.html`. The other windows need their URLs updated:

- pill: default (no URL field) — set to `src/pages/pill/index.html`
- setup: `src/pages/setup/index.html`
- history: `src/pages/history/index.html`
- settings: `src/pages/settings/index.html`

Note: Vite's multi-page build preserves the directory structure under `dist/`. Verify the exact output paths by running `cd ui && pnpm build` and inspecting the `dist/` folder. Adjust the URLs in `tauri.conf.json` to match.

- [ ] **Step 2: Verify Vite build succeeds**

Run: `cd ui && pnpm build`
Expected: Build completes, `dist/` contains all 4 HTML entry points

- [ ] **Step 3: Verify the dist structure matches tauri.conf.json URLs**

Run: `find ui/dist -name "*.html"`
Expected: Four HTML files, one per page. Update `tauri.conf.json` URLs if paths differ from Step 1.

- [ ] **Step 4: Run all tests**

Run: `cd ui && pnpm test`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: update tauri config for Vite build pipeline"
```

---

## Task 9: CONTRIBUTING.md and README Update

**Files:**
- Create: `CONTRIBUTING.md`
- Modify: `README.md`

- [ ] **Step 1: Create CONTRIBUTING.md**

```markdown
# Contributing to Zecho

## Prerequisites

- macOS 12+
- [Rust toolchain](https://rustup.rs/) (`rustup`)
- [Tauri CLI v2](https://v2.tauri.app/) (`cargo install tauri-cli --version "^2"`)
- [Node.js 20+](https://nodejs.org/)
- [pnpm](https://pnpm.io/) (`npm install -g pnpm`)

## Dev Setup

```bash
# Clone
git clone https://github.com/dzearing/zecho.git
cd zecho

# Install frontend dependencies
cd ui && pnpm install && cd ..

# Download models (Whisper + Qwen)
./scripts/download-models.sh

# Run in development mode
cargo tauri dev
```

Models are stored in `~/Library/Application Support/zecho/models/`. The app also downloads models on demand through its settings UI.

## Running Tests

```bash
# Rust tests
cd src-tauri && cargo test

# Frontend tests
cd ui && pnpm test

# Frontend tests in watch mode
cd ui && pnpm test:watch
```

## Project Structure

```
src-tauri/           Rust backend (Tauri v2)
  src/
    audio.rs           Audio capture (cpal)
    transcribe.rs      Whisper speech-to-text
    cleanup.rs         Qwen LLM text cleanup
    hotkey.rs          FN key listener (macOS)
    history.rs         Clipboard history storage
    settings.rs        User preferences
    models.rs          Model management & downloads
    macos_panel.rs     Native floating panel
  tests/               Rust unit tests

ui/                  Frontend (Vite + React + TypeScript)
  src/
    tokens/            Design token definitions and CSS generation
    shared/            Shared hooks, atoms, styles, test utilities
    pages/
      pill/            Floating pill window
      settings/        Settings window
      history/         Clipboard history window
      setup/           First-run setup wizard

scripts/             Setup scripts
docs/                Landing page and design docs
```

## Code Conventions

- **Folders**: kebab-case (`pill-states/`, `model-card/`)
- **React components**: PascalCase files (`PillStates.tsx`)
- **Non-component files**: kebab-case (`pill-atoms.ts`, `theme-hook.ts`)
- **CSS Modules**: co-located, matching component name (`PillStates.module.css`)
- **Tests**: co-located (`PillStates.test.tsx` next to `PillStates.tsx`)
- **All filenames must be unique** across the project — no two files share the same name
- **Design tokens**: defined in `ui/src/tokens/tokens.json`, never hardcode colors in CSS
```

- [ ] **Step 2: Update README.md**

Remove the "Getting Started", "Prerequisites", "Setup", and "Project Structure" sections. Replace with a pointer to `CONTRIBUTING.md`. Keep: description, how it works, features, download, license.

The updated README should end the Features section with:

```markdown
## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions, testing, and code conventions.
```

Followed by the existing Download and License sections.

- [ ] **Step 3: Commit**

```bash
git add CONTRIBUTING.md README.md
git commit -m "docs: add CONTRIBUTING.md, slim README to user-facing content"
```

---

## Task 10: Remove Old Frontend Files

**Files:**
- Delete: `ui/index.html`
- Delete: `ui/main.js`
- Delete: `ui/styles.css`
- Delete: `ui/settings.html`
- Delete: `ui/history.html`
- Delete: `ui/setup.html`

- [ ] **Step 1: Verify new frontend builds and tests pass**

Run: `cd ui && pnpm build && pnpm test`
Expected: Build succeeds, all tests pass

- [ ] **Step 2: Remove old files**

```bash
git rm ui/index.html ui/main.js ui/styles.css ui/settings.html ui/history.html ui/setup.html
```

- [ ] **Step 3: Verify build still works after removal**

Run: `cd ui && pnpm build`
Expected: Build succeeds (no references to old files)

- [ ] **Step 4: Commit**

```bash
git commit -m "chore: remove old static HTML/CSS/JS frontend"
```

---

## Task 11: Final Verification

- [ ] **Step 1: Run full test suite**

Run: `cd ui && pnpm test`
Expected: All tests pass

- [ ] **Step 2: Run Vite build**

Run: `cd ui && pnpm build`
Expected: Build succeeds with no warnings

- [ ] **Step 3: Verify .gitignore covers ui/node_modules and ui/dist**

Check that `.gitignore` includes:
```
ui/node_modules/
ui/dist/
```

If not, add them.

- [ ] **Step 4: Run TypeScript type check**

Run: `cd ui && pnpm exec tsc --noEmit`
Expected: No type errors

- [ ] **Step 5: Final commit if needed**

```bash
git add -A
git commit -m "chore: final cleanup — gitignore, type check"
```
