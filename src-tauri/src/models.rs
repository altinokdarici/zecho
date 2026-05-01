use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub size_mb: u32,
    pub quality_score: u8,
    pub speed_score: u8,
    pub filename: &'static str,
    pub url: &'static str,
    pub model_type: ModelType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    Whisper,
    Cleanup,
}

pub const AVAILABLE_MODELS: &[ModelInfo] = &[
    // Whisper models
    ModelInfo {
        id: "whisper-tiny-en",
        name: "Whisper Tiny (English)",
        description: "Fastest transcription, good for short phrases",
        size_mb: 75,
        quality_score: 6,
        speed_score: 10,
        filename: "ggml-tiny.en.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin",
        model_type: ModelType::Whisper,
    },
    ModelInfo {
        id: "whisper-base-en",
        name: "Whisper Base (English)",
        description: "Recommended — good balance of speed and accuracy",
        size_mb: 142,
        quality_score: 8,
        speed_score: 8,
        filename: "ggml-base.en.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin",
        model_type: ModelType::Whisper,
    },
    ModelInfo {
        id: "whisper-small-en",
        name: "Whisper Small (English)",
        description: "High accuracy, slower on older hardware",
        size_mb: 466,
        quality_score: 9,
        speed_score: 5,
        filename: "ggml-small.en.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin",
        model_type: ModelType::Whisper,
    },
    // Cleanup models — Qwen 3 family (latest)
    ModelInfo {
        id: "qwen3-0.6b",
        name: "Qwen 3 0.6B",
        description: "Fastest cleanup — good for simple text",
        size_mb: 397,
        quality_score: 7,
        speed_score: 10,
        filename: "Qwen3-0.6B-Q4_K_M.gguf",
        url: "https://huggingface.co/unsloth/Qwen3-0.6B-GGUF/resolve/main/Qwen3-0.6B-Q4_K_M.gguf",
        model_type: ModelType::Cleanup,
    },
    ModelInfo {
        id: "qwen3-1.7b",
        name: "Qwen 3 1.7B",
        description: "Recommended — great balance of speed and quality",
        size_mb: 1100,
        quality_score: 8,
        speed_score: 8,
        filename: "Qwen3-1.7B-Q4_K_M.gguf",
        url: "https://huggingface.co/unsloth/Qwen3-1.7B-GGUF/resolve/main/Qwen3-1.7B-Q4_K_M.gguf",
        model_type: ModelType::Cleanup,
    },
    ModelInfo {
        id: "qwen3-3.6b",
        name: "Qwen 3 3.6B",
        description: "Highest quality — best for nuanced text cleanup",
        size_mb: 2300,
        quality_score: 10,
        speed_score: 6,
        filename: "Qwen3-3.6B-Q4_K_M.gguf",
        url: "https://huggingface.co/unsloth/Qwen3-3.6B-GGUF/resolve/main/Qwen3-3.6B-Q4_K_M.gguf",
        model_type: ModelType::Cleanup,
    },
];

pub fn model_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zecho")
        .join("models")
}

pub fn get_model(id: &str) -> Option<&'static ModelInfo> {
    AVAILABLE_MODELS.iter().find(|m| m.id == id)
}

pub fn model_path(info: &ModelInfo) -> PathBuf {
    model_dir().join(info.filename)
}

pub fn is_downloaded(info: &ModelInfo) -> bool {
    model_path(info).exists()
}

pub fn default_whisper_model() -> &'static ModelInfo {
    get_model("whisper-base-en").unwrap()
}

pub fn default_cleanup_model() -> &'static ModelInfo {
    get_model("qwen3-1.7b").unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub id: String,
    pub name: String,
    pub description: String,
    pub size_mb: u32,
    pub quality_score: u8,
    pub speed_score: u8,
    pub downloaded: bool,
    pub model_type: ModelType,
}

pub fn list_models() -> Vec<ModelStatus> {
    AVAILABLE_MODELS
        .iter()
        .map(|m| ModelStatus {
            id: m.id.to_string(),
            name: m.name.to_string(),
            description: m.description.to_string(),
            size_mb: m.size_mb,
            quality_score: m.quality_score,
            speed_score: m.speed_score,
            downloaded: is_downloaded(m),
            model_type: m.model_type.clone(),
        })
        .collect()
}

pub fn download_model_blocking(info: &ModelInfo) -> Result<PathBuf, String> {
    let path = model_path(info);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let response = reqwest::blocking::get(info.url).map_err(|e| format!("Download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let bytes = response.bytes().map_err(|e| format!("Failed to read response: {}", e))?;
    std::fs::write(&path, &bytes).map_err(|e| format!("Failed to write model file: {}", e))?;

    Ok(path)
}
