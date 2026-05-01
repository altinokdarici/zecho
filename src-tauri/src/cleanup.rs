use crate::settings::{CleanupLevel, WritingStyle};

pub struct TextCleaner;

impl TextCleaner {
    pub fn new() -> Self {
        Self
    }

    pub fn build_prompt(
        raw_text: &str,
        style: &WritingStyle,
        level: &CleanupLevel,
        custom_prompt: Option<&str>,
    ) -> String {
        let style_instruction = match style {
            WritingStyle::Formal => {
                "Use proper capitalization and full punctuation (periods, commas, semicolons). \
                 Write in complete, well-structured sentences."
            }
            WritingStyle::Casual => {
                "Use standard capitalization but lighter punctuation. \
                 Sentences can be shorter and more conversational."
            }
            WritingStyle::VeryCasual => {
                "Use lowercase throughout. Minimal punctuation. \
                 Keep the tone relaxed and informal, like a text message."
            }
        };

        let level_instruction = match level {
            CleanupLevel::None => {
                "Transcribe exactly what was said, including mistakes and filler words. \
                 Only apply the writing style formatting."
            }
            CleanupLevel::Light => {
                "Remove filler words (um, uh, like, you know). \
                 Fix basic grammar. Keep the original phrasing."
            }
            CleanupLevel::Medium => {
                "Remove filler words. Fix grammar. \
                 Edit for clarity and conciseness while keeping the speaker's voice. \
                 If the speaker corrects themselves, keep only the correction."
            }
            CleanupLevel::High => {
                "Rewrite for brevity and polish. Remove all filler and redundancy. \
                 If the speaker corrects themselves, keep only the correction. \
                 Tighten the language while preserving the intended meaning."
            }
        };

        let custom = custom_prompt
            .map(|p| format!("\n\nAdditional instructions: {}", p))
            .unwrap_or_default();

        format!(
            "Clean up this voice transcription.\n\n\
             Writing style: {}\n\n\
             Cleanup level: {}\n\n\
             Do not add information that wasn't spoken. \
             Return ONLY the cleaned text, no explanations.{}\n\n\
             Transcription: {}",
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

        // TODO: Send prompt to local Qwen model via llama.cpp
        // For now, do basic cleanup inline
        let _prompt = Self::build_prompt(raw_text, style, level, custom_prompt);
        Ok(Self::basic_cleanup(raw_text, style))
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
        let result = result
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        Self::apply_style_only(&result, style)
    }
}
