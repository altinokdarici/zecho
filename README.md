# Zecho

**Voice to clipboard, instantly.** Speak naturally, get clean text -- no subscriptions, no cloud, no ads. Everything runs locally on your machine.

Hold the **FN key** to record, release to get polished text copied to your clipboard. That's it.

## How It Works

1. **Speak** -- Hold FN and talk naturally. Corrections, pauses, and filler words are all fine.
2. **AI cleans** -- Local Whisper transcribes your speech. A local Qwen LLM removes filler words, fixes grammar, and resolves self-corrections ("I want red, uh, no actually blue" becomes "I want blue").
3. **Paste** -- Clean text lands in your clipboard. Paste anywhere.

## Features

- **Fully local** -- No data leaves your machine. No accounts, no network calls, no telemetry.
- **FN key recording** -- Hold to record, release to transcribe. Double-tap to lock recording.
- **AI text cleanup** -- Removes "um," "uh," filler words, and resolves self-corrections using a local Qwen 2.5 LLM.
- **Writing styles** -- Formal, Casual, or Very Casual. Custom prompts for fine-tuning.
- **Cleanup levels** -- None, Light, Medium, or High -- control how aggressively AI edits your text.
- **Clipboard history** -- Every transcription is saved with before/after comparison (raw transcription vs. cleaned text).
- **macOS native floating panel** -- Hover and interact without stealing focus from your active window.
- **Lightweight** -- Built with Rust and Tauri v2. ~3 MB app overhead.
- **Auto updates** -- New versions are detected and applied automatically.

## Getting Started

### Prerequisites

- macOS 12+
- Rust toolchain (`rustup`)
- Tauri CLI v2 (`cargo install tauri-cli --version "^2"`)

### Setup

```bash
# Clone the repo
git clone https://github.com/dzearing/zecho.git
cd zecho

# Download the default models (Whisper + Qwen)
./scripts/download-models.sh

# Run in development mode
cargo tauri dev
```

The model download script places files in `~/Library/Application Support/zecho/models/`. The app will also download models on demand through its settings UI.

### Project Structure

```
ui/              # Frontend (HTML/CSS/JS)
src-tauri/       # Rust backend (Tauri v2)
  src/
    audio.rs       # Audio capture (cpal)
    transcribe.rs  # Whisper speech-to-text
    cleanup.rs     # Qwen LLM text cleanup
    hotkey.rs      # FN key listener (macOS)
    history.rs     # Clipboard history storage
    settings.rs    # User preferences
    models.rs      # Model management & downloads
    macos_panel.rs # Native floating panel
scripts/         # Setup scripts
docs/            # Landing page (GitHub Pages)
```

## Download

Get the latest release: [github.com/dzearing/zecho/releases/latest](https://github.com/dzearing/zecho/releases/latest)

## License

Apache 2.0
