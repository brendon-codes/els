use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use crate::utils::{collapse_whitespace, is_printable_ascii};

const PREVIEW_READ_LEN: usize = 256;
const PREVIEW_TRUNC_LEN: usize = 20;
const DIR_PREVIEW_MAX_FILES: usize = 32;
const DIR_PREVIEW_TRUNC_LEN: usize = 20;

pub fn preview_directory(fname: &str) -> String {
    let path = Path::new(fname);

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return String::from("-"),
    };

    let all_files: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let entry_path = e.path();
            let real_path = if entry_path.is_symlink() {
                fs::canonicalize(&entry_path).unwrap_or(entry_path.clone())
            } else {
                entry_path.clone()
            };

            if real_path.is_dir() {
                format!("{}/", name)
            } else {
                name
            }
        })
        .collect();

    let all_len = all_files.len();
    let sub_files: Vec<&String> = all_files.iter().take(DIR_PREVIEW_MAX_FILES).collect();
    let sub_len = sub_files.len();

    let txt: String = sub_files.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(" ");
    let truncated: String = txt.chars().take(DIR_PREVIEW_TRUNC_LEN).collect();

    let lastindex = truncated.rfind(' ');
    let cleaned = match lastindex {
        Some(idx) if idx > 0 => truncated[..idx].to_string(),
        _ => truncated,
    };

    if sub_len < all_len {
        format!("{} ...", cleaned)
    } else {
        cleaned
    }
}

pub fn preview_binary(fname: &str) -> String {
    let path = Path::new(fname);

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return String::from(" "),
    };

    let mut buffer = vec![0u8; PREVIEW_READ_LEN];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return String::from(" "),
    };

    if bytes_read == 0 {
        return String::from(" ");
    }

    let printable: String = buffer[..bytes_read]
        .iter()
        .filter(|&&b| is_printable_ascii(b))
        .map(|&b| b as char)
        .collect();

    let cleaned = collapse_whitespace(&printable);
    cleaned.chars().take(PREVIEW_TRUNC_LEN).collect()
}

pub fn preview_text(fname: &str) -> String {
    let path = Path::new(fname);

    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => {
            let mut file = match File::open(path) {
                Ok(f) => f,
                Err(_) => return String::from(" "),
            };
            let mut buffer = vec![0u8; PREVIEW_READ_LEN];
            let bytes_read = match file.read(&mut buffer) {
                Ok(n) => n,
                Err(_) => return String::from(" "),
            };
            String::from_utf8_lossy(&buffer[..bytes_read]).to_string()
        }
    };

    if content.is_empty() {
        return String::from(" ");
    }

    let truncated: String = content.chars().take(PREVIEW_READ_LEN).collect();
    let cleaned = collapse_whitespace(&truncated);
    cleaned.chars().take(PREVIEW_TRUNC_LEN).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;

    #[test]
    fn test_preview_directory_empty() {
        let dir = TempDir::new().unwrap();
        let result = preview_directory(dir.path().to_str().unwrap());
        assert_eq!(result, "");
    }

    #[test]
    fn test_preview_directory_with_files() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("a.txt")).unwrap();
        File::create(dir.path().join("b.txt")).unwrap();

        let result = preview_directory(dir.path().to_str().unwrap());
        assert!(!result.is_empty());
    }

    #[test]
    fn test_preview_directory_with_subdir() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();

        let result = preview_directory(dir.path().to_str().unwrap());
        assert!(result.contains('/'));
    }

    #[test]
    fn test_preview_directory_nonexistent() {
        let result = preview_directory("/nonexistent/path/12345");
        assert_eq!(result, "-");
    }

    #[test]
    fn test_preview_binary_empty() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("empty.bin");
        File::create(&file_path).unwrap();

        let result = preview_binary(file_path.to_str().unwrap());
        assert_eq!(result, " ");
    }

    #[test]
    fn test_preview_binary_with_printable() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.bin");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"\x00\x01hello\x00world\x00").unwrap();

        let result = preview_binary(file_path.to_str().unwrap());
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_preview_binary_nonexistent() {
        let result = preview_binary("/nonexistent/path/12345.bin");
        assert_eq!(result, " ");
    }

    #[test]
    fn test_preview_text_short() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("short.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello").unwrap();

        let result = preview_text(file_path.to_str().unwrap());
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_preview_text_truncates() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("long.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "This is a very long line that should be truncated").unwrap();

        let result = preview_text(file_path.to_str().unwrap());
        assert!(result.len() <= PREVIEW_TRUNC_LEN);
    }

    #[test]
    fn test_preview_text_collapses_whitespace() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("spaces.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "a   b   c").unwrap();

        let result = preview_text(file_path.to_str().unwrap());
        assert_eq!(result, "a b c");
    }

    #[test]
    fn test_preview_text_empty() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("empty.txt");
        File::create(&file_path).unwrap();

        let result = preview_text(file_path.to_str().unwrap());
        assert_eq!(result, " ");
    }

    #[test]
    fn test_preview_text_nonexistent() {
        let result = preview_text("/nonexistent/path/12345.txt");
        assert_eq!(result, " ");
    }
}
