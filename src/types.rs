#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Directory,
    NotReadable,
    Empty,
    BinaryExecutable,
    BinaryOther,
    Text,
    Other,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColType {
    Acls,
    Owner,
    FileType,
    Size,
    TimeIso,
    SrcName,
    TargetName,
    Preview,
}

#[derive(Debug, Clone)]
pub struct StatResult {
    pub st_mode: u32,
    pub st_mtime: i64,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_size: u64,
}

#[derive(Debug, Clone)]
pub struct FileRowInfo {
    pub fname: String,
    pub ftype: FileType,
    pub stat_res: StatResult,
    pub content_type: ContentType,
    pub time_epoch: String,
}

#[derive(Debug, Clone, Default)]
pub struct RenderedCols {
    pub acls: String,
    pub owner: String,
    pub filetype: String,
    pub size: String,
    pub timeiso: String,
    pub srcname: String,
    pub targetname: String,
    pub preview: String,
}

#[derive(Debug, Clone)]
pub struct FileRow {
    pub info: FileRowInfo,
    pub render: RenderedCols,
}

pub struct ColDef {
    pub name: ColType,
    pub align: Align,
    pub only_full: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ColPaddings {
    pub acls: usize,
    pub owner: usize,
    pub filetype: usize,
    pub size: usize,
    pub timeiso: usize,
    pub srcname: usize,
    pub targetname: usize,
    pub preview: usize,
}

#[derive(Debug, Clone)]
pub struct Args {
    pub start_path: String,
    pub filter: Option<String>,
    pub full: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            start_path: String::from("./"),
            filter: None,
            full: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_default_start_path() {
        let args = Args::default();
        assert_eq!(args.start_path, "./");
    }

    #[test]
    fn test_args_default_filter() {
        let args = Args::default();
        assert!(args.filter.is_none());
    }

    #[test]
    fn test_args_default_full() {
        let args = Args::default();
        assert!(!args.full);
    }

    #[test]
    fn test_filetype_eq() {
        assert_eq!(FileType::File, FileType::File);
        assert_eq!(FileType::Directory, FileType::Directory);
    }

    #[test]
    fn test_filetype_neq() {
        assert_ne!(FileType::File, FileType::Directory);
    }

    #[test]
    fn test_contenttype_variants() {
        assert_ne!(ContentType::Directory, ContentType::Text);
        assert_ne!(ContentType::BinaryExecutable, ContentType::BinaryOther);
        assert_ne!(ContentType::Empty, ContentType::NotReadable);
    }

    #[test]
    fn test_align_variants() {
        assert_ne!(Align::Left, Align::Right);
        assert_eq!(Align::Left, Align::Left);
    }

    #[test]
    fn test_coltype_variants() {
        assert_ne!(ColType::Acls, ColType::Owner);
        assert_ne!(ColType::Size, ColType::TimeIso);
        assert_eq!(ColType::SrcName, ColType::SrcName);
    }

    #[test]
    fn test_colpaddings_default() {
        let paddings = ColPaddings::default();
        assert_eq!(paddings.acls, 0);
        assert_eq!(paddings.owner, 0);
        assert_eq!(paddings.filetype, 0);
        assert_eq!(paddings.size, 0);
        assert_eq!(paddings.timeiso, 0);
        assert_eq!(paddings.srcname, 0);
        assert_eq!(paddings.targetname, 0);
        assert_eq!(paddings.preview, 0);
    }

    #[test]
    fn test_renderedcols_default() {
        let cols = RenderedCols::default();
        assert_eq!(cols.acls, "");
        assert_eq!(cols.owner, "");
        assert_eq!(cols.filetype, "");
        assert_eq!(cols.size, "");
        assert_eq!(cols.timeiso, "");
        assert_eq!(cols.srcname, "");
        assert_eq!(cols.targetname, "");
        assert_eq!(cols.preview, "");
    }
}
