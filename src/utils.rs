pub fn format_timestamp(secs: i64) -> String {
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let time_t = secs as libc::time_t;

    unsafe {
        libc::localtime_r(&time_t, &mut tm);
    }

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec
    )
}

pub fn format_size_with_commas(size: u64) -> String {
    let s = size.to_string();
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::with_capacity(s.len() + s.len() / 3);

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    result
}

pub fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_space = true;

    for c in s.chars() {
        if c.is_whitespace() || c.is_control() || (c as u32) < 0x20 {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(c);
            prev_was_space = false;
        }
    }

    result.trim().to_string()
}

pub fn is_printable_ascii(b: u8) -> bool {
    (0x21..=0x7E).contains(&b) || (0x09..=0x0D).contains(&b)
}

pub fn truncate_middle(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        return s.to_string();
    }

    let ellipsis = "...";
    let ellipsis_len = ellipsis.len();
    let available = max_len - ellipsis_len;
    let front_len = (available + 1) / 2;
    let back_len = available / 2;

    let front: String = s.chars().take(front_len).collect();
    let back: String = s.chars().skip(char_count - back_len).collect();

    format!("{}{}{}", front, ellipsis, back)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_with_commas_zero() {
        assert_eq!(format_size_with_commas(0), "0");
    }

    #[test]
    fn test_format_size_with_commas_single_digit() {
        assert_eq!(format_size_with_commas(5), "5");
    }

    #[test]
    fn test_format_size_with_commas_three_digits() {
        assert_eq!(format_size_with_commas(999), "999");
    }

    #[test]
    fn test_format_size_with_commas_four_digits() {
        assert_eq!(format_size_with_commas(1000), "1,000");
    }

    #[test]
    fn test_format_size_with_commas_six_digits() {
        assert_eq!(format_size_with_commas(999999), "999,999");
    }

    #[test]
    fn test_format_size_with_commas_seven_digits() {
        assert_eq!(format_size_with_commas(1234567), "1,234,567");
    }

    #[test]
    fn test_format_size_with_commas_large() {
        assert_eq!(format_size_with_commas(1234567890123), "1,234,567,890,123");
    }

    #[test]
    fn test_collapse_whitespace_empty() {
        assert_eq!(collapse_whitespace(""), "");
    }

    #[test]
    fn test_collapse_whitespace_no_change() {
        assert_eq!(collapse_whitespace("hello"), "hello");
    }

    #[test]
    fn test_collapse_whitespace_single_spaces() {
        assert_eq!(collapse_whitespace("a b c"), "a b c");
    }

    #[test]
    fn test_collapse_whitespace_multiple_spaces() {
        assert_eq!(collapse_whitespace("a  b   c"), "a b c");
    }

    #[test]
    fn test_collapse_whitespace_tabs() {
        assert_eq!(collapse_whitespace("a\tb\tc"), "a b c");
    }

    #[test]
    fn test_collapse_whitespace_newlines() {
        assert_eq!(collapse_whitespace("a\nb\nc"), "a b c");
    }

    #[test]
    fn test_collapse_whitespace_mixed() {
        assert_eq!(collapse_whitespace(" a \t b \n c "), "a b c");
    }

    #[test]
    fn test_collapse_whitespace_leading_trailing() {
        assert_eq!(collapse_whitespace("  hello  "), "hello");
    }

    #[test]
    fn test_is_printable_ascii_letter() {
        assert!(is_printable_ascii(b'A'));
        assert!(is_printable_ascii(b'z'));
    }

    #[test]
    fn test_is_printable_ascii_digit() {
        assert!(is_printable_ascii(b'0'));
        assert!(is_printable_ascii(b'9'));
    }

    #[test]
    fn test_is_printable_ascii_tab() {
        assert!(is_printable_ascii(b'\t'));
    }

    #[test]
    fn test_is_printable_ascii_newline() {
        assert!(is_printable_ascii(b'\n'));
    }

    #[test]
    fn test_is_printable_ascii_space_excluded() {
        assert!(!is_printable_ascii(b' '));
    }

    #[test]
    fn test_is_printable_ascii_null() {
        assert!(!is_printable_ascii(0x00));
    }

    #[test]
    fn test_is_printable_ascii_del() {
        assert!(!is_printable_ascii(0x7F));
    }

    #[test]
    fn test_is_printable_ascii_tilde() {
        assert!(is_printable_ascii(b'~'));
    }

    #[test]
    fn test_is_printable_ascii_bang() {
        assert!(is_printable_ascii(b'!'));
    }

    #[test]
    fn test_truncate_middle_no_truncation() {
        assert_eq!(truncate_middle("short", 10), "short");
    }

    #[test]
    fn test_truncate_middle_exact_length() {
        assert_eq!(truncate_middle("exact", 5), "exact");
    }

    #[test]
    fn test_truncate_middle_truncates() {
        assert_eq!(truncate_middle("abcdefghij", 7), "ab...ij");
    }

    #[test]
    fn test_truncate_middle_even_available() {
        assert_eq!(truncate_middle("abcdefgh", 8), "abcdefgh");
        assert_eq!(truncate_middle("abcdefghi", 8), "abc...hi");
    }

    #[test]
    fn test_format_timestamp_format() {
        let result = format_timestamp(1704067200);
        let parts: Vec<&str> = result.split(' ').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 10);
        assert_eq!(parts[1].len(), 8);
    }

    #[test]
    fn test_format_timestamp_contains_dashes_colons() {
        let result = format_timestamp(1704067200);
        assert!(result.contains('-'));
        assert!(result.contains(':'));
    }
}
