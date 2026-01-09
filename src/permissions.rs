use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct UserGroupCache {
    users: HashMap<u32, String>,
    groups: HashMap<u32, String>,
}

impl UserGroupCache {
    pub fn new() -> Self {
        let users = parse_passwd();
        let groups = parse_group();
        Self { users, groups }
    }

    pub fn get_user_name(&self, uid: u32) -> String {
        self.users
            .get(&uid)
            .cloned()
            .unwrap_or_else(|| uid.to_string())
    }

    pub fn get_group_name(&self, gid: u32) -> String {
        self.groups
            .get(&gid)
            .cloned()
            .unwrap_or_else(|| gid.to_string())
    }
}

impl Default for UserGroupCache {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_passwd() -> HashMap<u32, String> {
    let mut map = HashMap::new();
    let file = match File::open("/etc/passwd") {
        Ok(f) => f,
        Err(_) => return map,
    };

    let reader = BufReader::new(file);
    for line in reader.lines().flatten() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            if let Ok(uid) = parts[2].parse::<u32>() {
                map.insert(uid, parts[0].to_string());
            }
        }
    }
    map
}

fn parse_group() -> HashMap<u32, String> {
    let mut map = HashMap::new();
    let file = match File::open("/etc/group") {
        Ok(f) => f,
        Err(_) => return map,
    };

    let reader = BufReader::new(file);
    for line in reader.lines().flatten() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            if let Ok(gid) = parts[2].parse::<u32>() {
                map.insert(gid, parts[0].to_string());
            }
        }
    }
    map
}

pub fn get_acls_all(mode: u32) -> String {
    let user = (mode >> 6) & 0o7;
    let group = (mode >> 3) & 0o7;
    let other = mode & 0o7;
    format!("{}{}{}", user, group, other)
}

pub fn get_acls_me(path: &Path) -> String {
    let mut val = 0u8;

    if File::open(path).is_ok() {
        val |= 4;
    }

    if OpenOptions::new().write(true).open(path).is_ok() {
        val |= 2;
    }

    if is_executable(path) {
        val |= 1;
    }

    val.to_string()
}

fn is_executable(path: &Path) -> bool {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };

    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();

    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    let file_uid = metadata.uid();
    let file_gid = metadata.gid();

    use std::os::unix::fs::MetadataExt;

    if uid == file_uid {
        return (mode & 0o100) != 0;
    }

    if gid == file_gid {
        return (mode & 0o010) != 0;
    }

    (mode & 0o001) != 0
}

pub fn col_acls(path: &Path, mode: u32) -> String {
    let all_acls = get_acls_all(mode);
    let me_acls = get_acls_me(path);
    format!("{} {}", all_acls, me_acls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_get_acls_all_755() {
        assert_eq!(get_acls_all(0o755), "755");
    }

    #[test]
    fn test_get_acls_all_644() {
        assert_eq!(get_acls_all(0o644), "644");
    }

    #[test]
    fn test_get_acls_all_777() {
        assert_eq!(get_acls_all(0o777), "777");
    }

    #[test]
    fn test_get_acls_all_000() {
        assert_eq!(get_acls_all(0o000), "000");
    }

    #[test]
    fn test_get_acls_all_with_file_type_bits() {
        assert_eq!(get_acls_all(0o100755), "755");
        assert_eq!(get_acls_all(0o40755), "755");
    }

    #[test]
    fn test_user_group_cache_creation() {
        let _cache = UserGroupCache::new();
    }

    #[test]
    fn test_user_group_cache_default() {
        let _cache = UserGroupCache::default();
    }

    #[test]
    fn test_get_user_name_unknown() {
        let cache = UserGroupCache::new();
        let name = cache.get_user_name(99999);
        assert_eq!(name, "99999");
    }

    #[test]
    fn test_get_group_name_unknown() {
        let cache = UserGroupCache::new();
        let name = cache.get_group_name(99999);
        assert_eq!(name, "99999");
    }

    #[test]
    fn test_get_acls_me_readable_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("readable.txt");
        File::create(&file_path).unwrap();
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o644)).unwrap();

        let result = get_acls_me(&file_path);
        assert!(result.contains('4') || result.contains('6') || result.contains('7'));
    }

    #[test]
    fn test_get_acls_me_writable_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("writable.txt");
        File::create(&file_path).unwrap();
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o666)).unwrap();

        let result = get_acls_me(&file_path);
        assert!(result.contains('6') || result.contains('7'));
    }

    #[test]
    fn test_col_acls_format() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o644)).unwrap();

        let result = col_acls(&file_path, 0o644);
        assert!(result.starts_with("644 "));
    }
}
