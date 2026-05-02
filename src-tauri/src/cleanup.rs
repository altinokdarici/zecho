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

        let model_params = LlamaModelParams::default().with_n_gpu_layers(0);
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

        let tokens = model
            .str_to_token(&prompt, AddBos::Always)
            .map_err(|e| format!("Tokenize error: {}", e))?;

        let n_ctx = std::cmp::max(tokens.len() as u32 + 256, 1024);
        let ctx_params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(n_ctx));
        let mut ctx = model
            .new_context(backend, ctx_params)
            .map_err(|e| format!("Context error: {}", e))?;

        println!("LLM cleanup: {} prompt tokens, {} ctx size", tokens.len(), n_ctx);

        let mut batch = LlamaBatch::new(n_ctx as usize, 1);
        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch
                .add(*token, i as i32, &[0], is_last)
                .map_err(|e| format!("Batch add error: {}", e))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| format!("Decode error: {}", e))?;

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(0.1),
            LlamaSampler::top_p(0.95, 1),
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
            let piece = match model.token_to_str(token, Special::Tokenize) {
                Ok(s) => s,
                Err(_) => break,
            };

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
        println!("LLM cleanup result: {:?}", result);
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
        let mut words: Vec<&str> = text.split_whitespace().collect();

        // Remove filler words (case-insensitive)
        let fillers = [
            "um", "um,", "umm", "umm,", "uh", "uh,", "uhh", "uhh,",
            "er", "er,", "err", "err,", "ah", "ah,",
        ];
        words.retain(|w| {
            let lower = w.to_lowercase();
            let stripped = lower.trim_end_matches(['.', ',', '!', '?']);
            !fillers.contains(&lower.as_str()) && !fillers.contains(&stripped)
        });

        // Remove filler phrases
        let mut result = words.join(" ");
        let filler_phrases = [
            "you know,", "you know", "like,", "basically,", "basically",
            "sort of", "kind of", "I guess", "I mean,",
        ];
        for phrase in &filler_phrases {
            while let Some(pos) = result.to_lowercase().find(&phrase.to_lowercase()) {
                let end = pos + phrase.len();
                let after = result[end..].trim_start_matches([' ', ',']).to_string();
                result = format!("{}{}", &result[..pos], after);
            }
        }

        // Handle self-corrections: find correction markers and keep only the correction
        let correction_markers = [
            " actually ", " actually, ", " no actually ", " no, actually ",
            " no no ", " no no, ", " wait ", " wait, ",
            " I mean ", " I meant ", " rather ", " or rather ",
            " not ", // "I want red, not blue" -> tricky, keep both
        ];

        // Process corrections from right to left (handle nested corrections)
        for marker in &correction_markers {
            if *marker == " not " {
                continue; // "not" is too ambiguous
            }
            let lower = result.to_lowercase();
            if let Some(pos) = lower.rfind(&marker.to_lowercase()) {
                let before = result[..pos].trim();
                let after = result[pos + marker.len()..].trim();

                if after.is_empty() {
                    continue;
                }

                // Find the start of the clause being corrected
                // Look for the last sentence boundary or comma before the marker
                let clause_start = before.rfind(". ")
                    .map(|p| p + 2)
                    .or_else(|| before.rfind(", ").map(|p| p + 2))
                    .unwrap_or(0);

                // Reconstruct: everything before the corrected clause + the correction
                let prefix = &before[..clause_start];
                // Capitalize the correction if it's at the start
                let corrected = if prefix.is_empty() {
                    let mut chars = after.chars();
                    match chars.next() {
                        Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
                        None => after.to_string(),
                    }
                } else {
                    after.to_string()
                };

                result = if prefix.is_empty() {
                    corrected
                } else {
                    format!("{}{}", prefix, corrected)
                };
            }
        }

        // Clean up double spaces and trailing commas
        result = result.replace("  ", " ").replace(" ,", ",").replace(" .", ".");
        let result = result.trim().to_string();

        Self::apply_style_only(&result, style)
    }
}
