use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;
use std::num::NonZeroU32;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: test_llm <model_path> <prompt>");
        std::process::exit(1);
    }

    let model_path = &args[1];
    let prompt = &args[2];

    let backend = LlamaBackend::init().expect("backend init");
    let model_params = LlamaModelParams::default().with_n_gpu_layers(0);
    let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
        .expect("model load");

    let tokens = model.str_to_token(prompt, AddBos::Always).expect("tokenize");
    let n_ctx = std::cmp::max(tokens.len() as u32 + 256, 1024);
    let ctx_params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(n_ctx));
    let mut ctx = model.new_context(&backend, ctx_params).expect("context");

    let mut batch = LlamaBatch::new(n_ctx as usize, 1);
    for (i, token) in tokens.iter().enumerate() {
        batch.add(*token, i as i32, &[0], i == tokens.len() - 1).expect("batch");
    }
    ctx.decode(&mut batch).expect("decode");

    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::temp(0.1),
        LlamaSampler::top_p(0.95, 1),
        LlamaSampler::dist(42),
    ]);

    let mut output = String::new();
    let mut n_cur = tokens.len() as i32;

    for _ in 0..256 {
        let token = sampler.sample(&ctx, -1);
        if model.is_eog_token(token) {
            break;
        }

        #[allow(deprecated)]
        match model.token_to_str(token, Special::Tokenize) {
            Ok(piece) => output.push_str(&piece),
            Err(_) => break,
        }

        if output.contains("<|im_end|>") || output.contains("<|endoftext|>") {
            output = output.replace("<|im_end|>", "").replace("<|endoftext|>", "");
            break;
        }

        batch.clear();
        batch.add(token, n_cur, &[0], true).expect("batch");
        n_cur += 1;
        ctx.decode(&mut batch).expect("decode");
    }

    print!("{}", output.trim());
}
