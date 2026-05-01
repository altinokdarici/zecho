use std::path::PathBuf;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct Transcriber {
    ctx: Option<WhisperContext>,
}

unsafe impl Send for Transcriber {}
unsafe impl Sync for Transcriber {}

impl Transcriber {
    pub fn new() -> Self {
        Self { ctx: None }
    }

    pub fn load_model(&mut self, model_path: &std::path::Path) -> Result<(), String> {
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().ok_or("Invalid model path")?,
            WhisperContextParameters::default(),
        )
        .map_err(|e| format!("Failed to load whisper model: {}", e))?;
        self.ctx = Some(ctx);
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.ctx.is_some()
    }

    pub fn transcribe(&self, audio_samples: &[f32]) -> Result<String, String> {
        let ctx = self
            .ctx
            .as_ref()
            .ok_or("Whisper model not loaded. Check Settings for model status.")?;

        if audio_samples.is_empty() {
            return Ok(String::new());
        }

        let mut state = ctx.create_state().map_err(|e| format!("Whisper state error: {}", e))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);

        state
            .full(params, audio_samples)
            .map_err(|e| format!("Whisper transcription error: {}", e))?;

        let num_segments = state
            .full_n_segments()
            .map_err(|e| format!("Segment count error: {}", e))?;

        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }

        Ok(text.trim().to_string())
    }

    pub fn model_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("zecho")
            .join("models")
    }

    pub fn default_model_path() -> PathBuf {
        Self::model_dir().join("ggml-base.en.bin")
    }
}
