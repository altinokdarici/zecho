use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;
use std::num::NonZeroU32;

fn main() {
    let model_path = format!(
        "{}/Library/Application Support/zecho/models/Qwen2.5-1.5B-Instruct-Q4_K_M.gguf",
        std::env::var("HOME").unwrap()
    );

    println!("=== Step 1: Init backend ===");
    let backend = LlamaBackend::init().expect("backend init failed");

    println!("=== Step 2: Load model (n_gpu_layers=0) ===");
    let model_params = LlamaModelParams::default().with_n_gpu_layers(0);
    let model = match LlamaModel::load_from_file(&backend, &model_path, &model_params) {
        Ok(m) => {
            println!("Model loaded OK");
            m
        }
        Err(e) => {
            println!("Model load FAILED: {:?}", e);
            return;
        }
    };

    println!("=== Step 3: Tokenize ===");
    let prompt = "<|im_start|>system\nClean up this text. Remove filler words. Return only cleaned text.<|im_end|>\n<|im_start|>user\nI want the background to be green uhhh actually purple<|im_end|>\n<|im_start|>assistant\n";
    let tokens = model.str_to_token(prompt, AddBos::Always).expect("tokenize failed");
    println!("Tokens: {}", tokens.len());

    println!("=== Step 4: Create context ===");
    let n_ctx = (tokens.len() as u32) + 256;
    let ctx_params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(n_ctx));
    let mut ctx = model.new_context(&backend, ctx_params).expect("context failed");

    println!("=== Step 5: Batch + decode prompt ===");
    let mut batch = LlamaBatch::new(n_ctx as usize, 1);
    for (i, token) in tokens.iter().enumerate() {
        batch.add(*token, i as i32, &[0], i == tokens.len() - 1).expect("batch add");
    }
    ctx.decode(&mut batch).expect("decode failed");

    println!("=== Step 6: Sample tokens ===");
    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::temp(0.1),
        LlamaSampler::top_p(0.95, 1),
        LlamaSampler::dist(42),
    ]);

    let mut output = String::new();
    let mut n_cur = tokens.len() as i32;

    for i in 0..128 {
        let token = sampler.sample(&ctx, -1);
        if model.is_eog_token(token) {
            println!("EOG at token {}", i);
            break;
        }

        #[allow(deprecated)]
        match model.token_to_str(token, Special::Tokenize) {
            Ok(piece) => {
                print!("{}", piece);
                output.push_str(&piece);
                if output.contains("<|im_end|>") {
                    break;
                }
            }
            Err(e) => {
                println!("\nDetokenize error at {}: {:?}", i, e);
                break;
            }
        }

        batch.clear();
        batch.add(token, n_cur, &[0], true).expect("batch add");
        n_cur += 1;
        ctx.decode(&mut batch).expect("decode failed");
    }

    let result = output.replace("<|im_end|>", "").trim().to_string();
    println!("\n\n=== RESULT: {:?} ===", result);
}
