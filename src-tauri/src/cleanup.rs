use crate::settings::{CleanupLevel, WritingStyle};
use std::path::Path;
use std::sync::mpsc;

pub struct CleanupRequest {
    pub raw_text: String,
    pub style: WritingStyle,
    pub level: CleanupLevel,
    pub custom_prompt: Option<String>,
    pub reply: mpsc::Sender<Result<String, String>>,
}

pub struct TextCleaner {
    sender: Option<mpsc::Sender<CleanupRequest>>,
}

impl TextCleaner {
    pub fn new() -> Self {
        Self { sender: None }
    }

    pub fn start_worker(&mut self, model_path: &Path) -> Result<(), String> {
        let path = model_path.to_path_buf();
        let (tx, rx) = mpsc::channel::<CleanupRequest>();

        // Run LLM inference in a subprocess to avoid backend conflicts with whisper.
        // The test_llm binary is compiled alongside zecho and handles all llama.cpp calls.
        std::thread::spawn(move || {
            println!("Cleanup worker: subprocess mode, model at {}", path.display());

            for req in rx {
                let result = run_subprocess(&path, &req);
                let _ = req.reply.send(result);
            }

            fn run_subprocess(model_path: &std::path::Path, req: &CleanupRequest) -> Result<String, String> {
                let prompt = build_prompt(&req.raw_text, &req.style, &req.level, req.custom_prompt.as_deref());

                // Find the test_llm binary next to the zecho binary
                let exe = std::env::current_exe().map_err(|e| e.to_string())?;
                let test_llm = exe.parent().unwrap().join("test_llm");
                if !test_llm.exists() {
                    return Err(format!("test_llm binary not found at {}", test_llm.display()));
                }

                let output = std::process::Command::new(&test_llm)
                    .arg(model_path.to_str().unwrap())
                    .arg(&prompt)
                    .output()
                    .map_err(|e| format!("Failed to run cleanup subprocess: {}", e))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("Cleanup subprocess failed: {}", stderr));
                }

                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if stdout.is_empty() {
                    Ok(req.raw_text.clone())
                } else {
                    Ok(stdout)
                }
            }
        });

        self.sender = Some(tx);
        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.sender.is_some()
    }

    pub fn clean(
        &self,
        raw_text: &str,
        style: &WritingStyle,
        level: &CleanupLevel,
        custom_prompt: Option<&str>,
    ) -> Result<String, String> {
        if *level == CleanupLevel::None {
            return Ok(raw_text.to_string());
        }

        if let Some(sender) = &self.sender {
            let (reply_tx, reply_rx) = mpsc::channel();
            let req = CleanupRequest {
                raw_text: raw_text.to_string(),
                style: style.clone(),
                level: level.clone(),
                custom_prompt: custom_prompt.map(|s| s.to_string()),
                reply: reply_tx,
            };

            if sender.send(req).is_err() {
                return Ok(raw_text.to_string());
            }

            match reply_rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(result) => result,
                Err(_) => {
                    eprintln!("Cleanup timed out, using raw text");
                    Ok(raw_text.to_string())
                }
            }
        } else {
            Ok(raw_text.to_string())
        }
    }
}

fn build_prompt(
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
        CleanupLevel::Light => "Remove filler words (um, uh, like, you know). Fix grammar.",
        CleanupLevel::Medium => "Remove filler words. Fix grammar. Resolve self-corrections. Edit for clarity.",
        CleanupLevel::High => "Remove all filler. Resolve all self-corrections. Rewrite for maximum brevity while preserving meaning.",
    };

    let custom = custom_prompt
        .map(|p| format!("\n\nAdditional instructions: {}", p))
        .unwrap_or_default();

    format!(
        "<|im_start|>system\nYou are a voice transcription editor. Clean up spoken text into polished written text.\n\n\
        CRITICAL RULES:\n\
        1. SELF-CORRECTIONS: When the speaker changes their mind, KEEP ONLY THE FINAL VERSION.\n\
           - \"I want red, no actually purple\" -> \"I want purple\"\n\
           - \"Let's meet Monday, wait, Tuesday\" -> \"Let's meet Tuesday\"\n\
           - \"The background should be green, uhhh actually purple\" -> \"The background should be purple\"\n\
        2. FILLER WORDS: Remove um, uh, like, you know, so, basically\n\
        3. PRESERVE MEANING: Never add information not spoken.\n\
        4. OUTPUT: Return ONLY the cleaned text. No explanations.\n\n\
        {}\n{}{}<|im_end|>\n\
        <|im_start|>user\n{}<|im_end|>\n\
        <|im_start|>assistant\n",
        style_instruction, level_instruction, custom, raw_text
    )
}
