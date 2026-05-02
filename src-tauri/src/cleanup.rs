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
            WritingStyle::Formal => "Style: proper capitalization, full punctuation, complete sentences.",
            WritingStyle::Casual => "Style: standard caps, lighter punctuation, conversational.",
            WritingStyle::VeryCasual => "Style: all lowercase, minimal punctuation, like a text message.",
        };

        let level_instruction = match level {
            CleanupLevel::None => "Do not change the words. Only apply formatting.",
            CleanupLevel::Light => "Remove filler words (um, uh, like, you know, so, basically). Fix grammar. Keep original phrasing.",
            CleanupLevel::Medium => "Remove filler words. Fix grammar. Resolve self-corrections. Edit for clarity and conciseness.",
            CleanupLevel::High => "Remove all filler. Resolve all self-corrections. Rewrite for maximum brevity and polish while preserving meaning.",
        };

        let custom = custom_prompt
            .map(|p| format!("\n\nAdditional instructions: {}", p))
            .unwrap_or_default();

        format!(
            "<|im_start|>system\nYou are a voice transcription editor. Your ONLY job is to clean up spoken text into polished written text.\n\n\
            CRITICAL RULES:\n\
            1. SELF-CORRECTIONS: When the speaker changes their mind, KEEP ONLY THE FINAL VERSION. Examples:\n\
               - \"I want red, no actually purple\" -> \"I want purple\"\n\
               - \"Let's meet Monday, wait, Tuesday\" -> \"Let's meet Tuesday\"\n\
               - \"The background should be green, uhhh actually purple\" -> \"The background should be purple\"\n\
               - \"Send it to John, I mean Sarah\" -> \"Send it to Sarah\"\n\
            2. FILLER WORDS: Remove um, uh, like, you know, so, basically, actually (when used as filler)\n\
            3. PRESERVE MEANING: Never add information. Never change the speaker's intent.\n\
            4. OUTPUT: Return ONLY the cleaned text. No explanations, no quotes, no prefixes.\n\n\
            {}\n{}{}<|im_end|>\n\
            <|im_start|>user\n{}<|im_end|>\n\
            <|im_start|>assistant\n",
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
            " um ", " uh ", " like, ", " you know, ", " basically, ",
            " um, ", " uh, ", " uhh ", " uhh, ", " umm ", " umm, ",
        ];
        let mut result = format!(" {} ", text);
        for filler in &fillers {
            while result.contains(filler) {
                result = result.replacen(filler, " ", 1);
            }
        }

        // Basic self-correction: "X, actually Y" / "X, no Y" / "X, I mean Y"
        let correction_patterns = [
            ", actually ", ", no actually ", ", no ", ", wait, ", ", I mean ",
            ". Actually, ", ". No, ", ". Wait, ",
        ];
        for pattern in &correction_patterns {
            if let Some(pos) = result.rfind(pattern) {
                let before_correction = &result[..pos];
                let after_correction = &result[pos + pattern.len()..];
                if let Some(sentence_start) = before_correction.rfind(". ") {
                    result = format!("{}. {}", &before_correction[..sentence_start], after_correction);
                } else {
                    result = after_correction.to_string();
                }
            }
        }

        let result = result.split_whitespace().collect::<Vec<_>>().join(" ");
        Self::apply_style_only(&result, style)
    }
}
