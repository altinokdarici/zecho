use crate::settings::{CleanupLevel, WritingStyle};
use std::path::Path;
use std::sync::mpsc;

pub struct CleanupRequest {
    pub raw_text: String,
    pub style: WritingStyle,
    pub level: CleanupLevel,
    pub custom_prompt: Option<String>,
    pub model_id: String,
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

            

            for req in rx {
                let prompt = build_prompt(&req.raw_text, &req.style, &req.level, req.custom_prompt.as_deref(), &req.model_id);
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

                let result = response.trim()
                    .replace("\\n", "\n")
                    .replace("<end_of_turn>", "")
                    .replace("<start_of_turn>", "")
                    .replace("<|im_end|>", "")
                    .replace("<|endoftext|>", "")
                    .trim().to_string();
                let result = strip_model_artifacts(&result);
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
        model_id: &str,
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
                model_id: model_id.to_string(),
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

pub fn build_prompt(
    raw_text: &str,
    style: &WritingStyle,
    level: &CleanupLevel,
    custom_prompt: Option<&str>,
    model_id: &str,
) -> String {
    let style_instruction = match style {
        WritingStyle::Formal => "Capitalize properly, use full punctuation.",
        WritingStyle::Casual => "Capitalize normally, light punctuation.",
        WritingStyle::VeryCasual => "All lowercase, minimal punctuation.",
    };

    let level_rules = match level {
        CleanupLevel::None => "Do not change any words. Only fix capitalization and punctuation.",
        CleanupLevel::Light => "Remove filler words (um, uh, like, you know, basically, so). Keep all other words exactly as spoken.",
        CleanupLevel::Medium => "Remove filler words. Resolve self-corrections (keep the corrected version only).",
        CleanupLevel::High => "Remove filler words. Resolve self-corrections. You may tighten phrasing slightly but never change meaning.",
    };

    let examples = match level {
        CleanupLevel::Medium | CleanupLevel::High => "\n\nSelf-correction examples:\n\
            - \"I want red no I mean blue\" → \"I want blue\"\n\
            - \"he is bad. I mean good\" → \"he is good\"\n\
            - \"use Java actually no use Python\" → \"use Python\"\n\
            - \"make it bigger well actually smaller\" → \"make it smaller\"\n\
            - \"at 3 sorry 4 o'clock\" → \"at 4 o'clock\"",
        _ => "",
    };

    let custom = custom_prompt
        .map(|p| format!("\n{}", p))
        .unwrap_or_default();

    let system = format!(
        "You clean voice transcriptions. Rules:\n\
        1. {}\n\
        2. {}\n\
        3. Output ONLY the cleaned text{}{}\n",
        level_rules, style_instruction, examples, custom
    );

    if model_id.contains("gemma") {
        format!(
            "<start_of_turn>user\n{}\nDo not add quotes, markdown, or explanation.\n\nClean this:\n{}<end_of_turn>\n<start_of_turn>model\n",
            system, raw_text
        )
    } else {
        format!(
            "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
            system, raw_text
        )
    }
}

fn strip_model_artifacts(text: &str) -> String {
    let mut s = text.trim().to_string();

    // Strip wrapping quotes (double or single)
    if s.len() >= 2 {
        if (s.starts_with('"') && s.ends_with('"'))
            || (s.starts_with('\'') && s.ends_with('\''))
        {
            s = s[1..s.len() - 1].to_string();
        }
    }

    // Strip wrapping backticks
    if s.starts_with('`') && s.ends_with('`') && !s.contains('\n') {
        s = s[1..s.len() - 1].to_string();
    }

    // Strip markdown bold wrapping
    if s.starts_with("**") && s.ends_with("**") && s.len() > 4 {
        s = s[2..s.len() - 2].to_string();
    }

    // Strip common prefixes added by models
    let lower = s.to_lowercase();
    let prefixes = [
        "here is the cleaned text:\n",
        "here is the cleaned text: ",
        "here's the cleaned text:\n",
        "here's the cleaned text: ",
        "cleaned text:\n",
        "cleaned text: ",
        "cleaned version:\n",
        "cleaned version: ",
        "cleaned:\n",
        "cleaned: ",
        "clean:\n",
        "clean: ",
    ];
    for prefix in &prefixes {
        if lower.starts_with(prefix) {
            s = s[prefix.len()..].to_string();
            break;
        }
    }

    s.trim().to_string()
}
