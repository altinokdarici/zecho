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
    pub multilingual: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    Whisper,
    Cleanup,
}

pub const AVAILABLE_MODELS: &[ModelInfo] = &[
    // Whisper models — English-only (faster for English)
    ModelInfo {
        id: "whisper-tiny-en",
        name: "Fast",
        description: "Quickest transcription. Good for short, clear speech.",
        size_mb: 75,
        quality_score: 6,
        speed_score: 10,
        filename: "ggml-tiny.en.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin",
        model_type: ModelType::Whisper,
        multilingual: false,
    },
    ModelInfo {
        id: "whisper-base-en",
        name: "Balanced",
        description: "Recommended — accurate and fast for everyday use.",
        size_mb: 142,
        quality_score: 8,
        speed_score: 8,
        filename: "ggml-base.en.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin",
        model_type: ModelType::Whisper,
        multilingual: false,
    },
    ModelInfo {
        id: "whisper-small-en",
        name: "Accurate",
        description: "Best accuracy. Uses more memory and is slower.",
        size_mb: 466,
        quality_score: 9,
        speed_score: 5,
        filename: "ggml-small.en.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin",
        model_type: ModelType::Whisper,
        multilingual: false,
    },
    // Whisper models — Multilingual
    ModelInfo {
        id: "whisper-tiny-multi",
        name: "Fast",
        description: "Quickest transcription. Multilingual support.",
        size_mb: 75,
        quality_score: 5,
        speed_score: 10,
        filename: "ggml-tiny.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin",
        model_type: ModelType::Whisper,
        multilingual: true,
    },
    ModelInfo {
        id: "whisper-base-multi",
        name: "Balanced",
        description: "Recommended — accurate and fast. Multilingual support.",
        size_mb: 142,
        quality_score: 7,
        speed_score: 8,
        filename: "ggml-base.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
        model_type: ModelType::Whisper,
        multilingual: true,
    },
    ModelInfo {
        id: "whisper-small-multi",
        name: "Accurate",
        description: "Best accuracy. Multilingual support.",
        size_mb: 466,
        quality_score: 8,
        speed_score: 5,
        filename: "ggml-small.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
        model_type: ModelType::Whisper,
        multilingual: true,
    },
    // Cleanup models
    ModelInfo {
        id: "qwen25-1.5b",
        name: "Balanced",
        description: "Recommended — reliable cleanup with good speed.",
        size_mb: 986,
        quality_score: 8,
        speed_score: 8,
        filename: "Qwen2.5-1.5B-Instruct-Q4_K_M.gguf",
        url: "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf",
        model_type: ModelType::Cleanup,
        multilingual: false,
    },
    ModelInfo {
        id: "qwen25-3b",
        name: "Quality",
        description: "Best at handling corrections and nuance. Slower.",
        size_mb: 1940,
        quality_score: 9,
        speed_score: 6,
        filename: "Qwen2.5-3B-Instruct-Q4_K_M.gguf",
        url: "https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q4_k_m.gguf",
        model_type: ModelType::Cleanup,
        multilingual: false,
    },
    ModelInfo {
        id: "gemma4-e2b",
        name: "Gemma 4 E2B",
        description: "Google Gemma 4 — fast, high-quality cleanup.",
        size_mb: 2963,
        quality_score: 9,
        speed_score: 7,
        filename: "gemma-4-E2B-it-Q4_K_M.gguf",
        url: "https://huggingface.co/unsloth/gemma-4-E2B-it-GGUF/resolve/main/gemma-4-E2B-it-Q4_K_M.gguf",
        model_type: ModelType::Cleanup,
        multilingual: false,
    },
    ModelInfo {
        id: "gemma4-e4b",
        name: "Gemma 4 E4B",
        description: "Google Gemma 4 — best quality, larger download.",
        size_mb: 4747,
        quality_score: 10,
        speed_score: 5,
        filename: "gemma-4-E4B-it-Q4_K_M.gguf",
        url: "https://huggingface.co/unsloth/gemma-4-E4B-it-GGUF/resolve/main/gemma-4-E4B-it-Q4_K_M.gguf",
        model_type: ModelType::Cleanup,
        multilingual: false,
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

pub fn whisper_model_for_language(current_id: &str, language: &str) -> String {
    let need_multilingual = language != "en";
    let current = get_model(current_id);
    let current_is_multilingual = current.map(|m| m.multilingual).unwrap_or(false);

    if need_multilingual == current_is_multilingual {
        return current_id.to_string();
    }

    let tier = if current_id.contains("tiny") {
        "tiny"
    } else if current_id.contains("small") {
        "small"
    } else {
        "base"
    };

    let suffix = if need_multilingual { "multi" } else { "en" };
    let target_id = format!("whisper-{}-{}", tier, suffix);

    if get_model(&target_id).is_some() {
        target_id
    } else {
        current_id.to_string()
    }
}

pub fn default_cleanup_model() -> &'static ModelInfo {
    get_model("qwen25-1.5b").unwrap()
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
    pub multilingual: bool,
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
            multilingual: m.multilingual,
        })
        .collect()
}

pub fn download_model_blocking(
    info: &ModelInfo,
    mut on_progress: impl FnMut(u64, u64),
) -> Result<PathBuf, String> {
    use std::io::{Read, Write};

    let path = model_path(info);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let response = reqwest::blocking::get(info.url).map_err(|e| format!("Download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total = response.content_length().unwrap_or(info.size_mb as u64 * 1024 * 1024);
    let tmp_path = path.with_extension("tmp");
    let mut file = std::fs::File::create(&tmp_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut downloaded: u64 = 0;
    let mut reader = response;
    let mut buf = [0u8; 65536];

    loop {
        let n = reader.read(&mut buf).map_err(|e| format!("Read error: {}", e))?;
        if n == 0 { break; }
        file.write_all(&buf[..n]).map_err(|e| format!("Write error: {}", e))?;
        downloaded += n as u64;
        on_progress(downloaded, total);
    }

    file.flush().map_err(|e| e.to_string())?;
    drop(file);
    std::fs::rename(&tmp_path, &path).map_err(|e| format!("Failed to rename: {}", e))?;

    Ok(path)
}
