use zecho_lib::cleanup::build_prompt;
use zecho_lib::settings::{CleanupLevel, WritingStyle};

#[test]
fn prompt_has_chatml_format() {
    let prompt = build_prompt("hello world", &WritingStyle::Casual, &CleanupLevel::Light, None);

    assert!(prompt.starts_with("<|im_start|>system\n"));
    assert!(prompt.contains("<|im_end|>"));
    assert!(prompt.contains("<|im_start|>user\n"));
    assert!(prompt.contains("<|im_start|>assistant\n"));
    assert!(prompt.ends_with("<|im_start|>assistant\n"));

    // User text appears inside the user block
    assert!(prompt.contains("<|im_start|>user\nhello world<|im_end|>"));
}

#[test]
fn medium_level_mentions_self_corrections() {
    let prompt = build_prompt("test", &WritingStyle::Casual, &CleanupLevel::Medium, None);
    assert!(
        prompt.contains("corrects themselves"),
        "Medium level should mention self-corrections"
    );
}

#[test]
fn high_level_mentions_self_corrections() {
    let prompt = build_prompt("test", &WritingStyle::Casual, &CleanupLevel::High, None);
    assert!(
        prompt.contains("self-corrections"),
        "High level should mention self-corrections"
    );
}

#[test]
fn light_level_instruction_focuses_on_filler() {
    let prompt = build_prompt("test", &WritingStyle::Casual, &CleanupLevel::Light, None);
    assert!(
        prompt.contains("ONLY remove filler words"),
        "Light level should focus on filler word removal"
    );
    assert!(
        !prompt.contains("corrects themselves"),
        "Light level instruction should not mention speaker corrections"
    );
}

#[test]
fn formal_style_has_full_punctuation() {
    let prompt = build_prompt("test", &WritingStyle::Formal, &CleanupLevel::Light, None);
    assert!(
        prompt.contains("capitalize properly") && prompt.contains("full punctuation"),
        "Formal style should require proper capitalization and full punctuation"
    );
}

#[test]
fn casual_style_has_light_punctuation() {
    let prompt = build_prompt("test", &WritingStyle::Casual, &CleanupLevel::Light, None);
    assert!(
        prompt.contains("capitalize normally") && prompt.contains("light punctuation"),
        "Casual style should use normal capitalization and light punctuation"
    );
}

#[test]
fn very_casual_style_is_lowercase() {
    let prompt = build_prompt("test", &WritingStyle::VeryCasual, &CleanupLevel::Light, None);
    assert!(
        prompt.contains("all lowercase") && prompt.contains("minimal punctuation"),
        "VeryCasual style should use all lowercase and minimal punctuation"
    );
}

#[test]
fn styles_produce_different_instructions() {
    let formal = build_prompt("x", &WritingStyle::Formal, &CleanupLevel::Light, None);
    let casual = build_prompt("x", &WritingStyle::Casual, &CleanupLevel::Light, None);
    let very_casual = build_prompt("x", &WritingStyle::VeryCasual, &CleanupLevel::Light, None);

    assert_ne!(formal, casual);
    assert_ne!(casual, very_casual);
    assert_ne!(formal, very_casual);
}

#[test]
fn custom_prompt_is_appended() {
    let prompt = build_prompt(
        "test input",
        &WritingStyle::Casual,
        &CleanupLevel::Light,
        Some("Always use Oxford commas"),
    );
    assert!(
        prompt.contains("Extra: Always use Oxford commas"),
        "Custom prompt should appear in the output"
    );
}

#[test]
fn no_custom_prompt_means_no_extra_section() {
    let prompt = build_prompt("test", &WritingStyle::Casual, &CleanupLevel::Light, None);
    assert!(
        !prompt.contains("Extra:"),
        "Without a custom prompt, 'Extra:' should not appear"
    );
}
