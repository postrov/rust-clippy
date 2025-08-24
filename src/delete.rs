use crate::{Result, common::BUCKET_KEY};
use nut::DBBuilder;
use std::path::Path;

pub fn delete_last(db_path: &Path) -> Result<()> {
    let mut db = DBBuilder::new(db_path).build()?;
    let mut tx = db.begin_rw_tx()?;
    let mut bucket = tx.bucket_mut(BUCKET_KEY)?;
    let mut key = None;
    {
        let cursor = bucket.cursor()?;
        let last = cursor.last()?;
        if let Some(k) = last.key {
            key = Some(k.to_vec());
        }
    }
    if let Some(k) = key {
        bucket.delete(&k)?;
    }
    Ok(())
}
