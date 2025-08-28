use glob::Pattern;
use std::io;

/// Compile glob patterns from strings
pub fn compile_patterns(pattern_strings: &[String]) -> io::Result<Vec<Pattern>> {
    let mut patterns = Vec::new();
    for pattern_str in pattern_strings {
        match Pattern::new(pattern_str) {
            Ok(pattern) => patterns.push(pattern),
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid glob pattern '{}': {}", pattern_str, e),
                ));
            }
        }
    }
    Ok(patterns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_patterns() {
        let patterns = compile_patterns(&[
            "*.rs".to_string(),
            "src/**/*.go".to_string(),
        ]).unwrap();
        assert_eq!(patterns.len(), 2);
    }

    #[test]
    fn test_invalid_pattern() {
        let result = compile_patterns(&["[".to_string()]);
        assert!(result.is_err());
    }
}