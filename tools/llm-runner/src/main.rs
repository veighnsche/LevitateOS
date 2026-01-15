//! FunctionGemma inference runner for LevitateOS
//!
//! Translates natural language queries into shell commands using FunctionGemma.

use anyhow::{Context, Result};
use clap::Parser;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;

#[derive(Parser)]
#[command(name = "llm-runner")]
#[command(about = "Translate natural language to shell commands using FunctionGemma")]
struct Args {
    /// Path to the GGUF model file
    #[arg(short, long)]
    model: String,

    /// Natural language prompt to translate
    #[arg(short, long)]
    prompt: String,

    /// Maximum tokens to generate
    #[arg(short = 'n', long, default_value = "128")]
    max_tokens: i32,
}

/// FunctionGemma prompt format
fn build_prompt(user_query: &str) -> String {
    // Standard Gemma format
    format!(
        "<start_of_turn>user\n{user_query}<end_of_turn>\n<start_of_turn>model\n"
    )
}

/// Extract the shell command from model output
fn extract_command(output: &str) -> Option<String> {
    // Clean up the output
    let cleaned = output
        .replace("<end_of_turn>", "")
        .replace("<start_function_call>", "")
        .replace("<end_function_call>", "")
        .trim()
        .to_string();

    // If output looks like a command (starts with common command patterns)
    let first_line = cleaned.lines().next().unwrap_or("").trim();

    if !first_line.is_empty() {
        // Return first line if it looks like a command
        return Some(first_line.to_string());
    }

    // Try to find function call format
    if let Some(start_idx) = cleaned.find("command:<escape>") {
        let after_start = &cleaned[start_idx + "command:<escape>".len()..];
        if let Some(end_idx) = after_start.find("<escape>") {
            return Some(after_start[..end_idx].to_string());
        }
    }

    if !cleaned.is_empty() {
        return Some(cleaned);
    }

    None
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize llama backend
    let backend = LlamaBackend::init().context("Failed to initialize llama backend")?;

    // Load model
    eprintln!("Loading model from {}...", args.model);
    let model_params = LlamaModelParams::default();
    let model = LlamaModel::load_from_file(&backend, &args.model, &model_params)
        .context("Failed to load model")?;

    // Create context with appropriate size
    let ctx_params = LlamaContextParams::default();
    let mut ctx = model
        .new_context(&backend, ctx_params)
        .context("Failed to create context")?;

    // Build and tokenize prompt
    let prompt = build_prompt(&args.prompt);
    let tokens = model
        .str_to_token(&prompt, AddBos::Always)
        .context("Failed to tokenize prompt")?;

    eprintln!("Prompt tokens: {}", tokens.len());

    // Create batch and add prompt tokens
    let mut batch = LlamaBatch::new(512, 1);
    for (i, token) in tokens.iter().enumerate() {
        let is_last = i == tokens.len() - 1;
        batch.add(*token, i as i32, &[0], is_last)?;
    }

    // Process prompt (prefill)
    ctx.decode(&mut batch).context("Failed to decode prompt")?;

    // Set up sampler for generation
    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::temp(0.7),
        LlamaSampler::top_k(40),
        LlamaSampler::top_p(0.95, 1),
        LlamaSampler::greedy(),
    ]);

    // Generate tokens
    let mut output = String::new();
    let mut n_cur = tokens.len() as i32;

    eprintln!("Generating...");

    for _ in 0..args.max_tokens {
        // Sample next token from last position
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);

        // Accept the token
        sampler.accept(token);

        // Check for end of generation
        if model.is_eog_token(token) {
            break;
        }

        // Decode token to string
        match model.token_to_str(token, Special::Tokenize) {
            Ok(token_str) => {
                output.push_str(&token_str);
            }
            Err(_) => {
                // Skip tokens that can't be decoded
                continue;
            }
        }

        // Stop at end markers
        if output.contains("<end_of_turn>") || output.contains("<end_function_call>") {
            break;
        }

        // Prepare next batch with the new token
        batch.clear();
        batch.add(token, n_cur, &[0], true)?;
        n_cur += 1;

        // Decode the new token
        ctx.decode(&mut batch).context("Failed to decode token")?;
    }

    eprintln!("Raw output: {}", output);

    // Extract and print the command
    if let Some(command) = extract_command(&output) {
        println!("{}", command);
    } else {
        eprintln!("Could not extract command from model output");
        std::process::exit(1);
    }

    Ok(())
}
