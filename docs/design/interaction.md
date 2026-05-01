# Zecho Interaction Design

## Core Interaction Flow

### Pill Widget
- The pill is a small floating overlay that sits at the bottom center of the screen
- It overlays all other apps (always on top)
- It is draggable — the user can reposition it anywhere on screen
- Default state: collapsed, ~8px tall, subtle dark bar with a center dot
- On hover: expands to ~32px showing history (left) and settings (right) buttons

### Recording

#### Starting a recording
- **Hold FN key**: Press and hold the FN (globe) key to start recording. Audio capture begins immediately. The pill animates to its recording state (44px tall, shows cancel X, waveform, stop square)
- **Double-tap FN**: Tap FN twice quickly to "lock" recording — the user doesn't have to hold the key down. Recording continues until explicitly stopped.

#### During recording
- The pill shows: [X cancel] [waveform bars] [stop square]
- The waveform animates based on audio input levels
- The user speaks naturally — filler words, corrections, pauses are all fine

#### Stopping a recording
- **Release FN key** (if holding): Recording stops, processing begins
- **Press FN once** (if locked): Recording stops, processing begins  
- **Click stop button**: Recording stops, processing begins
- **Press Escape**: Recording is cancelled, no processing occurs
- **Click X button**: Recording is cancelled, no processing occurs

### Processing
- After recording stops, the pill shows a spinner with "Processing"
- Audio is transcribed via Whisper (local STT)
- Transcription is cleaned up via the configured cleanup model and settings
- Clean text is copied to clipboard
- If auto-paste is enabled, Cmd+V is simulated to paste into the active app

### Done State
- Pill briefly shows a green checkmark with "Copied" (1.2s)
- Returns to collapsed idle state

### History
- Click the history button (clock icon) on the pill to expand the history panel
- History panel slides up above the pill
- Shows a scrollable list of past transcriptions (truncated, one line each)
- Each item shows the cleaned text and a relative timestamp
- Click an item to copy it to clipboard
- Hover an item to reveal a delete button (X)
- Click outside or the close button to dismiss

### Settings
- Click the gear button on the pill to open the Settings window
- Settings is a separate native window (decorated, centered)
- Contains: Writing Style picker, Cleanup Level picker, Model management, General settings, Custom prompt

## System Tray
- Zecho appears in the macOS menu bar with a small icon
- Right-click or click shows: Show Zecho, Settings, Quit

## Global Hotkey (FN Key)

### macOS Implementation
The FN/Globe key on macOS requires special handling — it's not a regular key that global shortcut APIs can capture. Implementation options:
1. **CGEventTap** (Core Graphics): Create an event tap that monitors key events at the system level. The FN key sends `kCGEventFlagsChanged` events with the `kCGEventFlagMaskSecondaryFn` flag.
2. **Accessibility permissions**: The app needs Accessibility permissions in System Preferences > Privacy & Security to capture global key events.

### Fallback Hotkey
If the user hasn't granted Accessibility permissions, or on platforms where FN capture isn't possible, fall back to a configurable keyboard shortcut (default: Ctrl+Shift+R). The user can change this in Settings.

## Window Behavior
- The pill window uses `alwaysOnTop: true` to stay above all apps
- The pill window is transparent and frameless (no title bar)
- The pill window should be draggable via `data-tauri-drag-region`
- The pill window height should accommodate the expanded history panel
- Position: bottom center of screen by default, user-repositionable

## Auto-Paste
After text is copied to clipboard, simulate Cmd+V (macOS) or Ctrl+V (Windows) to paste into whatever app was active before the recording started. This can be toggled off in Settings.
