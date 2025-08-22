use nut::Bucket;
use nut::DBBuilder;
use std::io::Read;
use std::path::Path;

use crate::Result;
use crate::common::*;

// #[derive(Error, Debug)]
// pub enum StoreError {
//     #[error("failed to read input: {0}")]
//     ReadInput(#[source] std::io::Error),
//
//     #[error("opening db: {0}")]
//     OpenDb(#[source] nut::Error),
//
//     #[error("begin tx: {0}")]
//     BeginTx(#[source] nut::Error),
//
//     #[error("bucket not found")]
//     BucketNotFound,
//
//     #[error("deduplicating: {0}")]
//     Deduplicate(String),
//
//     #[error("getting next sequence: {0}")]
//     GetNextSequence(#[source] nut::Error),
//
//     #[error("put value: {0}")]
//     Put(#[source] nut::Error),
//
//     #[error("trimming length: {0}")]
//     TrimLength(#[source] nut::Error),
//
//     #[error("commit tx: {0}")]
//     CommitTx(#[source] nut::Error),
// }

pub fn store<R: Read>(
    db_path: &Path,
    mut reader: R,
    max_dedupe_search: u64,
    max_items: u64,
) -> Result<()> {
    // Read all input
    let mut input = Vec::new();
    reader.read_to_end(&mut input)?;

    if input.len() > MAX_SIZE {
        return Ok(()); // TODO: do we really want to return ok here?
    }

    let mut db = DBBuilder::new(db_path).build()?;

    if input.iter().all(|b| b.is_ascii_whitespace()) {
        return Ok(());
    }
    // TODO: the above checks if trimmed is empty, we still want to trim the input though

    // Start a writable transaction
    let mut tx = db.begin_rw_tx()?;

    // Access or create bucket with the name = BUCKET_KEY
    {
        let mut bucket = tx.bucket_mut(BUCKET_KEY)?;

        deduplicate(&mut bucket, &input, max_dedupe_search)?;

        // Get next sequence id
        let id = bucket.next_sequence()?;

        bucket.put(&u64_to_be_bytes(id), input)?;

        trim_length(&mut bucket, max_items)?;
    }
    tx.commit()?;

    Ok(())
}

// Trim the bucket length to max_items by deleting the oldest entries
fn trim_length(bucket: &mut Bucket, max_items: u64) -> Result<()> {
    // Count bucket items
    let mut count = 0;
    {
        let c = bucket.cursor()?;
        let mut item = c.first()?;
        loop {
            if item.is_none() {
                break;
            }
            count += 1;
            item = c.next()?;
        }
    }

    if count <= max_items {
        return Ok(());
    }

    let to_remove = count - max_items;
    let mut removed = 0;
    let mut keys_to_remove = Vec::with_capacity(to_remove.try_into().unwrap_or(0));

    // bucket.cursor() returns items in key order, assuming numeric big-endian keys
    {
        let c = bucket.cursor()?;
        let mut item = c.first()?;
        loop {
            if removed >= to_remove {
                break;
            }
            if let Some(key) = item.key {
                keys_to_remove.push(key.to_vec());
            } else {
                break;
            }
            removed += 1;
            item = c.next()?;
        }
    }

    for key in keys_to_remove.into_iter() {
        bucket.delete(&key)?;
    }

    Ok(())
}

fn deduplicate(bucket: &mut Bucket, input: &[u8], max_dedupe_search: u64) -> Result<()> {
    let c = bucket.cursor()?;
    let mut item = c.last()?;
    let mut seen = 0;
    let mut to_remove = Vec::new();
    loop {
        if item.is_none() || seen > max_dedupe_search {
            break;
        }
        let value = item.value.unwrap_or(&[]);
        if input == value
            && let Some(key) = item.key
        {
            to_remove.push(key.to_vec());
        }

        item = c.prev()?;
        seen += 1;
    }

    for key in to_remove.into_iter() {
        bucket.delete(&key)?;
    }

    Ok(())
}
