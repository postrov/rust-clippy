use crate::{
    Error, Result,
    common::{BUCKET_KEY, extract_id, itob},
};
use nut::DBBuilder;
use std::{
    io::{BufRead, BufReader, Read},
    path::Path,
};

pub fn delete_last(db_path: &Path) -> Result<()> {
    let mut db = DBBuilder::new(db_path).build()?;
    let mut tx = db.begin_rw_tx()?;
    {
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
    }
    tx.commit()?;
    Ok(())
}

pub fn delete<R: Read>(db_path: &Path, reader: R) -> Result<()> {
    let buf_reader = BufReader::new(reader);

    let mut db = DBBuilder::new(db_path).build()?;
    let mut tx = db.begin_rw_tx()?;

    for line in buf_reader.lines() {
        let text = line?;
        let id = extract_id(text)?;

        let mut bucket = tx.bucket_mut(BUCKET_KEY)?;
        bucket.delete(&itob(id))?;
    }
    tx.commit()?;

    Ok(())
}

pub fn wipe(db_path: &Path) -> Result<()> {
    let mut db = DBBuilder::new(db_path).build()?;
    let mut tx = db.begin_rw_tx()?;
    {
        let mut bucket = tx.bucket_mut(BUCKET_KEY)?;
        let c = bucket.cursor()?;
        let mut item = c.first()?;
        let mut keys = Vec::new();

        while let Some(key) = item.key {
            keys.push(key.to_vec());
            item = c.next()?;
        }

        for key in keys.into_iter() {
            bucket.delete(&key)?;
        }
    }
    tx.commit()?;

    Ok(())
}

pub fn delete_query(db_path: &Path, query: &str) -> Result<()> {
    if query.is_empty() {
        return Err(Error::from("please provide a query"));
    }
    let mut db = DBBuilder::new(db_path).build()?;
    let mut tx = db.begin_rw_tx()?;
    {
        let mut bucket = tx.bucket_mut(BUCKET_KEY)?;
        let c = bucket.cursor()?;
        let mut item = c.first()?;
        let mut keys = Vec::new();
        let q = query.as_bytes();

        while let Some(key) = item.key {
            if let Some(value) = item.value
                && value.windows(q.len()).any(|window| window == q)
            {
                keys.push(key.to_vec());
            }
            item = c.next()?;
        }

        for key in keys.into_iter() {
            bucket.delete(&key)?;
        }
    }
    tx.commit()?;
    Ok(())
}
