use crate::cli::Args;
use crate::language::{detect_lang, to_comment};
use crate::util::format::{format_size, generate_truncation_message, TruncateType, TruncationInfo};
use std::fs;
use std::io::{Read, Seek};
use std::path::Path;

/// Size of chunk to read for binary file detection (8KB)
const PROBE_BYTES: usize = 8192;

pub fn load_file_content_with_limits(
    path: &Path,
    truncate_bytes: Option<usize>,
    max_lines: Option<usize>,
) -> (String, TruncationInfo) {
    // First, try to detect if it's a binary file by reading first chunk
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            return (
                format!("Error reading file: {}", e),
                TruncationInfo {
                    truncated: false,
                    total_lines: 0,
                    total_bytes: 0,
                    shown_lines: 0,
                    shown_bytes: 0,
                    truncate_type: TruncateType::None,
                },
            );
        }
    };

    // Read first chunk to detect binary content
    let cap = PROBE_BYTES.min(file.metadata().map(|m| m.len() as usize).unwrap_or(PROBE_BYTES));
    let mut buffer = vec![0; cap];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(e) => {
            return (
                format!("Error reading file: {}", e),
                TruncationInfo {
                    truncated: false,
                    total_lines: 0,
                    total_bytes: 0,
                    shown_lines: 0,
                    shown_bytes: 0,
                    truncate_type: TruncateType::None,
                },
            );
        }
    };
    buffer.truncate(bytes_read);

    // Check for binary content (NULL bytes or other control characters)
    let is_binary = buffer
        .iter()
        .any(|&b| b == 0 || (b < 32 && b != 9 && b != 10 && b != 13));

    if is_binary {
        let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
        return (
            format!("Binary file ({})", format_size(file_size)),
            TruncationInfo {
                truncated: false,
                total_lines: 0,
                total_bytes: file_size as usize,
                shown_lines: 0,
                shown_bytes: 0,
                truncate_type: TruncateType::None,
            },
        );
    }

    // Reset file position and read as text
    file.seek(std::io::SeekFrom::Start(0)).ok();

    let mut full_content = String::new();
    if let Err(e) = file.read_to_string(&mut full_content) {
        return (
            format!("Error reading file as text: {}", e),
            TruncationInfo {
                truncated: false,
                total_lines: 0,
                total_bytes: 0,
                shown_lines: 0,
                shown_bytes: 0,
                truncate_type: TruncateType::None,
            },
        );
    }

    let total_bytes = full_content.len();
    let total_lines = full_content.lines().count();

    let mut truncated = false;
    let mut truncate_type = TruncateType::None;
    let mut result = String::new();
    let mut shown_lines = 0;
    let mut shown_bytes = 0;

    for line in full_content.lines() {
        // Check line limit
        if let Some(max) = max_lines {
            if shown_lines >= max {
                truncated = true;
                truncate_type = if truncate_bytes.is_some() {
                    TruncateType::Both
                } else {
                    TruncateType::Lines
                };
                break;
            }
        }

        let line_with_newline = format!("{}\n", line);
        let line_bytes = line_with_newline.len();

        // Check byte limit
        if let Some(max) = truncate_bytes {
            if shown_bytes + line_bytes > max {
                // Add partial line if there's room, respecting UTF-8 boundaries
                let remaining = max.saturating_sub(shown_bytes);
                if remaining > 0 {
                    // Find safe UTF-8 boundary within remaining bytes
                    let mut safe_cut = 0;
                    for (idx, _) in line_with_newline.char_indices() {
                        if idx <= remaining {
                            safe_cut = idx;
                        } else {
                            break;
                        }
                    }
                    if safe_cut > 0 {
                        result.push_str(&line_with_newline[..safe_cut]);
                        shown_bytes += safe_cut;
                    }
                }
                truncated = true;
                truncate_type = if max_lines.is_some() {
                    TruncateType::Both
                } else {
                    TruncateType::Bytes
                };
                break;
            }
        }

        result.push_str(&line_with_newline);
        shown_lines += 1;
        shown_bytes += line_bytes;
    }

    (
        result,
        TruncationInfo {
            truncated,
            total_lines,
            total_bytes,
            shown_lines,
            shown_bytes,
            truncate_type,
        },
    )
}

pub fn print_file_content_with_display(path: &Path, display_path: &Path, args: &Args) {
    let (content, truncation_info) =
        load_file_content_with_limits(path, args.truncate, args.max_lines);

    // Detect language
    let file_name = path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_string_lossy();
    let lang = detect_lang(&file_name);

    let lang_name = lang.map(|l| l.name).unwrap_or("");

    // Print markdown code block
    println!("\n### {}", display_path.display());
    println!("```{}", lang_name);

    print!("{}", content);

    // Ensure newline at end
    if !content.ends_with('\n') {
        println!();
    }

    if truncation_info.truncated {
        let message = generate_truncation_message(&truncation_info);
        // For JSON files, print truncation message outside code block to avoid invalid syntax
        if lang.map(|l| l.name == "json").unwrap_or(false) {
            println!("```");
            println!("*{}*", message);
        } else {
            // Print truncation message as a comment in the appropriate language
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