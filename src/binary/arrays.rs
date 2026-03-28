use std::io::{self, Write};

use super::{BinaryRead, BinaryWrite};

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
