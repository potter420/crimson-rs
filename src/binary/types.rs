use std::io::{self, Write};

use super::{BinaryRead, BinaryWrite, check_remaining};

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

// ── CArray ──────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub struct CArray<T> {
    pub items: Vec<T>,
}

impl<'a, T: BinaryRead<'a>> BinaryRead<'a> for CArray<T> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let count = u32::read_from(data, offset)? as usize;
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
