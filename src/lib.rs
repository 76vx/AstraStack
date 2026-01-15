use std::{
    collections::HashSet,
    io::{BufRead, Write},
    os::raw::c_char,
    ptr,
    slice,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// TransformProfile defines the operations applied to each record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformProfile {
    pub trim: bool,
    pub to_upper: bool,
    pub drop_empty: bool,
    pub deduplicate: bool,
}

impl Default for TransformProfile {
    fn default() -> Self {
        Self {
            trim: true,
            to_upper: false,
            drop_empty: true,
            deduplicate: false,
        }
    }
}

/// Deduplicator tracks seen strings when deduplication is requested.
pub struct Deduplicator {
    seen: HashSet<String>,
}

impl Deduplicator {
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    pub fn insert(&mut self, value: &str) -> bool {
        self.seen.insert(value.to_owned())
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.seen.clear();
    }
}

/// Stats from a processing run.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub read: usize,
    pub written: usize,
    pub skipped: usize,
}

/// Apply a profile to a single line.
pub fn transform_line(
    line: &str,
    profile: &TransformProfile,
    dedup: &mut Deduplicator,
) -> Option<String> {
    let mut out = line.to_owned();

    if profile.trim {
        out = out.trim().to_owned();
    }

    if profile.to_upper {
        out.make_ascii_uppercase();
    }

    if profile.drop_empty && out.is_empty() {
        return None;
    }

    if profile.deduplicate && !dedup.insert(&out) {
        return None;
    }

    Some(out)
}

/// Process a stream line by line, writing results to the writer.
pub fn process_stream<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
    profile: TransformProfile,
) -> Result<Stats> {
    let mut stats = Stats::default();
    let mut dedup = Deduplicator::new();

    let mut line = String::new();
    loop {
        line.clear();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        stats.read += 1;

        if let Some(transformed) = transform_line(&line, &profile, &mut dedup) {
            writer.write_all(transformed.as_bytes())?;
            writer.write_all(b"\n")?;
            stats.written += 1;
        } else {
            stats.skipped += 1;
        }
    }

    writer.flush()?;
    Ok(stats)
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AstraProfile {
    pub trim: bool,
    pub to_upper: bool,
    pub drop_empty: bool,
    pub deduplicate: bool,
}

impl From<AstraProfile> for TransformProfile {
    fn from(value: AstraProfile) -> Self {
        Self {
            trim: value.trim,
            to_upper: value.to_upper,
            drop_empty: value.drop_empty,
            deduplicate: value.deduplicate,
        }
    }
}

impl From<TransformProfile> for AstraProfile {
    fn from(value: TransformProfile) -> Self {
        Self {
            trim: value.trim,
            to_upper: value.to_upper,
            drop_empty: value.drop_empty,
            deduplicate: value.deduplicate,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct AstraBuffer {
    pub ptr: *mut c_char,
    pub len: usize,
    pub capacity: usize,
}

impl AstraBuffer {
    fn null() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }

    fn empty() -> Self {
        Self::null()
    }

    fn from_string(mut s: String) -> Self {
        let mut bytes = s.into_bytes();
        bytes.push(0);
        let len = bytes.len() - 1;
        let capacity = bytes.capacity();
        let ptr = bytes.as_mut_ptr() as *mut c_char;
        std::mem::forget(bytes);
        Self { ptr, len, capacity }
    }

    pub fn free(buf: AstraBuffer) {
        if buf.ptr.is_null() {
            return;
        }
        unsafe {
            drop(Vec::from_raw_parts(
                buf.ptr,
                buf.len + 1,
                buf.capacity,
            ));
        }
    }
}

/// Session that holds the profile and deduplication state for FFI callers.
pub struct AstraSession {
    profile: TransformProfile,
    dedup: Deduplicator,
}

#[no_mangle]
pub extern "C" fn astra_profile_default() -> AstraProfile {
    TransformProfile::default().into()
}

#[no_mangle]
pub extern "C" fn astra_session_new(profile: AstraProfile) -> *mut AstraSession {
    let profile = profile.into();
    Box::into_raw(Box::new(AstraSession {
        profile,
        dedup: Deduplicator::new(),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn astra_session_free(session: *mut AstraSession) {
    if session.is_null() {
        return;
    }
    drop(Box::from_raw(session));
}

#[no_mangle]
pub unsafe extern "C" fn astra_session_transform(
    session: *mut AstraSession,
    data: *const c_char,
    len: usize,
) -> AstraBuffer {
    if session.is_null() || data.is_null() {
        return AstraBuffer::null();
    }

    let session = &mut *session;
    let bytes = slice::from_raw_parts(data as *const u8, len);
    let text = match std::str::from_utf8(bytes) {
        Ok(v) => v,
        Err(_) => return AstraBuffer::null(),
    };

    match transform_line(text, &session.profile, &mut session.dedup) {
        Some(out) if !out.is_empty() => AstraBuffer::from_string(out),
        _ => AstraBuffer::empty(),
    }
}

#[no_mangle]
pub extern "C" fn astra_buffer_free(buf: AstraBuffer) {
    AstraBuffer::free(buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_uppercases() {
        let profile = TransformProfile {
            trim: true,
            to_upper: true,
            drop_empty: false,
            deduplicate: false,
        };
        let mut dedup = Deduplicator::new();
        let out = transform_line("  hola mundo  ", &profile, &mut dedup).unwrap();
        assert_eq!(out, "HOLA MUNDO");
    }

    #[test]
    fn drops_empty_after_trim() {
        let profile = TransformProfile {
            trim: true,
            to_upper: false,
            drop_empty: true,
            deduplicate: false,
        };
        let mut dedup = Deduplicator::new();
        assert!(transform_line("   ", &profile, &mut dedup).is_none());
    }

    #[test]
    fn deduplicates() {
        let profile = TransformProfile {
            trim: true,
            to_upper: false,
            drop_empty: false,
            deduplicate: true,
        };
        let mut dedup = Deduplicator::new();
        assert!(transform_line("hola", &profile, &mut dedup).is_some());
        assert!(transform_line("hola", &profile, &mut dedup).is_none());
    }

    #[test]
    fn process_stream_counts() {
        let input = "a\nb\na\n".as_bytes();
        let mut output = Vec::new();
        let profile = TransformProfile {
            trim: true,
            to_upper: false,
            drop_empty: true,
            deduplicate: true,
        };
        let stats = process_stream(input, &mut output, profile).unwrap();
        assert_eq!(stats.read, 3);
        assert_eq!(stats.written, 2);
        assert_eq!(stats.skipped, 1);
        let text = String::from_utf8(output).unwrap();
        assert_eq!(text, "a\nb\n");
    }
}
