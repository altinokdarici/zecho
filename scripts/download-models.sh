#!/bin/bash
set -euo pipefail

MODEL_DIR="${HOME}/Library/Application Support/zecho/models"
mkdir -p "${MODEL_DIR}"

download_model() {
    local name="$1"
    local filename="$2"
    local url="$3"
    local size="$4"

    if [ -f "${MODEL_DIR}/${filename}" ]; then
        echo "  ${name} already downloaded"
    else
        echo "  Downloading ${name} (~${size})..."
        curl -L --progress-bar -o "${MODEL_DIR}/${filename}" "${url}"
        echo "  Done."
    fi
}

echo "Zecho Model Setup"
echo "=================="
echo ""
echo "Downloading to: ${MODEL_DIR}"
echo ""

echo "Speech-to-Text (Whisper):"
download_model \
    "Whisper Base (English)" \
    "ggml-base.en.bin" \
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin" \
    "142MB"

download_model \
    "Whisper Base (Multilingual)" \
    "ggml-base.bin" \
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" \
    "142MB"

echo ""
echo "Text Cleanup (Qwen 3):"
download_model \
    "Qwen 3 0.6B (Q4_K_M)" \
    "Qwen3-0.6B-Q4_K_M.gguf" \
    "https://huggingface.co/unsloth/Qwen3-0.6B-GGUF/resolve/main/Qwen3-0.6B-Q4_K_M.gguf" \
    "397MB"

echo ""
echo "All models ready. Run: cargo tauri dev"
