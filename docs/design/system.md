# Zecho Design System

## Principles
1. **Invisible until needed** — the pill stays out of the way, reveals on interaction
2. **Every pixel intentional** — no clipping, no wrapping, no orphaned text
3. **Consistent feedback** — every action has a visual response
4. **Native feel** — follows macOS conventions (SF Pro, system colors, blur effects)

## Color Tokens

### Backgrounds
| Token | Value | Usage |
|-------|-------|-------|
| `--bg` | `#0a0a0c` | Landing page background |
| `--bg-app` | `#1c1c1e` | Settings window background |
| `--surface` | `#2c2c2e` | Cards, grouped settings |
| `--surface-hover` | `#3a3a3c` | Hovered cards |
| `--pill-bg` | `rgba(28, 28, 30, 0.92)` | Pill (transparent, blurred) |

### Text
| Token | Value | Usage |
|-------|-------|-------|
| `--text` | `#f5f5f7` | Primary text |
| `--text-secondary` | `#98989d` | Descriptions, labels |
| `--text-dim` | `rgba(255, 255, 255, 0.35)` | Tertiary, timestamps |

### Accents
| Token | Value | Usage |
|-------|-------|-------|
| `--accent` | `#6c5ce7` | Primary brand, selected states, CTAs |
| `--accent-light` | `rgba(108, 92, 231, 0.12)` | Selected card backgrounds |
| `--blue` | `#4a9eff` | Interactive highlights |
| `--red` | `#ff453a` | Recording state, destructive |
| `--green` | `#30d158` | Success, "Copied" state |
| `--orange` | `#ff9f0a` | Warnings |

### Borders
| Token | Value | Usage |
|-------|-------|-------|
| `--border` | `rgba(255, 255, 255, 0.06)` | Default borders |
| `--border-accent` | `rgba(108, 92, 231, 0.2)` | Accent borders |

## Typography

### Font Stack
`-apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", "Helvetica Neue", system-ui, sans-serif`

### Scale
| Size | Weight | Usage |
|------|--------|-------|
| 28px | 700 | Settings page title |
| 17px | 650 | Card titles, feature headings |
| 16px | 650 | Option card titles |
| 15px | 500-600 | Setting row labels |
| 14px | 500 | Nav links, body text |
| 13px | 500 | Pill labels, descriptions |
| 12px | 400-600 | Secondary text, section labels, card descriptions |
| 11px | 400 | Timestamps, footnotes |

### Letter Spacing
- Headers: `-0.02em` to `-0.03em`
- Body: `0` (default)
- Uppercase labels: `0.05em` to `0.06em`

## Spacing
- Section padding: `28px` horizontal
- Card padding: `16px`
- Card gap: `10px`
- Element gap (within cards): `6px`

## Border Radius
| Size | Value | Usage |
|------|-------|-------|
| `--radius-sm` | `8px` | Inner elements, preview boxes, inputs |
| `--radius` | `14px` | Cards, settings groups |
| `--radius-lg` | `22px` | Pill (recording state) |
| `--radius-pill` | `4px` collapsed, `16px-22px` expanded | Pill states |
| `--radius-circle` | `50%` | Buttons, dots, toggles |

## Shadows
| Usage | Value |
|-------|-------|
| Pill | `0 4px 20px rgba(0, 0, 0, 0.5)` |
| Pill (recording) | `0 4px 24px rgba(0, 0, 0, 0.5), 0 0 24px rgba(255, 69, 58, 0.1)` |
| Cards (hover) | `0 12px 40px rgba(0, 0, 0, 0.3)` |
| Panels | `0 8px 32px rgba(0, 0, 0, 0.5)` |

## Animation
| Type | Duration | Easing |
|------|----------|--------|
| Fast (hover, focus) | `0.1s-0.12s` | `ease` |
| Standard (state changes) | `0.15s-0.2s` | `ease` |
| Smooth (pill resize) | `0.3s` | `cubic-bezier(0.34, 1.56, 0.64, 1)` |
| Slide up (panels) | `0.25s` | `cubic-bezier(0.4, 0, 0.2, 1)` |

## Components

### Tooltips
- Custom-styled, not native OS tooltips
- Dark background (`#2c2c2e`), white text, 11px font
- Appears above the element, centered
- 0.4s delay before showing
- Rounded corners (`6px`), subtle shadow
- Arrow/pointer toward the target element

### Buttons
- **Pill action buttons**: 24px circle, transparent bg, `--text-dim` color, hover reveals bg
- **Recording buttons**: 32px circle, colored backgrounds (cancel: white 10%, stop: red 15%)
- **CTA buttons**: Rounded rect, `--accent` bg, white text, 12px radius
- **Setting toggles**: 46x28px track, 24px knob, slides 18px

### Cards
- `--surface` background, `--border` border, `--radius` corners
- Selected: `--accent` border, `--accent-light` background
- Hover: `--surface-hover` background
- Preview boxes inside cards: `--preview-bg`, `--preview-text`, `8px` radius

### Icons
- Stroke-based SVGs, `currentColor`, 1.2-1.5px stroke width
- 14px in pill actions, 16-18px in feature areas, 24px in settings
- Must be recognizable at their rendered size
- History: clock icon
- Settings: gear/cog icon (NOT sun/brightness)
- Cancel: X icon
- Stop: filled rounded square
- Success: checkmark

### Pill States
| State | Height | Width | Visual |
|-------|--------|-------|--------|
| Collapsed | 8px | 80px | Thin bar with center dot |
| Hover | 32px | 120px | History + dot + Settings buttons |
| Recording | 44px | 200px | Cancel + waveform + Stop |
| Processing | 36px | 160px | Spinner + "Processing" |
| Done | 36px | 130px | Checkmark + "Copied" (green) |
