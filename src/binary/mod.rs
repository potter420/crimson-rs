mod primitives;
mod types;
mod arrays;

pub use types::*;

use std::io::{self, Write};

// ── Traits ──────────────────────────────────────────────────────────────────

pub trait BinaryRead<'a>: Sized {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self>;
}

pub trait BinaryWrite {
    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()>;
}

// ── Helpers ─────────────────────────────────────────────────────────────────

pub(crate) fn check_remaining(data: &[u8], offset: usize, need: usize) -> io::Result<()> {
    if offset + need > data.len() {
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "not enough data"))
    } else {
        Ok(())
    }
}

// ── Macro for simple structs ────────────────────────────────────────────────

#[macro_export]
macro_rules! binary_struct {
    (
        $(#[$meta:meta])*
        pub struct $name:ident $(<$lt:lifetime>)? {
            $(pub $field:ident : $ty:ty),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name $(<$lt>)? {
            $(pub $field: $ty),*
        }

        impl<'a> $crate::binary::BinaryRead<'a> for $name $(<$lt>)? {
            fn read_from(data: &'a [u8], offset: &mut usize) -> std::io::Result<Self> {
                Ok($name {
                    $($field: $crate::binary::BinaryRead::read_from(data, offset)?),*
                })
            }
        }

        impl $(< $lt >)? $crate::binary::BinaryWrite for $name $(< $lt >)? {
            fn write_to(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
                $($crate::binary::BinaryWrite::write_to(&self.$field, w)?;)*
                Ok(())
            }
        }
    };
}
