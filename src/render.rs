use crate::colors::{add_color, get_color_for_field};
use crate::types::{Align, ColPaddings, ColType, FileRow};

pub fn get_col_paddings(rows: &[FileRow]) -> ColPaddings {
    let mut paddings = ColPaddings::default();

    for row in rows {
        paddings.acls = paddings.acls.max(row.render.acls.chars().count());
        paddings.owner = paddings.owner.max(row.render.owner.chars().count());
        paddings.filetype = paddings.filetype.max(row.render.filetype.chars().count());
        paddings.size = paddings.size.max(row.render.size.chars().count());
        paddings.timeiso = paddings.timeiso.max(row.render.timeiso.chars().count());
        paddings.srcname = paddings.srcname.max(row.render.srcname.chars().count());
        paddings.targetname = paddings.targetname.max(row.render.targetname.chars().count());
        paddings.preview = paddings.preview.max(row.render.preview.chars().count());
    }

    paddings
}

pub fn add_padding(text: &str, width: usize, align: Align) -> String {
    if text.is_empty() {
        return String::from(" ");
    }

    let text_len = text.chars().count();
    if text_len >= width {
        return text.to_string();
    }

    let pad_amount = width - text_len;
    let padding: String = " ".repeat(pad_amount);

    match align {
        Align::Left => format!("{}{}", text, padding),
        Align::Right => format!("{}{}", padding, text),
    }
}

pub fn get_cols_listing(full: bool) -> Vec<ColType> {
    let mut cols = Vec::new();

    if full {
        cols.push(ColType::Acls);
        cols.push(ColType::Owner);
        cols.push(ColType::FileType);
    }

    cols.push(ColType::Size);
    cols.push(ColType::TimeIso);
    cols.push(ColType::SrcName);
    cols.push(ColType::TargetName);

    if full {
        cols.push(ColType::Preview);
    }

    cols
}

fn get_col_value(row: &FileRow, col: ColType) -> &str {
    match col {
        ColType::Acls => &row.render.acls,
        ColType::Owner => &row.render.owner,
        ColType::FileType => &row.render.filetype,
        ColType::Size => &row.render.size,
        ColType::TimeIso => &row.render.timeiso,
        ColType::SrcName => &row.render.srcname,
        ColType::TargetName => &row.render.targetname,
        ColType::Preview => &row.render.preview,
    }
}

fn get_col_padding(paddings: &ColPaddings, col: ColType) -> usize {
    match col {
        ColType::Acls => paddings.acls,
        ColType::Owner => paddings.owner,
        ColType::FileType => paddings.filetype,
        ColType::Size => paddings.size,
        ColType::TimeIso => paddings.timeiso,
        ColType::SrcName => paddings.srcname,
        ColType::TargetName => paddings.targetname,
        ColType::Preview => paddings.preview,
    }
}

fn get_col_align(col: ColType) -> Align {
    match col {
        ColType::Size => Align::Right,
        _ => Align::Left,
    }
}

fn make_pretty(row: &FileRow, col: ColType, paddings: &ColPaddings) -> String {
    let value = get_col_value(row, col);
    let width = get_col_padding(paddings, col);
    let align = get_col_align(col);
    let color = get_color_for_field(row, col);

    let padded = add_padding(value, width, align);
    add_color(&padded, color)
}

pub fn render_cols(row: &FileRow, paddings: &ColPaddings, full: bool) -> String {
    let margin = "  ";
    let cols = get_cols_listing(full);

    let rendered: Vec<String> = cols.iter().map(|&col| make_pretty(row, col, paddings)).collect();

    format!("{}{}", margin, rendered.join(margin))
}

pub fn render_rows(rows: &[FileRow], full: bool) -> String {
    let paddings = get_col_paddings(rows);

    let rendered: Vec<String> = rows.iter().map(|row| render_cols(row, &paddings, full)).collect();

    rendered.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContentType, FileRowInfo, RenderedCols, StatResult};

    fn make_test_row(fname: &str, ftype: crate::types::FileType) -> FileRow {
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
            render: RenderedCols {
                acls: String::from("644 4"),
                owner: String::from("user:group"),
                filetype: String::from("t"),
                size: String::from("1,024"),
                timeiso: String::from("2024-01-01 00:00:00"),
                srcname: String::from(fname),
                targetname: String::from(" "),
                preview: String::from("content"),
            },
        }
    }

    #[test]
    fn test_add_padding_empty() {
        assert_eq!(add_padding("", 5, Align::Left), " ");
    }

    #[test]
    fn test_add_padding_left() {
        assert_eq!(add_padding("abc", 6, Align::Left), "abc   ");
    }

    #[test]
    fn test_add_padding_right() {
        assert_eq!(add_padding("abc", 6, Align::Right), "   abc");
    }

    #[test]
    fn test_add_padding_exact() {
        assert_eq!(add_padding("abc", 3, Align::Left), "abc");
    }

    #[test]
    fn test_add_padding_overflow() {
        assert_eq!(add_padding("abcdef", 3, Align::Left), "abcdef");
    }

    #[test]
    fn test_get_cols_listing_not_full() {
        let cols = get_cols_listing(false);
        assert_eq!(cols.len(), 4);
        assert_eq!(cols[0], ColType::Size);
        assert_eq!(cols[1], ColType::TimeIso);
        assert_eq!(cols[2], ColType::SrcName);
        assert_eq!(cols[3], ColType::TargetName);
    }

    #[test]
    fn test_get_cols_listing_full() {
        let cols = get_cols_listing(true);
        assert_eq!(cols.len(), 8);
        assert_eq!(cols[0], ColType::Acls);
        assert_eq!(cols[1], ColType::Owner);
        assert_eq!(cols[2], ColType::FileType);
        assert_eq!(cols[7], ColType::Preview);
    }

    #[test]
    fn test_get_col_paddings_empty() {
        let rows: Vec<FileRow> = vec![];
        let paddings = get_col_paddings(&rows);
        assert_eq!(paddings.acls, 0);
        assert_eq!(paddings.size, 0);
    }

    #[test]
    fn test_get_col_paddings_single_row() {
        let rows = vec![make_test_row("test.txt", crate::types::FileType::File)];
        let paddings = get_col_paddings(&rows);
        assert_eq!(paddings.srcname, 8);
        assert_eq!(paddings.size, 5);
    }

    #[test]
    fn test_get_col_paddings_max_widths() {
        let mut row1 = make_test_row("short", crate::types::FileType::File);
        let mut row2 = make_test_row("muchlonger", crate::types::FileType::File);
        row1.render.srcname = String::from("short");
        row2.render.srcname = String::from("muchlonger");
        let rows = vec![row1, row2];
        let paddings = get_col_paddings(&rows);
        assert_eq!(paddings.srcname, 10);
    }

    #[test]
    fn test_render_rows_empty() {
        let rows: Vec<FileRow> = vec![];
        let result = render_rows(&rows, false);
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_rows_contains_margin() {
        let rows = vec![make_test_row("test.txt", crate::types::FileType::File)];
        let paddings = ColPaddings::default();
        let result = render_cols(&rows[0], &paddings, false);
        assert!(result.starts_with("  "));
    }
}
