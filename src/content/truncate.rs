/// Truncate content by keeping the first `max_chars` characters at a line boundary.
/// Returns (truncated_content, omitted_line_count).
pub fn truncate_head(content: &str, max_chars: usize) -> (String, usize) {
    if content.len() <= max_chars {
        return (content.to_string(), 0);
    }

    let total_lines = content.lines().count();
    let mut result = String::new();
    let mut kept_lines = 0;

    for line in content.lines() {
        let candidate = if result.is_empty() {
            line.len()
        } else {
            result.len() + 1 + line.len() // +1 for newline
        };
        if candidate > max_chars && !result.is_empty() {
            break;
        }
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(line);
        kept_lines += 1;
    }

    let omitted = total_lines.saturating_sub(kept_lines);
    (result, omitted)
}

/// Truncate content by keeping only low-indentation lines.
/// Lines whose leading whitespace exceeds `max_indent` are collapsed into
/// `... (N lines)` markers. `max_indent` is reduced until the result fits
/// within `max_chars`.
/// Returns (truncated_content, total_omitted_line_count).
pub fn truncate_nest(content: &str, max_chars: usize) -> (String, usize) {
    if content.len() <= max_chars {
        return (content.to_string(), 0);
    }

    let lines: Vec<&str> = content.lines().collect();

    // Find the maximum indent present
    let max_existing_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| indent_level(l))
        .max()
        .unwrap_or(0);

    // Progressively lower the indent threshold until we fit
    for threshold in (0..=max_existing_indent).rev() {
        let (result, omitted) = collapse_at_indent(&lines, threshold);
        if result.len() <= max_chars {
            return (result, omitted);
        }
    }

    // Even with threshold 0, still too large — fall back to head truncation
    truncate_head(content, max_chars)
}

/// Allocate a character budget proportionally across files.
/// `file_sizes` contains (index, char_count) for each file.
/// Returns per-file budget in the same order.
pub fn allocate_budget(file_sizes: &[usize], total_budget: usize) -> Vec<usize> {
    let total_chars: usize = file_sizes.iter().sum();
    if total_chars == 0 || total_chars <= total_budget {
        return file_sizes.to_vec();
    }

    let mut budgets: Vec<usize> = file_sizes
        .iter()
        .map(|&size| ((size as f64 / total_chars as f64) * total_budget as f64).floor() as usize)
        .collect();

    // Distribute remaining budget from rounding
    let allocated: usize = budgets.iter().sum();
    let remaining = total_budget.saturating_sub(allocated);
    let len = budgets.len();
    for i in 0..remaining {
        budgets[i % len] += 1;
    }

    budgets
}

fn indent_level(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Collapse lines with indent > threshold into `... (N lines)` markers.
fn collapse_at_indent(lines: &[&str], threshold: usize) -> (String, usize) {
    let mut result = String::new();
    let mut total_omitted = 0;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Empty lines are always kept (they don't have meaningful indent)
        if line.trim().is_empty() || indent_level(line) <= threshold {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
            i += 1;
        } else {
            // Count consecutive lines that exceed threshold
            let start = i;
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && indent_level(lines[i]) > threshold
            {
                i += 1;
            }
            let count = i - start;
            total_omitted += count;
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&format!("... ({} lines)", count));
        }
    }

    (result, total_omitted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_head_no_truncation() {
        let content = "line1\nline2\nline3";
        let (result, omitted) = truncate_head(content, 100);
        assert_eq!(result, content);
        assert_eq!(omitted, 0);
    }

    #[test]
    fn test_truncate_head_truncates() {
        let content = "line1\nline2\nline3\nline4";
        // "line1\nline2" = 11 chars
        let (result, omitted) = truncate_head(content, 11);
        assert_eq!(result, "line1\nline2");
        assert_eq!(omitted, 2);
    }

    #[test]
    fn test_truncate_head_single_long_line() {
        let content = "a".repeat(100);
        let (result, omitted) = truncate_head(&content, 50);
        // First line is already 100 chars but it's the first line, so it's kept
        assert_eq!(result, content);
        assert_eq!(omitted, 0);
    }

    #[test]
    fn test_truncate_nest_no_truncation() {
        let content = "fn main() {\n    println!(\"hello\");\n}";
        let (result, omitted) = truncate_nest(content, 1000);
        assert_eq!(result, content);
        assert_eq!(omitted, 0);
    }

    #[test]
    fn test_truncate_nest_collapses_deep_indent() {
        let content = "fn main() {\n    if true {\n        deeply_nested();\n        more_nested();\n    }\n}";
        // With a tight budget, deeply indented lines should collapse
        let (result, omitted) = truncate_nest(content, 60);
        assert!(result.contains("... ("));
        assert!(omitted > 0);
    }

    #[test]
    fn test_allocate_budget_proportional() {
        let sizes = vec![100, 200, 300];
        let budgets = allocate_budget(&sizes, 60);
        // Proportional: 10, 20, 30
        assert_eq!(budgets.iter().sum::<usize>(), 60);
    }

    #[test]
    fn test_allocate_budget_no_truncation_needed() {
        let sizes = vec![10, 20, 30];
        let budgets = allocate_budget(&sizes, 100);
        assert_eq!(budgets, vec![10, 20, 30]);
    }

    #[test]
    fn test_allocate_budget_empty() {
        let sizes: Vec<usize> = vec![];
        let budgets = allocate_budget(&sizes, 100);
        assert!(budgets.is_empty());
    }

    #[test]
    fn test_collapse_at_indent() {
        let lines = vec!["fn main() {", "    let x = 1;", "    let y = 2;", "}"];
        let (result, omitted) = collapse_at_indent(&lines, 0);
        assert!(result.contains("fn main() {"));
        assert!(result.contains("... (2 lines)"));
        assert!(result.contains("}"));
        assert_eq!(omitted, 2);
    }
}
