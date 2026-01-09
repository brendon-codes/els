#![allow(dead_code)]

use crate::types::{ColType, FileRow, FileType};

pub const ANSI_RED: &str = "\x1b[31m";
pub const ANSI_MAGENTA: &str = "\x1b[35m";
pub const ANSI_LIGHT_MAGENTA: &str = "\x1b[95m";
pub const ANSI_GREEN: &str = "\x1b[32m";
pub const ANSI_LIGHT_GREEN: &str = "\x1b[92m";
pub const ANSI_LIGHT_CYAN: &str = "\x1b[96m";
pub const ANSI_LIGHT_YELLOW: &str = "\x1b[93m";
pub const ANSI_LIGHT_GRAY: &str = "\x1b[37m";
pub const ANSI_DARK_GRAY: &str = "\x1b[30m";
pub const ANSI_LIGHT_RED: &str = "\x1b[91m";
pub const ANSI_LIGHT_BLUE: &str = "\x1b[94m";
pub const ANSI_BLUE: &str = "\x1b[34m";
pub const ANSI_END: &str = "\x1b[0m";

pub fn add_color(text: &str, color_code: &str) -> String {
    format!("{}{}{}", color_code, text, ANSI_END)
}

pub fn get_color_for_field(row: &FileRow, field: ColType) -> &'static str {
    match field {
        ColType::TargetName => ANSI_LIGHT_CYAN,
        ColType::SrcName => {
            if row.info.ftype == FileType::Directory {
                ANSI_LIGHT_RED
            } else {
                ANSI_LIGHT_GREEN
            }
        }
        ColType::TimeIso => ANSI_BLUE,
        ColType::Size => {
            if row.info.ftype == FileType::Directory {
                ANSI_MAGENTA
            } else {
                ANSI_GREEN
            }
        }
        ColType::Acls => ANSI_DARK_GRAY,
        ColType::Owner => ANSI_DARK_GRAY,
        ColType::FileType => ANSI_DARK_GRAY,
        ColType::Preview => ANSI_DARK_GRAY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContentType, FileRowInfo, RenderedCols, StatResult};

    fn make_test_row(ftype: FileType) -> FileRow {
        let info = FileRowInfo {
            fname: String::from("test"),
            ftype,
            stat_res: StatResult {
                st_mode: 0o644,
                st_mtime: 1704067200,
                st_uid: 1000,
                st_gid: 1000,
                st_size: 1024,
            },
            content_type: ContentType::Text,
            time_epoch: String::from("1704067200"),
        };
        FileRow {
            info,
            render: RenderedCols::default(),
        }
    }

    #[test]
    fn test_add_color_basic() {
        let result = add_color("hello", ANSI_RED);
        assert_eq!(result, "\x1b[31mhello\x1b[0m");
    }

    #[test]
    fn test_add_color_empty() {
        let result = add_color("", ANSI_GREEN);
        assert_eq!(result, "\x1b[32m\x1b[0m");
    }

    #[test]
    fn test_add_color_contains_reset() {
        let result = add_color("test", ANSI_BLUE);
        assert!(result.ends_with(ANSI_END));
    }

    #[test]
    fn test_get_color_targetname() {
        let row = make_test_row(FileType::File);
        assert_eq!(get_color_for_field(&row, ColType::TargetName), ANSI_LIGHT_CYAN);
    }

    #[test]
    fn test_get_color_srcname_directory() {
        let row = make_test_row(FileType::Directory);
        assert_eq!(get_color_for_field(&row, ColType::SrcName), ANSI_LIGHT_RED);
    }

    #[test]
    fn test_get_color_srcname_file() {
        let row = make_test_row(FileType::File);
        assert_eq!(get_color_for_field(&row, ColType::SrcName), ANSI_LIGHT_GREEN);
    }

    #[test]
    fn test_get_color_size_directory() {
        let row = make_test_row(FileType::Directory);
        assert_eq!(get_color_for_field(&row, ColType::Size), ANSI_MAGENTA);
    }

    #[test]
    fn test_get_color_size_file() {
        let row = make_test_row(FileType::File);
        assert_eq!(get_color_for_field(&row, ColType::Size), ANSI_GREEN);
    }

    #[test]
    fn test_get_color_timeiso() {
        let row = make_test_row(FileType::File);
        assert_eq!(get_color_for_field(&row, ColType::TimeIso), ANSI_BLUE);
    }

    #[test]
    fn test_get_color_acls() {
        let row = make_test_row(FileType::File);
        assert_eq!(get_color_for_field(&row, ColType::Acls), ANSI_DARK_GRAY);
    }

    #[test]
    fn test_get_color_owner() {
        let row = make_test_row(FileType::File);
        assert_eq!(get_color_for_field(&row, ColType::Owner), ANSI_DARK_GRAY);
    }

    #[test]
    fn test_get_color_filetype() {
        let row = make_test_row(FileType::File);
        assert_eq!(get_color_for_field(&row, ColType::FileType), ANSI_DARK_GRAY);
    }

    #[test]
    fn test_get_color_preview() {
        let row = make_test_row(FileType::File);
        assert_eq!(get_color_for_field(&row, ColType::Preview), ANSI_DARK_GRAY);
    }
}
