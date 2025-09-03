use crate::cli::Args;
use crate::content::io;
use crate::language::{detect_lang, to_comment};
use crate::util::format::{format_size, generate_truncation_message, TruncationInfo};
use std::path::Path;

pub struct ReadPayload {
    pub content: String,
    pub info: TruncationInfo,
}

// Re-export ReadError from io module for backwards compatibility
pub use io::ReadError;

pub fn load_file_content_with_limits(
    path: &Path,
    truncate_bytes: Option<usize>,
    max_lines: Option<usize>,
) -> Result<ReadPayload, ReadError> {
    // Delegate to centralized I/O function
    let (content, info) = io::read_text_prefix(path, truncate_bytes, max_lines)?;
    Ok(ReadPayload { content, info })
}

pub fn print_file_content_with_display(path: &Path, display_path: &Path, _args: &Args) {
    // Content embedding is deprecated but still functional for backwards compatibility
    // Using None for truncate and max_lines since those fields are removed
    match load_file_content_with_limits(path, None, None) {
        Ok(payload) => {
            let file_name = path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new(""))
                .to_string_lossy();
            let lang = detect_lang(&file_name);
            let lang_name = lang.map(|l| l.name).unwrap_or("");

            println!("\n### {}", display_path.display());
            println!("```{}", lang_name);
            print!("{}", payload.content);

            if payload.info.truncated {
                let message = generate_truncation_message(&payload.info);
                if lang.map(|l| l.name == "json").unwrap_or(false) {
                    println!("```");
                    println!("*{}*", message);
                } else {
                    if let Some(l) = lang {
                        println!("{}", to_comment(l, &message));
                    } else {
                        println!("// {}", message);
                    }
                    println!("```");
                }
            } else {
                println!("```");
            }
        }
        Err(ReadError::Binary(sz)) => {
            println!("\n### {}", display_path.display());
            println!("*Binary file ({}).*", format_size(sz));
        }
        Err(e) => {
            println!("\n### {}", display_path.display());
            println!("*Failed to read content: {}*", e);
        }
    }
}
