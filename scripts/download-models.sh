#!/bin/bash
set -euo pipefail

MODEL_DIR="${HOME}/Library/Application Support/zecho/models"
WHISPER_MODEL="ggml-base.en.bin"
WHISPER_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/${WHISPER_MODEL}"

mkdir -p "${MODEL_DIR}"

if [ -f "${MODEL_DIR}/${WHISPER_MODEL}" ]; then
    echo "Whisper model already exists at ${MODEL_DIR}/${WHISPER_MODEL}"
    echo "Delete it and re-run to re-download."
else
    echo "Downloading whisper model (${WHISPER_MODEL})..."
    echo "This is ~142MB and may take a minute."
    curl -L --progress-bar -o "${MODEL_DIR}/${WHISPER_MODEL}" "${WHISPER_URL}"
    echo "Done. Model saved to ${MODEL_DIR}/${WHISPER_MODEL}"
fi

echo ""
echo "Models ready. You can now run: cargo tauri dev"
