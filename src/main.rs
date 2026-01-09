mod colors;
mod columns;
mod display;
mod file_info;
mod permissions;
mod preview;
mod render;
mod types;
mod utils;

use std::fs;
use std::path::Path;

use columns::{
    render_col_acls, render_col_filetype, render_col_owner, render_col_preview, render_col_size,
    render_col_srcname, render_col_targetname, render_col_timeiso,
};
use display::display;
use file_info::get_row_info;
use permissions::UserGroupCache;
use render::render_rows;
use types::{Args, FileRow, FileType, RenderedCols};

fn parse_args() -> Args {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print_help();
        std::process::exit(0);
    }

    let full = pargs.contains(["-f", "--full"]);

    let filter: Option<String> = match pargs.opt_value_from_str(["-g", "--filter"]) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let remaining = pargs.finish();
    let mut start_path = String::from("./");

    for arg in &remaining {
        let s = arg.to_string_lossy();
        if s.starts_with('-') {
            eprintln!("Unknown argument: {}", s);
            std::process::exit(1);
        }
        if start_path == "./" {
            start_path = s.into_owned();
        }
    }

    Args {
        start_path,
        filter,
        full,
    }
}

fn print_help() {
    println!("rust-ls - Enhanced directory listing utility");
    println!();
    println!("Usage: rust-ls [OPTIONS] [STARTPATH]");
    println!();
    println!("Arguments:");
    println!("  STARTPATH    Directory path to list (default: './')");
    println!();
    println!("Options:");
    println!("  -f, --full       Enable full output mode");
    println!("  -g, --filter     Filter results by substring");
    println!("  -h, --help       Show this help message");
}

fn get_dir_listing(start: &str, filter: Option<&str>) -> Option<Vec<String>> {
    let path = Path::new(start);

    if !path.exists() || !path.is_dir() {
        return None;
    }

    let entries = fs::read_dir(path).ok()?;

    let trimmed = start.trim_end_matches('/');
    let real_start = if trimmed.starts_with("./") && trimmed.len() > 2 {
        &trimmed[2..]
    } else if trimmed == "." || trimmed.is_empty() {
        ""
    } else {
        trimmed
    };

    let paths: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(f) = filter {
                if !name.to_lowercase().contains(&f.to_lowercase()) {
                    return None;
                }
            }
            if real_start.is_empty() {
                Some(name)
            } else {
                Some(format!("{}/{}", real_start, name))
            }
        })
        .collect();

    Some(paths)
}

fn build_row(fname: &str, cache: &UserGroupCache, full: bool) -> Option<FileRow> {
    let info = get_row_info(fname)?;

    let render = RenderedCols {
        acls: if full { render_col_acls(&info) } else { String::from(" ") },
        owner: if full { render_col_owner(&info, cache) } else { String::from(" ") },
        filetype: if full { render_col_filetype(&info) } else { String::from(" ") },
        size: render_col_size(&info),
        timeiso: render_col_timeiso(&info),
        srcname: render_col_srcname(&info),
        targetname: render_col_targetname(&info),
        preview: if full { render_col_preview(&info) } else { String::from(" ") },
    };

    Some(FileRow { info, render })
}

fn sort_rows(rows: &mut [FileRow]) {
    rows.sort_by(|a, b| {
        let a_is_dir = a.info.ftype == FileType::Directory;
        let b_is_dir = b.info.ftype == FileType::Directory;

        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let a_name = a.info.fname.to_lowercase();
                let b_name = b.info.fname.to_lowercase();
                a_name.cmp(&b_name)
            }
        }
    });
}

fn get_files(start: &str, full: bool, filter: Option<&str>) -> Option<Vec<FileRow>> {
    let paths = get_dir_listing(start, filter)?;
    let cache = UserGroupCache::new();

    let mut rows: Vec<FileRow> = paths
        .iter()
        .filter_map(|p| build_row(p, &cache, full))
        .collect();

    sort_rows(&mut rows);
    Some(rows)
}

fn render_error() {
    eprintln!("Path could not be found, or path is not a directory.");
}

fn run(start: &str, full: bool, filter: Option<&str>) -> bool {
    let files = match get_files(start, full, filter) {
        Some(f) => f,
        None => {
            render_error();
            return false;
        }
    };

    let rows = render_rows(&files, full);
    display(&rows);
    true
}

fn main() {
    let args = parse_args();
    let success = run(&args.start_path, args.full, args.filter.as_deref());

    if !success {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    fn make_test_row(fname: &str, ftype: FileType) -> FileRow {
        use types::{ContentType, FileRowInfo, StatResult};

        let info = FileRowInfo {
            fname: String::from(fname),
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
    fn test_sort_rows_dirs_first() {
        let mut rows = vec![
            make_test_row("file.txt", FileType::File),
            make_test_row("dir", FileType::Directory),
        ];
        sort_rows(&mut rows);
        assert_eq!(rows[0].info.ftype, FileType::Directory);
        assert_eq!(rows[1].info.ftype, FileType::File);
    }

    #[test]
    fn test_sort_rows_alphabetical() {
        let mut rows = vec![
            make_test_row("zebra", FileType::File),
            make_test_row("apple", FileType::File),
            make_test_row("mango", FileType::File),
        ];
        sort_rows(&mut rows);
        assert!(rows[0].info.fname.contains("apple"));
        assert!(rows[1].info.fname.contains("mango"));
        assert!(rows[2].info.fname.contains("zebra"));
    }

    #[test]
    fn test_sort_rows_case_insensitive() {
        let mut rows = vec![
            make_test_row("Zebra", FileType::File),
            make_test_row("apple", FileType::File),
        ];
        sort_rows(&mut rows);
        assert!(rows[0].info.fname.contains("apple"));
        assert!(rows[1].info.fname.contains("Zebra"));
    }

    #[test]
    fn test_sort_rows_dirs_alphabetical() {
        let mut rows = vec![
            make_test_row("zdir", FileType::Directory),
            make_test_row("adir", FileType::Directory),
            make_test_row("file", FileType::File),
        ];
        sort_rows(&mut rows);
        assert!(rows[0].info.fname.contains("adir"));
        assert!(rows[1].info.fname.contains("zdir"));
        assert!(rows[2].info.fname.contains("file"));
    }

    #[test]
    fn test_get_dir_listing_valid() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("file1.txt")).unwrap();
        File::create(dir.path().join("file2.txt")).unwrap();

        let result = get_dir_listing(dir.path().to_str().unwrap(), None);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_get_dir_listing_nonexistent() {
        let result = get_dir_listing("/nonexistent/path/12345", None);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_dir_listing_with_filter() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("test.txt")).unwrap();
        File::create(dir.path().join("other.log")).unwrap();

        let result = get_dir_listing(dir.path().to_str().unwrap(), Some("test"));
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_get_dir_listing_filter_case_insensitive() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("TEST.txt")).unwrap();
        File::create(dir.path().join("other.log")).unwrap();

        let result = get_dir_listing(dir.path().to_str().unwrap(), Some("test"));
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_get_files_valid() {
        let dir = TempDir::new().unwrap();
        let mut file = File::create(dir.path().join("test.txt")).unwrap();
        writeln!(file, "Hello").unwrap();

        let result = get_files(dir.path().to_str().unwrap(), false, None);
        assert!(result.is_some());
    }

    #[test]
    fn test_get_files_nonexistent() {
        let result = get_files("/nonexistent/path/12345", false, None);
        assert!(result.is_none());
    }
}
