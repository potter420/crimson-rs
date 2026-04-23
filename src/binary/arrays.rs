use std::io::{self, Write};

use super::{BinaryRead, BinaryReadTracked, BinaryWrite, FieldRange, push_index, pop_path};

impl<'a> BinaryRead<'a> for [f32; 3] {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        Ok([
            f32::read_from(data, offset)?,
            f32::read_from(data, offset)?,
            f32::read_from(data, offset)?,
        ])
    }
}

impl BinaryWrite for [f32; 3] {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        for v in self {
            v.write_to(w)?;
        }
        Ok(())
    }
}

impl<'a> BinaryRead<'a> for [u32; 4] {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        Ok([
            u32::read_from(data, offset)?,
            u32::read_from(data, offset)?,
            u32::read_from(data, offset)?,
            u32::read_from(data, offset)?,
        ])
    }
}

impl BinaryWrite for [u32; 4] {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        for v in self {
            v.write_to(w)?;
        }
        Ok(())
    }
}

impl<'a> BinaryRead<'a> for [u8; 3] {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        super::check_remaining(data, *offset, 3)?;
        let arr = [data[*offset], data[*offset + 1], data[*offset + 2]];
        *offset += 3;
        Ok(arr)
    }
}

impl BinaryWrite for [u8; 3] {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(self)
    }
}

// ── Fixed-size array tracked reads ──────────────────────────────────────────
// Each element is reported as `<path>[i]` so the byte layout is preserved.

impl<'a> BinaryReadTracked<'a> for [f32; 3] {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let mut out = [0f32; 3];
        for i in 0..3 {
            let saved = push_index(path, i);
            out[i] = f32::read_tracked(data, offset, path, ranges)?;
            pop_path(path, saved);
        }
        Ok(out)
    }
}

impl<'a> BinaryReadTracked<'a> for [u32; 4] {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let mut out = [0u32; 4];
        for i in 0..4 {
            let saved = push_index(path, i);
            out[i] = u32::read_tracked(data, offset, path, ranges)?;
            pop_path(path, saved);
        }
        Ok(out)
    }
}

impl<'a> BinaryReadTracked<'a> for [u8; 3] {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let mut out = [0u8; 3];
        for i in 0..3 {
            let saved = push_index(path, i);
            out[i] = u8::read_tracked(data, offset, path, ranges)?;
            pop_path(path, saved);
        }
        Ok(out)
    }
}
