/// Truncate content to the first `n` lines.
/// Returns (truncated_content, omitted_line_count).
pub fn truncate_head_lines(content: &str, n: usize) -> (String, usize) {
    let lines: Vec<&str> = content.lines().collect();
    if n >= lines.len() {
        return (content.to_string(), 0);
    }
    let kept = lines[..n].join("\n");
    let omitted = lines.len() - n;
    (kept, omitted)
}

/// Find the largest n such that taking the first n lines of each file
/// keeps total chars <= max_chars. Uses binary search.
pub fn find_head_n(file_contents: &[&str], max_chars: usize) -> usize {
    if file_contents.is_empty() {
        return 0;
    }

    let max_lines = file_contents
        .iter()
        .map(|c| c.lines().count())
        .max()
        .unwrap_or(0);

    // Check if n=max_lines already fits
    if total_chars_at_head_n(file_contents, max_lines) <= max_chars {
        return max_lines;
    }

    // Binary search: lo is always feasible, hi is always infeasible
    let mut lo: usize = 0;
    let mut hi: usize = max_lines;

    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        if total_chars_at_head_n(file_contents, mid) <= max_chars {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }

    lo
}

fn total_chars_at_head_n(file_contents: &[&str], n: usize) -> usize {
    file_contents
        .iter()
        .map(|content| {
            let (truncated, _) = truncate_head_lines(content, n);
            truncated.len()
        })
        .sum()
}

/// Collapse lines with indent > threshold into `... (N lines)` markers.
pub fn collapse_at_indent(lines: &[&str], threshold: usize) -> (String, usize) {
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

/// Find the largest indent threshold such that collapsing lines with
/// indent > threshold across all files keeps total chars <= max_chars.
/// Returns the threshold, or None if even threshold=0 doesn't fit
/// (in which case caller should fall back to head mode).
pub fn find_nest_threshold(file_contents: &[&str], max_chars: usize) -> Option<usize> {
    if file_contents.is_empty() {
        return Some(usize::MAX);
    }

    // Collect all lines per file
    let file_lines: Vec<Vec<&str>> = file_contents.iter().map(|c| c.lines().collect()).collect();

    // Find the max indent across all files
    let max_indent = file_lines
        .iter()
        .flat_map(|lines| lines.iter())
        .filter(|l| !l.trim().is_empty())
        .map(|l| indent_level(l))
        .max()
        .unwrap_or(0);

    // Try thresholds from high to low (high = less collapsing)
    for threshold in (0..=max_indent).rev() {
        let total: usize = file_lines
            .iter()
            .map(|lines| collapse_at_indent(lines, threshold).0.len())
            .sum();
        if total <= max_chars {
            return Some(threshold);
        }
    }

    // Even threshold=0 doesn't fit
    None
}

fn indent_level(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_head_lines_no_truncation() {
        let content = "line1\nline2\nline3";
        let (result, omitted) = truncate_head_lines(content, 10);
        assert_eq!(result, content);
        assert_eq!(omitted, 0);
    }

    #[test]
    fn test_truncate_head_lines_truncates() {
        let content = "line1\nline2\nline3\nline4";
        let (result, omitted) = truncate_head_lines(content, 2);
        assert_eq!(result, "line1\nline2");
        assert_eq!(omitted, 2);
    }

    #[test]
    fn test_truncate_head_lines_zero() {
        let content = "line1\nline2";
        let (result, omitted) = truncate_head_lines(content, 0);
        assert_eq!(result, "");
        assert_eq!(omitted, 2);
    }

    #[test]
    fn test_find_head_n_all_fit() {
        let files = vec!["aaa\nbbb", "ccc"];
        let n = find_head_n(&files, 1000);
        assert_eq!(n, 2); // max lines across files
    }

    #[test]
    fn test_find_head_n_needs_truncation() {
        // file1: "aaaa\nbbbb\ncccc" (each line 4 chars)
        // file2: "dddd\neeee\nffff"
        // n=3 => 14+14=28, n=2 => 9+9=18, n=1 => 4+4=8
        let files = vec!["aaaa\nbbbb\ncccc", "dddd\neeee\nffff"];
        let n = find_head_n(&files, 20);
        assert_eq!(n, 2); // n=2 => 18 <= 20, n=3 => 28 > 20
    }

    #[test]
    fn test_find_head_n_uniform() {
        // All files get the same n
        let files = vec!["a\nb\nc\nd", "e\nf\ng\nh"];
        let n = find_head_n(&files, 6);
        // n=2 => "a\nb"(3) + "e\nf"(3) = 6 <= 6
        assert_eq!(n, 2);

        // Verify both files are truncated to the same number of lines
        let (r1, o1) = truncate_head_lines(files[0], n);
        let (r2, o2) = truncate_head_lines(files[1], n);
        assert_eq!(r1, "a\nb");
        assert_eq!(r2, "e\nf");
        assert_eq!(o1, 2);
        assert_eq!(o2, 2);
    }

    #[test]
    fn test_find_head_n_empty() {
        let files: Vec<&str> = vec![];
        assert_eq!(find_head_n(&files, 100), 0);
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

    #[test]
    fn test_find_nest_threshold_all_fit() {
        let files = vec!["fn main() {\n    hello();\n}"];
        let threshold = find_nest_threshold(&files, 1000);
        // Max indent is 4, so threshold should be >= max indent (everything kept)
        assert!(threshold.is_some());
        assert!(threshold.unwrap() >= 4);
    }

    #[test]
    fn test_find_nest_threshold_needs_collapsing() {
        let file1 = "fn a() {\n    if x {\n        deep1();\n        deep2();\n    }\n}";
        let file2 = "fn b() {\n    if y {\n        deep3();\n        deep4();\n    }\n}";
        let files = vec![file1, file2];

        // Find threshold with tight budget
        let threshold = find_nest_threshold(&files, 80);
        assert!(threshold.is_some());
        let t = threshold.unwrap();

        // Both files should be collapsed at the same threshold
        let lines1: Vec<&str> = file1.lines().collect();
        let lines2: Vec<&str> = file2.lines().collect();
        let (_, o1) = collapse_at_indent(&lines1, t);
        let (_, o2) = collapse_at_indent(&lines2, t);
        // Same threshold means symmetric collapsing for symmetric files
        assert_eq!(o1, o2);
    }

    #[test]
    fn test_find_nest_threshold_fallback() {
        // Even with threshold=0, content is too large
        let big = "a".repeat(1000);
        let files = vec![big.as_str()];
        let threshold = find_nest_threshold(&files, 10);
        assert!(threshold.is_none());
    }
}
