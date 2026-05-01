use crate::settings::{CleanupLevel, WritingStyle};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;
use std::num::NonZeroU32;
use std::path::Path;

pub struct TextCleaner {
    backend: Option<LlamaBackend>,
    model: Option<LlamaModel>,
}

unsafe impl Send for TextCleaner {}
unsafe impl Sync for TextCleaner {}

impl TextCleaner {
    pub fn new() -> Self {
        Self {
            backend: None,
            model: None,
        }
    }

    pub fn load_model(&mut self, model_path: &Path) -> Result<(), String> {
        let backend =
            LlamaBackend::init().map_err(|e| format!("Failed to init llama backend: {}", e))?;

        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
            .map_err(|e| format!("Failed to load cleanup model: {}", e))?;

        self.backend = Some(backend);
        self.model = Some(model);
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.model.is_some()
    }

    pub fn build_prompt(
        raw_text: &str,
        style: &WritingStyle,
        level: &CleanupLevel,
        custom_prompt: Option<&str>,
    ) -> String {
        let style_instruction = match style {
            WritingStyle::Formal => {
                "Use proper capitalization and full punctuation. Write in complete, well-structured sentences."
            }
            WritingStyle::Casual => {
                "Use standard capitalization but lighter punctuation. Conversational tone."
            }
            WritingStyle::VeryCasual => {
                "Use lowercase throughout. Minimal punctuation. Relaxed, like a text message."
            }
        };

        let level_instruction = match level {
            CleanupLevel::None => "Only apply the writing style formatting. Keep everything else as-is.",
            CleanupLevel::Light => "Remove filler words (um, uh, like, you know). Fix basic grammar.",
            CleanupLevel::Medium => {
                "Remove filler words. Fix grammar. Edit for clarity. If the speaker corrects themselves, keep only the correction."
            }
            CleanupLevel::High => {
                "Rewrite for brevity and polish. Remove all filler and redundancy. Keep only corrections. Tighten the language."
            }
        };

        let custom = custom_prompt
            .map(|p| format!("\n{}", p))
            .unwrap_or_default();

        format!(
            "<|im_start|>system\nYou clean up voice transcriptions. Rules:\n- {}\n- {}\n- Do NOT add information that wasn't spoken\n- Return ONLY the cleaned text, nothing else{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
            style_instruction, level_instruction, custom, raw_text
        )
    }

    pub fn clean(
        &self,
        raw_text: &str,
        style: &WritingStyle,
        level: &CleanupLevel,
        custom_prompt: Option<&str>,
    ) -> Result<String, String> {
        if *level == CleanupLevel::None {
            return Ok(Self::apply_style_only(raw_text, style));
        }

        if let (Some(model), Some(backend)) = (&self.model, &self.backend) {
            self.run_llm(model, backend, raw_text, style, level, custom_prompt)
        } else {
            Ok(Self::basic_cleanup(raw_text, style))
        }
    }

    fn run_llm(
        &self,
        model: &LlamaModel,
        backend: &LlamaBackend,
        raw_text: &str,
        style: &WritingStyle,
        level: &CleanupLevel,
        custom_prompt: Option<&str>,
    ) -> Result<String, String> {
        let prompt = Self::build_prompt(raw_text, style, level, custom_prompt);

        let ctx_params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(512));
        let mut ctx = model
            .new_context(backend, ctx_params)
            .map_err(|e| format!("Context error: {}", e))?;

        let tokens = model
            .str_to_token(&prompt, AddBos::Always)
            .map_err(|e| format!("Tokenize error: {}", e))?;

        let mut batch = LlamaBatch::new(512, 1);
        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch
                .add(*token, i as i32, &[0], is_last)
                .map_err(|e| format!("Batch add error: {}", e))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| format!("Decode error: {}", e))?;

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(0.3),
            LlamaSampler::top_p(0.9, 1),
            LlamaSampler::dist(42),
        ]);

        let mut output = String::new();
        let mut n_cur = tokens.len() as i32;
        let max_tokens = 256;

        for _ in 0..max_tokens {
            let token = sampler.sample(&ctx, -1);

            if model.is_eog_token(token) {
                break;
            }

            #[allow(deprecated)]
            let piece = model
                .token_to_str(token, Special::Tokenize)
                .map_err(|e| format!("Detokenize error: {}", e))?;

            output.push_str(&piece);
            if output.contains("<|im_end|>") || output.contains("<|endoftext|>") {
                output = output
                    .replace("<|im_end|>", "")
                    .replace("<|endoftext|>", "");
                break;
            }

            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .map_err(|e| format!("Batch add error: {}", e))?;
            n_cur += 1;

            ctx.decode(&mut batch)
                .map_err(|e| format!("Decode error: {}", e))?;
        }

        let result = output.trim().to_string();
        if result.is_empty() {
            Ok(Self::basic_cleanup(raw_text, style))
        } else {
            Ok(result)
        }
    }

    fn apply_style_only(text: &str, style: &WritingStyle) -> String {
        match style {
            WritingStyle::Formal => text.to_string(),
            WritingStyle::Casual => text.to_string(),
            WritingStyle::VeryCasual => text.to_lowercase(),
        }
    }

    fn basic_cleanup(text: &str, style: &WritingStyle) -> String {
        let fillers = [
            " um ", " uh ", " like, ", " you know, ", " basically, ", " actually, ",
            " um, ", " uh, ",
        ];
        let mut result = format!(" {} ", text);
        for filler in &fillers {
            while result.contains(filler) {
                result = result.replacen(filler, " ", 1);
            }
        }
        let result = result.split_whitespace().collect::<Vec<_>>().join(" ");
        Self::apply_style_only(&result, style)
    }
}
