#![allow(dead_code)]

use std::fs;
use std::path::Path;

use crate::permissions::{col_acls, UserGroupCache};
use crate::preview::{preview_binary, preview_directory, preview_text};
use crate::types::{Align, ColDef, ColType, ContentType, FileRowInfo, FileType};
use crate::utils::{format_size_with_commas, format_timestamp, truncate_middle};

pub fn get_col_defs() -> Vec<ColDef> {
    vec![
        ColDef {
            name: ColType::Acls,
            align: Align::Left,
            only_full: true,
        },
        ColDef {
            name: ColType::Owner,
            align: Align::Left,
            only_full: true,
        },
        ColDef {
            name: ColType::FileType,
            align: Align::Left,
            only_full: true,
        },
        ColDef {
            name: ColType::Size,
            align: Align::Right,
            only_full: false,
        },
        ColDef {
            name: ColType::TimeIso,
            align: Align::Left,
            only_full: false,
        },
        ColDef {
            name: ColType::SrcName,
            align: Align::Left,
            only_full: false,
        },
        ColDef {
            name: ColType::TargetName,
            align: Align::Left,
            only_full: false,
        },
        ColDef {
            name: ColType::Preview,
            align: Align::Left,
            only_full: true,
        },
    ]
}

pub fn render_col_acls(info: &FileRowInfo) -> String {
    let path = Path::new(&info.fname);
    col_acls(path, info.stat_res.st_mode)
}

pub fn render_col_owner(info: &FileRowInfo, cache: &UserGroupCache) -> String {
    let user = cache.get_user_name(info.stat_res.st_uid);
    let group = cache.get_group_name(info.stat_res.st_gid);
    format!("{}:{}", user, group)
}

pub fn render_col_filetype(info: &FileRowInfo) -> String {
    match info.content_type {
        ContentType::Directory => String::from("d"),
        ContentType::BinaryExecutable => String::from("e"),
        ContentType::BinaryOther => String::from("b"),
        ContentType::Text => String::from("t"),
        _ => String::from("u"),
    }
}

pub fn render_col_size(info: &FileRowInfo) -> String {
    if info.ftype == FileType::Directory {
        get_subfile_count(&info.fname)
    } else {
        format_size_with_commas(info.stat_res.st_size)
    }
}

fn get_subfile_count(fname: &str) -> String {
    let path = Path::new(fname);

    let real_path = if path.is_symlink() {
        match fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => return String::from("-"),
        }
    } else {
        path.to_path_buf()
    };

    if !real_path.is_dir() {
        return String::from("-");
    }

    match fs::read_dir(&real_path) {
        Ok(entries) => entries.count().to_string(),
        Err(_) => String::from("-"),
    }
}

pub fn render_col_timeiso(info: &FileRowInfo) -> String {
    format_timestamp(info.stat_res.st_mtime)
}

pub fn render_col_srcname(info: &FileRowInfo) -> String {
    let path = Path::new(&info.fname);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| info.fname.clone());

    if info.ftype == FileType::Directory {
        format!("{}/", name)
    } else {
        name
    }
}

pub fn render_col_targetname(info: &FileRowInfo) -> String {
    let path = Path::new(&info.fname);

    if !path.is_symlink() {
        return String::from(" ");
    }

    let real_path = match fs::read_link(path) {
        Ok(p) => p,
        Err(_) => return String::from(" "),
    };

    let target = real_path.to_string_lossy().to_string();

    let full = if info.ftype == FileType::Directory {
        format!("{}/", target)
    } else {
        target
    };

    truncate_middle(&full, 25)
}

pub fn render_col_preview(info: &FileRowInfo) -> String {
    match info.content_type {
        ContentType::Directory => preview_directory(&info.fname),
        ContentType::BinaryOther => preview_binary(&info.fname),
        ContentType::Text => preview_text(&info.fname),
        _ => String::from(" "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::StatResult;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    fn make_test_info(fname: &str, ftype: FileType, content_type: ContentType) -> FileRowInfo {
        FileRowInfo {
            fname: String::from(fname),
            ftype,
            stat_res: StatResult {
                st_mode: 0o644,
                st_mtime: 1704067200,
                st_uid: 1000,
                st_gid: 1000,
                st_size: 1024,
            },
            content_type,
            time_epoch: String::from("1704067200"),
        }
    }

    #[test]
    fn test_get_col_defs_count() {
        let defs = get_col_defs();
        assert_eq!(defs.len(), 8);
    }

    #[test]
    fn test_get_col_defs_size_align() {
        let defs = get_col_defs();
        let size_def = defs.iter().find(|d| d.name == ColType::Size).unwrap();
        assert_eq!(size_def.align, Align::Right);
    }

    #[test]
    fn test_get_col_defs_only_full() {
        let defs = get_col_defs();
        let acls_def = defs.iter().find(|d| d.name == ColType::Acls).unwrap();
        let preview_def = defs.iter().find(|d| d.name == ColType::Preview).unwrap();
        let size_def = defs.iter().find(|d| d.name == ColType::Size).unwrap();

        assert!(acls_def.only_full);
        assert!(preview_def.only_full);
        assert!(!size_def.only_full);
    }

    #[test]
    fn test_render_col_filetype_directory() {
        let info = make_test_info("dir", FileType::Directory, ContentType::Directory);
        assert_eq!(render_col_filetype(&info), "d");
    }

    #[test]
    fn test_render_col_filetype_executable() {
        let info = make_test_info("exe", FileType::File, ContentType::BinaryExecutable);
        assert_eq!(render_col_filetype(&info), "e");
    }

    #[test]
    fn test_render_col_filetype_binary() {
        let info = make_test_info("bin", FileType::File, ContentType::BinaryOther);
        assert_eq!(render_col_filetype(&info), "b");
    }

    #[test]
    fn test_render_col_filetype_text() {
        let info = make_test_info("txt", FileType::File, ContentType::Text);
        assert_eq!(render_col_filetype(&info), "t");
    }

    #[test]
    fn test_render_col_filetype_unknown() {
        let info = make_test_info("unk", FileType::File, ContentType::Unknown);
        assert_eq!(render_col_filetype(&info), "u");
    }

    #[test]
    fn test_render_col_size_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, World!").unwrap();

        let mut info = make_test_info(file_path.to_str().unwrap(), FileType::File, ContentType::Text);
        info.stat_res.st_size = 14;
        let result = render_col_size(&info);
        assert_eq!(result, "14");
    }

    #[test]
    fn test_render_col_size_directory() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("file1.txt")).unwrap();
        File::create(dir.path().join("file2.txt")).unwrap();

        let info = make_test_info(dir.path().to_str().unwrap(), FileType::Directory, ContentType::Directory);
        let result = render_col_size(&info);
        assert_eq!(result, "2");
    }

    #[test]
    fn test_render_col_timeiso_format() {
        let info = make_test_info("test", FileType::File, ContentType::Text);
        let result = render_col_timeiso(&info);
        assert!(result.contains('-'));
        assert!(result.contains(':'));
    }

    #[test]
    fn test_render_col_srcname_file() {
        let info = make_test_info("/path/to/file.txt", FileType::File, ContentType::Text);
        let result = render_col_srcname(&info);
        assert_eq!(result, "file.txt");
    }

    #[test]
    fn test_render_col_srcname_directory() {
        let info = make_test_info("/path/to/dir", FileType::Directory, ContentType::Directory);
        let result = render_col_srcname(&info);
        assert_eq!(result, "dir/");
    }

    #[test]
    fn test_render_col_targetname_not_symlink() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("regular.txt");
        File::create(&file_path).unwrap();

        let info = make_test_info(file_path.to_str().unwrap(), FileType::File, ContentType::Text);
        let result = render_col_targetname(&info);
        assert_eq!(result, " ");
    }

    #[test]
    fn test_render_col_preview_unknown() {
        let info = make_test_info("test", FileType::File, ContentType::Unknown);
        let result = render_col_preview(&info);
        assert_eq!(result, " ");
    }

    #[test]
    fn test_render_col_owner_format() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let info = make_test_info(file_path.to_str().unwrap(), FileType::File, ContentType::Text);
        let cache = UserGroupCache::new();
        let result = render_col_owner(&info, &cache);
        assert!(result.contains(':'));
    }
}
