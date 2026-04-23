use std::io::{self, Write};

use super::{
    BinaryRead, BinaryReadTracked, BinaryWrite, FieldRange, check_remaining,
    push_index, push_path, pop_path,
};

// ── CString ─────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub struct CString<'a> {
    pub length: u32,
    pub data: &'a str,
}

impl<'a> BinaryRead<'a> for CString<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let length = u32::read_from(data, offset)?;
        let len = length as usize;
        check_remaining(data, *offset, len)?;
        let bytes = &data[*offset..*offset + len];
        let s = std::str::from_utf8(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        *offset += len;
        Ok(CString { length, data: s })
    }
}

impl BinaryWrite for CString<'_> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.length.write_to(w)?;
        w.write_all(self.data.as_bytes())
    }
}

impl<'a> BinaryReadTracked<'a> for CString<'a> {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        // length prefix (u32) — recorded under `<path>.__len__`
        let len_start = *offset;
        let length = u32::read_from(data, offset)?;
        let saved = push_path(path, "__len__");
        ranges.push(FieldRange {
            path: path.clone(),
            start: len_start,
            end: *offset,
            ty: "CString.len",
        });
        pop_path(path, saved);

        // payload bytes — recorded under the field path itself
        let payload_start = *offset;
        let len = length as usize;
        check_remaining(data, *offset, len)?;
        let bytes = &data[*offset..*offset + len];
        let s = std::str::from_utf8(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        *offset += len;
        ranges.push(FieldRange {
            path: path.clone(),
            start: payload_start,
            end: *offset,
            ty: "CString",
        });
        Ok(CString { length, data: s })
    }
}

// ── CArray ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub struct CArray<T> {
    pub items: Vec<T>,
}

impl<'a, T: BinaryRead<'a>> BinaryRead<'a> for CArray<T> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let count = u32::read_from(data, offset)? as usize;
        // Sanity clamp: even the smallest element is >= 1 byte, so a
        // count exceeding the remaining byte budget can only be a
        // corrupted stream (e.g. a mod that byte-patched a count
        // prefix). Without this check, `Vec::with_capacity(huge)` can
        // attempt a multi-GB allocation before the actual read fails.
        let remaining = data.len().saturating_sub(*offset);
        if count > remaining {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!(
                    "CArray count {} exceeds remaining bytes {} at offset {}",
                    count, remaining, *offset,
                )));
        }
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(T::read_from(data, offset)?);
        }
        Ok(CArray { items })
    }
}

impl<T: BinaryWrite> BinaryWrite for CArray<T> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        (self.items.len() as u32).write_to(w)?;
        for item in &self.items {
            item.write_to(w)?;
        }
        Ok(())
    }
}

impl<'a, T: BinaryReadTracked<'a>> BinaryReadTracked<'a> for CArray<T> {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        // count prefix (u32) — recorded under `<path>.__count__`
        let count_start = *offset;
        let count = u32::read_from(data, offset)? as usize;
        let saved = push_path(path, "__count__");
        ranges.push(FieldRange {
            path: path.clone(),
            start: count_start,
            end: *offset,
            ty: "CArray.count",
        });
        pop_path(path, saved);

        // Same sanity clamp as `BinaryRead` impl — see notes there.
        let remaining = data.len().saturating_sub(*offset);
        if count > remaining {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!(
                    "CArray count {} exceeds remaining bytes {} at offset {}",
                    count, remaining, *offset,
                )));
        }

        let mut items = Vec::with_capacity(count);
        for i in 0..count {
            let saved = push_index(path, i);
            items.push(T::read_tracked(data, offset, path, ranges)?);
            pop_path(path, saved);
        }
        Ok(CArray { items })
    }
}

// ── COptional ───────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub struct COptional<T> {
    pub value: Option<T>,
}

impl<'a, T: BinaryRead<'a>> BinaryRead<'a> for COptional<T> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let flag = u8::read_from(data, offset)?;
        let value = if flag != 0 {
            Some(T::read_from(data, offset)?)
        } else {
            None
        };
        Ok(COptional { value })
    }
}

impl<T: BinaryWrite> BinaryWrite for COptional<T> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match &self.value {
            Some(v) => {
                1u8.write_to(w)?;
                v.write_to(w)
            }
            None => 0u8.write_to(w),
        }
    }
}

impl<'a, T: BinaryReadTracked<'a>> BinaryReadTracked<'a> for COptional<T> {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        // presence tag (u8) — recorded under `<path>.__tag__`
        let tag_start = *offset;
        let flag = u8::read_from(data, offset)?;
        let saved = push_path(path, "__tag__");
        ranges.push(FieldRange {
            path: path.clone(),
            start: tag_start,
            end: *offset,
            ty: "COptional.tag",
        });
        pop_path(path, saved);

        let value = if flag != 0 {
            Some(T::read_tracked(data, offset, path, ranges)?)
        } else {
            None
        };
        Ok(COptional { value })
    }
}

// ── LocalizableString ───────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub struct LocalizableString<'a> {
    pub category: u8,
    pub index: u64,
    pub default: CString<'a>,
}

impl<'a> BinaryRead<'a> for LocalizableString<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let category = u8::read_from(data, offset)?;
        let index = u64::read_from(data, offset)?;
        let default = CString::read_from(data, offset)?;
        Ok(LocalizableString { category, index, default })
    }
}

impl BinaryWrite for LocalizableString<'_> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.category.write_to(w)?;
        self.index.write_to(w)?;
        self.default.write_to(w)
    }
}

impl<'a> BinaryReadTracked<'a> for LocalizableString<'a> {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let saved = push_path(path, "category");
        let category = u8::read_tracked(data, offset, path, ranges)?;
        pop_path(path, saved);

        let saved = push_path(path, "index");
        let index = u64::read_tracked(data, offset, path, ranges)?;
        pop_path(path, saved);

        let saved = push_path(path, "default");
        let default = CString::read_tracked(data, offset, path, ranges)?;
        pop_path(path, saved);

        Ok(LocalizableString { category, index, default })
    }
}
