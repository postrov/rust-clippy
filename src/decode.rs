use std::{io::Read, io::Write, path::Path};

use nut::DBBuilder;

// FIXME: error type
pub fn decode<R: Read, W: Write>(
    db_path: &Path,
    mut input: R,
    mut output: W,
    input_str: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_data = if let Some(s) = input_str {
        s
    } else {
        let mut buffer = String::new();
        input.read_to_string(&mut buffer)?;
        buffer
    };

    let id = crate::common::extract_id(input_data)?;

    let db = DBBuilder::new(db_path).build()?;

    let tx = db.begin_tx()?;
    let bucket = tx.bucket(crate::common::BUCKET_KEY)?;

    // Convert id to bytes for lookup
    let key = crate::common::itob(id);

    if let Some(value) = bucket.get(&key) {
        output.write_all(value)?;
    } else {
        // If key not found, return an error or simply write nothing
        return Err(format!("key not found: {}", id).into());
    }

    Ok(())
}
