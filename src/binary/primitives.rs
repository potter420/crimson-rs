use std::io::{self, Write};

use super::{BinaryRead, BinaryReadTracked, BinaryWrite, FieldRange, check_remaining};

// Helper: record a leaf `FieldRange` for a primitive that delegates to
// `BinaryRead::read_from`. Keeps the per-primitive impl a three-liner.
macro_rules! tracked_leaf {
    ($t:ty, $ty_name:literal) => {
        impl<'a> BinaryReadTracked<'a> for $t {
            fn read_tracked(
                data: &'a [u8],
                offset: &mut usize,
                path: &mut String,
                ranges: &mut Vec<FieldRange>,
            ) -> io::Result<Self> {
                let start = *offset;
                let v = <$t as BinaryRead>::read_from(data, offset)?;
                ranges.push(FieldRange {
                    path: path.clone(),
                    start,
                    end: *offset,
                    ty: $ty_name,
                });
                Ok(v)
            }
        }
    };
}

tracked_leaf!(u8, "u8");
tracked_leaf!(u16, "u16");
tracked_leaf!(u32, "u32");
tracked_leaf!(u64, "u64");
tracked_leaf!(i8, "i8");
tracked_leaf!(i32, "i32");
tracked_leaf!(i64, "i64");
tracked_leaf!(f32, "f32");

impl<'a> BinaryRead<'a> for u8 {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        check_remaining(data, *offset, 1)?;
        let v = data[*offset];
        *offset += 1;
        Ok(v)
    }
}

impl BinaryWrite for u8 {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&[*self])
    }
}

impl<'a> BinaryRead<'a> for u16 {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        check_remaining(data, *offset, 2)?;
        let v = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap());
        *offset += 2;
        Ok(v)
    }
}

impl BinaryWrite for u16 {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}

impl<'a> BinaryRead<'a> for u32 {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        check_remaining(data, *offset, 4)?;
        let v = u32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
        *offset += 4;
        Ok(v)
    }
}

impl BinaryWrite for u32 {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}

impl<'a> BinaryRead<'a> for u64 {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        check_remaining(data, *offset, 8)?;
        let v = u64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
        *offset += 8;
        Ok(v)
    }
}

impl BinaryWrite for u64 {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}

impl<'a> BinaryRead<'a> for i8 {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        check_remaining(data, *offset, 1)?;
        let v = data[*offset] as i8;
        *offset += 1;
        Ok(v)
    }
}

impl BinaryWrite for i8 {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}

impl<'a> BinaryRead<'a> for i32 {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        check_remaining(data, *offset, 4)?;
        let v = i32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
        *offset += 4;
        Ok(v)
    }
}

impl BinaryWrite for i32 {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}

impl<'a> BinaryRead<'a> for i64 {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        check_remaining(data, *offset, 8)?;
        let v = i64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
        *offset += 8;
        Ok(v)
    }
}

impl BinaryWrite for i64 {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}

impl<'a> BinaryRead<'a> for f32 {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        check_remaining(data, *offset, 4)?;
        let v = f32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
        *offset += 4;
        Ok(v)
    }
}

impl BinaryWrite for f32 {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.to_le_bytes())
    }
}
