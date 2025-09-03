/// Format bytes into human-readable size
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Size badge for intuitive classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeBadge {
    Xs,  // ≤ 1 KB
    S,   // ≤ 8 KB
    M,   // ≤ 64 KB
    L,   // ≤ 512 KB
    Xl,  // ≤ 4 MB
    Xxl, // > 4 MB
}

impl std::fmt::Display for SizeBadge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SizeBadge::Xs => "XS",
            SizeBadge::S => "S",
            SizeBadge::M => "M",
            SizeBadge::L => "L",
            SizeBadge::Xl => "XL",
            SizeBadge::Xxl => "XXL",
        };
        write!(f, "{}", s)
    }
}

/// Size marking with badge and optional tick for position within class
pub struct SizeMark {
    pub badge: SizeBadge,
    pub human: String,      // "3.1 KB"
    pub tick: Option<char>, // ▁..▇ or None for XS
}

/// Get size tick mark only (no badge)
pub fn size_mark(bytes: u64) -> SizeMark {
    let human = format_size(bytes);

    // Files smaller than 1KB: no visual indicator to reduce noise
    if bytes <= 1_024 {
        return SizeMark {
            badge: SizeBadge::Xs, // Keep for compatibility but won't be displayed
            human,
            tick: None,
        };
    }

    // Use logarithmic scale for better visual distribution
    // Map file sizes from 1KB to 100MB on a log scale
    let min_log = (1_024_f64).ln();
    let max_log = (100_000_000_f64).ln(); // 100MB as practical max for visualization

    let current_log = (bytes as f64).ln();
    let position = ((current_log - min_log) / (max_log - min_log)).clamp(0.0, 1.0);

    // Map to tick marks (7 levels)
    let ticks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇'];
    let tick_index = (position * 6.0).round() as usize;

    // For backward compatibility, assign a badge (but we won't display it)
    let badge = match bytes {
        0..=1_024 => SizeBadge::Xs,
        1_025..=8_192 => SizeBadge::S,
        8_193..=65_536 => SizeBadge::M,
        65_537..=524_288 => SizeBadge::L,
        524_289..=4_194_304 => SizeBadge::Xl,
        _ => SizeBadge::Xxl,
    };

    SizeMark {
        badge,
        human,
        tick: Some(ticks[tick_index.min(6)]),
    }
}

/// Size category for LOC (simplified boundaries)
pub fn loc_category(loc: usize) -> &'static str {
    match loc {
        0..=10 => "XS",
        11..=120 => "S",
        121..=300 => "M",
        301..=600 => "L",
        601..=1000 => "XL",
        _ => "XXL",
    }
}

/// Generate a visual bar with directory-local normalization (slot-style)
pub fn loc_to_bar(loc: usize, max_loc_in_dir: usize, bar_width: usize) -> String {
    // Don't show bar for very small files (< 10 lines)
    if loc < 10 {
        return format!("[{}]", "·".repeat(bar_width));
    }

    // Cap at 1000 lines for normalization
    let capped_loc = loc.min(1000);
    let capped_max = max_loc_in_dir.min(1000).max(1);

    // Linear scale within the directory
    let norm = capped_loc as f64 / capped_max as f64;
    let filled_cells = (norm * bar_width as f64).ceil() as usize;

    // Ensure at least 1 block for non-zero values
    let num_filled = filled_cells.max(1).min(bar_width);
    let num_empty = bar_width - num_filled;

    // Build the bar with brackets
    format!("[{}{}]", "█".repeat(num_filled), "·".repeat(num_empty))
}

/// Format lines of code for display
pub fn format_loc_display(loc: usize) -> String {
    if loc >= 1000 {
        "1k+".to_string()
    } else {
        loc.to_string()
    }
}

/// Check if LOC is in global top percentile
pub fn is_global_outlier(loc: usize, threshold: usize) -> bool {
    loc >= threshold
}

/// Information about file truncation
#[derive(Debug)]
pub struct TruncationInfo {
    pub truncated: bool,
    pub total_lines: usize,
    pub total_bytes: usize,
    pub shown_lines: usize,
    pub shown_bytes: usize,
    pub truncate_type: TruncateType,
}

#[derive(Debug)]
pub enum TruncateType {
    None,
    Bytes,
    Lines,
    Both,
}

/// Generate a message describing how content was truncated
pub fn generate_truncation_message(info: &TruncationInfo) -> String {
    match info.truncate_type {
        TruncateType::Lines => {
            format!(
                "[Content truncated: showing first {} of {} lines]",
                info.shown_lines, info.total_lines
            )
        }
        TruncateType::Bytes => {
            format!(
                "[Content truncated: showing first {} of {} bytes]",
                info.shown_bytes, info.total_bytes
            )
        }
        TruncateType::Both => {
            format!(
                "[Content truncated: showing first {} of {} lines, {} of {} bytes]",
                info.shown_lines, info.total_lines, info.shown_bytes, info.total_bytes
            )
        }
        TruncateType::None => "[Content truncated]".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_loc_to_bar() {
        // Test XS files (< 10 lines) - should show empty bar
        assert_eq!(loc_to_bar(5, 100, 10), "[··········]");
        assert_eq!(loc_to_bar(0, 100, 10), "[··········]");

        // Test normal files with local normalization
        assert_eq!(loc_to_bar(100, 100, 10), "[██████████]"); // Max in dir gets full bar
        assert_eq!(loc_to_bar(50, 100, 10), "[█████·····]"); // Half gets half bar
        assert_eq!(loc_to_bar(25, 100, 10), "[███·······]"); // Quarter gets quarter bar

        // Test with capping at 1000 lines
        assert_eq!(loc_to_bar(2000, 2000, 10), "[██████████]"); // Both capped at 1000
        assert_eq!(loc_to_bar(500, 2000, 10), "[█████·····]"); // 500 vs capped 1000
    }

    #[test]
    fn test_loc_category() {
        assert_eq!(loc_category(5), "XS");
        assert_eq!(loc_category(50), "S");
        assert_eq!(loc_category(150), "M");
        assert_eq!(loc_category(400), "L");
        assert_eq!(loc_category(800), "XL");
        assert_eq!(loc_category(1500), "XXL");
    }

    #[test]
    fn test_format_loc_display() {
        assert_eq!(format_loc_display(100), "100");
        assert_eq!(format_loc_display(999), "999");
        assert_eq!(format_loc_display(1000), "1k+");
        assert_eq!(format_loc_display(5000), "1k+");
    }

    #[test]
    fn test_generate_truncation_message() {
        let info = TruncationInfo {
            truncated: true,
            total_lines: 100,
            total_bytes: 5000,
            shown_lines: 50,
            shown_bytes: 2500,
            truncate_type: TruncateType::Lines,
        };
        assert_eq!(
            generate_truncation_message(&info),
            "[Content truncated: showing first 50 of 100 lines]"
        );

        let info = TruncationInfo {
            truncated: true,
            total_lines: 100,
            total_bytes: 5000,
            shown_lines: 50,
            shown_bytes: 2500,
            truncate_type: TruncateType::Both,
        };
        assert_eq!(
            generate_truncation_message(&info),
            "[Content truncated: showing first 50 of 100 lines, 2500 of 5000 bytes]"
        );
    }
}
