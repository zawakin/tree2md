use crate::cli::Args;
use crate::language::{detect_lang, to_comment};
use crate::util::format::{format_size, generate_truncation_message, TruncateType, TruncationInfo};
use std::fs;
use std::io::{self, BufRead, Read, Seek};
use std::path::Path;

const PROBE_BYTES: usize = 8192;

pub struct ReadPayload {
    pub content: String,
    pub info: TruncationInfo,
}

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Binary(u64),
    NonUtf8,
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::Io(e) => write!(f, "{}", e),
            ReadError::Binary(sz) => write!(f, "Binary file ({})", format_size(*sz)),
            ReadError::NonUtf8 => write!(f, "File is not valid UTF-8 text"),
        }
    }
}
impl std::error::Error for ReadError {}
impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> Self {
        ReadError::Io(e)
    }
}

pub fn load_file_content_with_limits(
    path: &Path,
    truncate_bytes: Option<usize>,
    max_lines: Option<usize>,
) -> Result<ReadPayload, ReadError> {
    let mut file = fs::File::open(path).map_err(ReadError::Io)?;

    let meta_len = file.metadata().ok().map(|m| m.len()).unwrap_or(0);
    let mut probe = vec![0u8; PROBE_BYTES.min(meta_len as usize)];
    let n = file.read(&mut probe).map_err(ReadError::Io)?;
    probe.truncate(n);

    let is_binary = probe
        .iter()
        .any(|&b| b == 0 || (b < 32 && b != 9 && b != 10 && b != 13));

    if is_binary {
        return Err(ReadError::Binary(meta_len));
    }

    file.seek(std::io::SeekFrom::Start(0))
        .map_err(ReadError::Io)?;
    let mut rdr = io::BufReader::new(file);

    let total_bytes = meta_len as usize;
    let mut total_lines: usize = 0;

    let mut result_bytes: Vec<u8> = Vec::new();
    let mut buf: Vec<u8> = Vec::with_capacity(4096);

    let mut shown_lines = 0usize;
    let mut shown_bytes = 0usize;
    let mut truncated = false;
    let mut truncate_type = TruncateType::None;

    loop {
        buf.clear();
        let read_n = rdr.read_until(b'\n', &mut buf).map_err(ReadError::Io)?;
        if read_n == 0 {
            break;
        }

        total_lines = total_lines.saturating_add(1);

        if truncated {
            // Once truncated, just count remaining lines without processing
            if let Some(max_l) = max_lines {
                if total_lines >= max_l * 2 {
                    // Stop counting after reaching double the max lines
                    break;
                }
            }
            continue;
        }

        if let Some(max_b) = truncate_bytes {
            if shown_bytes + buf.len() > max_b {
                let remaining = max_b.saturating_sub(shown_bytes);
                if remaining > 0 {
                    let mut cut = remaining.min(buf.len());
                    while cut > 0 && std::str::from_utf8(&buf[..cut]).is_err() {
                        cut -= 1;
                    }
                    if cut > 0 {
                        result_bytes.extend_from_slice(&buf[..cut]);
                        shown_bytes += cut;
                        // Count all newlines in the truncated portion
                        shown_lines += buf[..cut].iter().filter(|&&b| b == b'\n').count();
                    }
                }
                truncated = true;
                truncate_type = if max_lines.is_some() {
                    TruncateType::Both
                } else {
                    TruncateType::Bytes
                };
                continue;
            }
        }

        if let Some(max_l) = max_lines {
            if shown_lines >= max_l {
                truncated = true;
                truncate_type = if truncate_bytes.is_some() {
                    TruncateType::Both
                } else {
                    TruncateType::Lines
                };
                continue;
            }
        }

        result_bytes.extend_from_slice(&buf);
        shown_bytes += buf.len();
        shown_lines += 1;
    }

    let mut content = String::from_utf8(result_bytes).map_err(|_| ReadError::NonUtf8)?;

    if !content.ends_with('\n') {
        content.push('\n');
    }

    Ok(ReadPayload {
        content,
        info: TruncationInfo {
            truncated,
            total_lines,
            total_bytes,
            shown_lines,
            shown_bytes,
            truncate_type,
        },
    })
}

pub fn print_file_content_with_display(path: &Path, display_path: &Path, args: &Args) {
    match load_file_content_with_limits(path, args.truncate, args.max_lines) {
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
