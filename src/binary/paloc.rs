use std::io::{self, Write};

use super::{BinaryRead, BinaryWrite, CString, check_remaining};

// ── Localization Entry ─────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LocalizationEntry<'a> {
    pub unk_id: u64,
    pub string_key: CString<'a>,
    pub string_value: CString<'a>,
}

impl<'a> BinaryRead<'a> for LocalizationEntry<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        Ok(LocalizationEntry {
            unk_id: u64::read_from(data, offset)?,
            string_key: CString::read_from(data, offset)?,
            string_value: CString::read_from(data, offset)?,
        })
    }
}

impl BinaryWrite for LocalizationEntry<'_> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.unk_id.write_to(w)?;
        self.string_key.write_to(w)?;
        self.string_value.write_to(w)
    }
}

// ── Localization File ──────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LocalizationFile<'a> {
    pub entries: Vec<LocalizationEntry<'a>>,
}

impl<'a> LocalizationFile<'a> {
    pub fn parse(data: &'a [u8]) -> io::Result<Self> {
        check_remaining(data, 0, 4)?;
        let count_offset = data.len() - 4;
        let entry_count = u32::from_le_bytes(data[count_offset..].try_into().unwrap()) as usize;

        let mut offset = 0;
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(LocalizationEntry::read_from(data, &mut offset)?);
        }

        if offset != count_offset {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "entry data ends at 0x{:X} but expected 0x{:X} (before trailing count)",
                    offset, count_offset,
                ),
            ));
        }

        Ok(LocalizationFile { entries })
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        for entry in &self.entries {
            entry.write_to(&mut buf)?;
        }
        (self.entries.len() as u32).write_to(&mut buf)?;
        Ok(buf)
    }
}
