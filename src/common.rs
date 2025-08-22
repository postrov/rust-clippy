use byteorder::{BigEndian, ByteOrder};

pub const MAX_SIZE: usize = 5 * 1_000_000; // 5MB
pub const BUCKET_KEY: &[u8] = b"b";
pub const FIELD_SEP: &str = "\t";

use crate::{Error, Result};

pub fn itob(v: u64) -> [u8; 8] {
    let mut b = [0u8; 8];
    BigEndian::write_u64(&mut b, v);
    b
}

pub fn btoi(v: &[u8]) -> u64 {
    BigEndian::read_u64(v)
}

pub fn u64_to_be_bytes(n: u64) -> [u8; 8] {
    n.to_be_bytes()
}

// This one is cleaner, but fails if sep is not present
// pub fn extract_id(input: String) -> Result<u64, ClippyError> {
//     let id_str = input
//         .split_once(FIELD_SEP)
//         .map(|(id, _)| id)
//         .ok_or(ClippyError::IdErr("input not prefixed with id".to_string()))?;
//
//     id_str
//         .parse()
//         .map_err(|_| ClippyError::IdErr("converting id".to_string()))
// }

fn cut(s: &str, sep: char) -> (&str, &str, bool) {
    if let Some(index) = s.find(sep) {
        let before = &s[..index];
        let after = &s[index + sep.len_utf8()..];
        (before, after, true)
    } else {
        (s, "", false)
    }
}

pub fn extract_id(input: String) -> Result<u64> {
    let (id_str, _, _) = cut(&input, '\t');

    if id_str.is_empty() {
        return Err(Error::from("input not prefixed with id"));
    }

    id_str
        .parse::<u64>()
        .map_err(|_| Error::from("converting id"))
}
