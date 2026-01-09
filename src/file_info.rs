use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use mimetype_detector::detect_file;

use crate::types::{ContentType, FileRowInfo, FileType, StatResult};

pub fn get_stat_result(path: &Path) -> Option<StatResult> {
    let metadata = fs::symlink_metadata(path).ok()?;
    Some(StatResult {
        st_mode: metadata.mode(),
        st_mtime: metadata.mtime(),
        st_uid: metadata.uid(),
        st_gid: metadata.gid(),
        st_size: metadata.size(),
    })
}

pub fn get_file_type(path: &Path) -> FileType {
    let real_path = if path.is_symlink() {
        fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    } else {
        path.to_path_buf()
    };

    if real_path.is_dir() {
        FileType::Directory
    } else {
        FileType::File
    }
}

pub fn get_content_type(path: &Path, stat_res: &StatResult) -> ContentType {
    if path.is_dir() {
        return ContentType::Directory;
    }

    if fs::File::open(path).is_err() {
        return ContentType::NotReadable;
    }

    if stat_res.st_size == 0 {
        return ContentType::Empty;
    }

    get_file_info_via_crate(path)
}

fn get_file_info_via_crate(path: &Path) -> ContentType {
    let mime = match detect_file(path) {
        Ok(m) => m,
        Err(_) => return ContentType::Unknown,
    };

    let kind = mime.kind();

    if kind.is_text() {
        return ContentType::Text;
    }

    if kind.is_executable() {
        return ContentType::BinaryExecutable;
    }

    ContentType::BinaryOther
}

pub fn get_row_info(fname: &str) -> Option<FileRowInfo> {
    let path = Path::new(fname);
    let stat_res = get_stat_result(path)?;
    let ftype = get_file_type(path);
    let content_type = get_content_type(path, &stat_res);
    let time_epoch = stat_res.st_mtime.to_string();

    Some(FileRowInfo {
        fname: fname.to_string(),
        ftype,
        stat_res,
        content_type,
        time_epoch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_get_stat_result_valid_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let result = get_stat_result(&file_path);
        assert!(result.is_some());
    }

    #[test]
    fn test_get_stat_result_nonexistent() {
        let path = Path::new("/nonexistent/path/12345.txt");
        let result = get_stat_result(path);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_stat_result_has_size() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, World!").unwrap();

        let result = get_stat_result(&file_path).unwrap();
        assert!(result.st_size > 0);
    }

    #[test]
    fn test_get_file_type_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let result = get_file_type(&file_path);
        assert_eq!(result, FileType::File);
    }

    #[test]
    fn test_get_file_type_directory() {
        let dir = TempDir::new().unwrap();
        let result = get_file_type(dir.path());
        assert_eq!(result, FileType::Directory);
    }

    #[test]
    fn test_get_content_type_directory() {
        let dir = TempDir::new().unwrap();
        let stat_res = get_stat_result(dir.path()).unwrap();
        let result = get_content_type(dir.path(), &stat_res);
        assert_eq!(result, ContentType::Directory);
    }

    #[test]
    fn test_get_content_type_empty_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("empty.txt");
        File::create(&file_path).unwrap();

        let stat_res = get_stat_result(&file_path).unwrap();
        let result = get_content_type(&file_path, &stat_res);
        assert_eq!(result, ContentType::Empty);
    }

    #[test]
    fn test_get_content_type_text_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("text.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, World!").unwrap();

        let stat_res = get_stat_result(&file_path).unwrap();
        let result = get_content_type(&file_path, &stat_res);
        assert_eq!(result, ContentType::Text);
    }

    #[test]
    fn test_get_row_info_valid_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello").unwrap();

        let result = get_row_info(file_path.to_str().unwrap());
        assert!(result.is_some());

        let info = result.unwrap();
        assert_eq!(info.ftype, FileType::File);
        assert!(info.fname.contains("test.txt"));
    }

    #[test]
    fn test_get_row_info_directory() {
        let dir = TempDir::new().unwrap();
        let result = get_row_info(dir.path().to_str().unwrap());
        assert!(result.is_some());

        let info = result.unwrap();
        assert_eq!(info.ftype, FileType::Directory);
        assert_eq!(info.content_type, ContentType::Directory);
    }

    #[test]
    fn test_get_row_info_nonexistent() {
        let result = get_row_info("/nonexistent/path/12345.txt");
        assert!(result.is_none());
    }
}
