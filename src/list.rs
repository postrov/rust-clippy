use nut::DBBuilder;
use std::{io::Write, path::Path};
use thiserror::Error;

use image::ImageReader;
use std::io::Cursor;

// Helper function for truncation with suffix
fn trunc(s: &str, width: usize, suffix: &str) -> String {
    if s.chars().count() <= width {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(width).collect();
        format!("{}{}", truncated, suffix)
    }
}

fn size_str(size: usize) -> String {
    let units = ["B", "KiB", "MiB"];
    let mut fsize = size as f64;
    let mut i = 0;

    while fsize >= 1024.0 && i < units.len() - 1 {
        fsize /= 1024.0;
        i += 1;
    }

    format!("{:.0} {}", fsize, units[i])
}

fn preview(index: u64, data: &[u8], width: u64) -> String {
    // Try to decode image config
    let cursor = Cursor::new(data);
    let image_reader = ImageReader::new(cursor).with_guessed_format().ok();

    let format_str = image_reader
        .as_ref()
        .and_then(|reader| reader.format())
        .map(|f| format!("{:?}", f).to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());

    let dimensions = image_reader
        .and_then(|reader| reader.decode().ok())
        .map(|img| (img.width(), img.height()));

    if let Some((w, h)) = dimensions {
        // image::ImageFormat is not returned here, so we guess format from image data
        // If you want format string, you'd have to separately guess it or use a different approach
        // For simplicity, mark format as unknown
        return format!(
            "{}{}[[ binary data {} {} {}x{} ]]",
            index,
            crate::common::FIELD_SEP,
            size_str(data.len()),
            format_str,
            w,
            h
        );
    }

    // If not an image, treat as text
    let prev = String::from_utf8_lossy(data);
    let prev_trimmed = prev.trim();
    // Replace multiple whitespace with single space
    let prev_cleaned = prev_trimmed
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let truncated = trunc(&prev_cleaned, width as usize, "…");

    format!("{}{}{}", index, crate::common::FIELD_SEP, truncated)
}

#[derive(Error, Debug)]
pub enum ListError {
    #[error("opening db: {0}")]
    OpenDb(#[source] nut::Error),

    #[error("begin tx: {0}")]
    BeginTx(#[source] nut::Error),

    #[error("bucket not found")]
    BucketNotFound,
}

pub fn list<W: Write>(db_path: &Path, mut out: W, preview_width: u64) -> Result<(), ListError> {
    let db = DBBuilder::new(db_path).build().map_err(ListError::OpenDb)?;

    let tx = db.begin_tx().map_err(ListError::BeginTx)?;

    let bucket = tx
        .bucket(crate::common::BUCKET_KEY)
        .map_err(|_| ListError::BucketNotFound)?;

    // TODO: error mapping instead of unwraps below
    let c = bucket.cursor().unwrap();
    let mut item = c.last().unwrap();
    loop {
        if item.is_none() {
            break;
        }
        let k = item.key.unwrap();
        let v = item.value.unwrap();
        out.write_all(preview(crate::common::btoi(k), v, preview_width).as_bytes())
            .unwrap();
        out.write_all("\n".as_bytes()).unwrap(); // FIXME
        item = c.prev().unwrap();
    }

    Ok(())
}
