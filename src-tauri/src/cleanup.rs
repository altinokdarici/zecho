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

        // Run LLM inference in a persistent subprocess to avoid backend conflicts with whisper.
        // The test_llm binary stays alive and accepts prompts via stdin (one per line).
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader, Write};
            use std::process::{Command, Stdio};

            let exe = match std::env::current_exe() {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Cleanup worker: can't find exe: {}", e);
                    return;
                }
            };
            let test_llm = exe.parent().unwrap().join("test_llm");
            if !test_llm.exists() {
                eprintln!("Cleanup worker: test_llm not found at {}", test_llm.display());
                return;
            }

            println!("Cleanup worker: starting persistent subprocess...");
            let mut child = match Command::new(&test_llm)
                .arg(path.to_str().unwrap())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Cleanup worker: failed to spawn: {}", e);
                    return;
                }
            };

            let mut stdin = child.stdin.take().expect("stdin");
            let stdout = child.stdout.take().expect("stdout");
            let mut reader = BufReader::new(stdout);

            println!("Cleanup worker: persistent subprocess ready (model loaded once)");

            for req in rx {
                let prompt = build_prompt(&req.raw_text, &req.style, &req.level, req.custom_prompt.as_deref());
                let escaped = prompt.replace('\n', "\\n");

                if writeln!(stdin, "{}", escaped).is_err() {
                    let _ = req.reply.send(Ok(req.raw_text.clone()));
                    continue;
                }
                if stdin.flush().is_err() {
                    let _ = req.reply.send(Ok(req.raw_text.clone()));
                    continue;
                }

                let mut response = String::new();
                match reader.read_line(&mut response) {
                    Ok(0) | Err(_) => {
                        let _ = req.reply.send(Ok(req.raw_text.clone()));
                        continue;
                    }
                    Ok(_) => {}
                }

                let result = response.trim().replace("\\n", "\n");
                if result.is_empty() {
                    let _ = req.reply.send(Ok(req.raw_text.clone()));
                } else {
                    let _ = req.reply.send(Ok(result));
                }
            }

            let _ = child.kill();
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
