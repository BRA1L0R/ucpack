#![cfg_attr(not(feature = "std"), no_std)]

pub mod buffer;
pub mod ser;

use core::fmt::Display;

use buffer::{Buffer, SliceCursor};

#[derive(Debug)]
pub enum UcPackError {
    NoSupport(&'static str),
    TooLong,
    BufferFull,
    SerError,
    WrongCrc,
}

impl Display for UcPackError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            UcPackError::NoSupport(typename) => {
                return write!(f, "there's no support for type {typename}")
            }
            Self::TooLong => "tried to serialize more than 256 bytes",
            Self::BufferFull => "tried to write but buffer reached capacity",
            Self::SerError => "serde encountered an error",
            Self::WrongCrc => "crc verification failed",
        };

        f.write_str(msg)
    }
}

impl serde::ser::Error for UcPackError {
    fn custom<T>(_: T) -> Self
    where
        T: Display,
    {
        Self::SerError
    }
}
// impl core for UcPackError {}

#[macro_export]
macro_rules! unimpl {
    (name = $name:expr) => {{
        return Err(UcPackError::NoSupport($name));
    }};

    ($func:tt) => {
        fn $func(self) -> Result<Self::Ok, Self::Error> {
            Err(UcPackError::NoSupport(""))
        }
    };

    ($func:ident, $type:ty) => {
        fn $func(self, _: $type) -> Result<Self::Ok, Self::Error> {
            Err(UcPackError::NoSupport(core::any::type_name::<$type>()))
        }
    };
}

/// UcPack structure
pub struct UcPack {
    start_index: u8,
    end_index: u8,
}

impl Default for UcPack {
    fn default() -> Self {
        Self {
            start_index: b'A',
            end_index: b'#',
        }
    }
}

impl UcPack {
    #[cfg(feature = "std")]
    pub fn serialize_vec(
        &self,
        payload: &impl serde::ser::Serialize,
    ) -> Result<Vec<u8>, UcPackError> {
        let mut buffer = vec![self.start_index, 0];

        let mut serializer = ser::Serializer::new(&mut buffer);
        payload.serialize(&mut serializer)?;

        let data_end = buffer.len();
        buffer[1] = u8::try_from(data_end - 2).map_err(|_| UcPackError::TooLong)?;

        buffer.push(self.end_index);
        buffer.push(crc8(buffer[2..data_end].into_iter().copied()));

        Ok(buffer)
    }

    pub fn serialize_slice(
        &self,
        buffer: &mut [u8],
        payload: &impl serde::ser::Serialize,
    ) -> Result<usize, UcPackError> {
        let mut cursor = SliceCursor::from_slice(buffer);
        cursor.push_slice(&[self.start_index, 0])?; // start_index + placeholder for length

        let mut serializer = ser::Serializer::new(&mut cursor);
        payload.serialize(&mut serializer)?;

        let data_end = cursor.written();
        let crc = crc8(cursor.inner()[2..data_end].into_iter().copied());
        cursor.push_slice(&[self.end_index, crc])?;

        let total_size = cursor.written();

        buffer[1] = u8::try_from(data_end - 2).map_err(|_| UcPackError::TooLong)?;
        Ok(total_size)
    }
}

pub fn crc8(input: impl IntoIterator<Item = u8>) -> u8 {
    let input = input.into_iter();

    input
        .flat_map(|byte| (0u8..8u8).map(move |j| (byte, j)))
        .fold(0, |mut crc, (byte, j)| {
            let sum = (crc ^ (byte >> j)) & 0x01;
            crc >>= 1;
            crc ^ (sum != 0).then_some(0x8C).unwrap_or(0) // more explicit than unwrap_or_default
        })
}
