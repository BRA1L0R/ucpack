#![cfg_attr(not(feature = "std"), no_std)]

pub mod buffer;
mod de;
mod ser;

use core::fmt::Display;

use buffer::{SliceCursor, WriteBuffer};
use serde::Deserialize;

#[derive(Debug)]
pub enum UcPackError {
    BadVariant,
    Eof,
    NoSupport(&'static str),
    TooLong,
    BufferFull,
    SerError,
    DeError,
    InvalidData,
    WrongCrc,
    WrongIndex,
}

impl Display for UcPackError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            Self::NoSupport(typename) => {
                return write!(f, "there's no support for type {typename}")
            }
            Self::Eof => "not enough data to deserialize",
            Self::InvalidData => "invalid data for data type",
            Self::BadVariant => "tried to serialize a variant index bigger than 255",
            Self::TooLong => "tried to serialize more than 256 bytes",
            Self::BufferFull => "tried to write but buffer reached capacity",
            Self::SerError => "serde encountered an error serializing",
            Self::DeError => "serde encountered an error deserializing",
            Self::WrongCrc => "crc verification failed",
            Self::WrongIndex => "invalid start and/or stop indices",
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

impl serde::de::Error for UcPackError {
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        UcPackError::DeError
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

#[macro_export]
macro_rules! unimpl_de {
    ($func:ident, $type:ty) => {
        fn $func<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            unimpl!(name = core::any::type_name::<$type>())
        }
    };
    ($func:ident, name = $name:expr) => {
        fn $func<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            unimpl!(name = $name)
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
        buffer.push(crc8(&buffer[2..data_end]));

        Ok(buffer)
    }

    pub fn serialize_slice(
        &self,
        payload: &impl serde::ser::Serialize,
        buffer: &mut [u8],
    ) -> Result<usize, UcPackError> {
        let mut cursor = SliceCursor::from_slice(&mut *buffer);
        cursor.push_slice(&[self.start_index, 0])?; // start_index + placeholder for length

        let mut serializer = ser::Serializer::new(&mut cursor);
        payload.serialize(&mut serializer)?;

        let data_end = cursor.index();
        let crc = crc8(&cursor.inner()[2..data_end]);
        cursor.push_slice(&[self.end_index, crc])?;

        let total_size = cursor.index();

        buffer[1] = u8::try_from(data_end - 2).map_err(|_| UcPackError::TooLong)?;
        Ok(total_size)
    }

    pub fn deserialize_slice<'d, 'b, T>(&self, buffer: &'b [u8]) -> Result<T, UcPackError>
    where
        T: Deserialize<'d>,
        'b: 'd,
    {
        let length = buffer
            .get(1)
            .ok_or(UcPackError::Eof)
            .map(|&length| length as usize)?;

        let packet = buffer.get(..(length + 4));
        let Some([index, _, payload @ .., end_index, crc]) = packet else {
            return Err(UcPackError::Eof);
        };

        if cfg!(feature = "strict") && (*index != self.start_index || *end_index != self.end_index)
        {
            return Err(UcPackError::WrongIndex);
        }

        let expected_crc = crc8(payload);
        if expected_crc != *crc {
            return Err(UcPackError::WrongCrc);
        }

        let mut cursor = SliceCursor::from_slice(payload);
        let mut de = de::Deserializer::new(&mut cursor);
        T::deserialize(&mut de)
    }
}

pub fn crc8(input: &[u8]) -> u8 {
    // let input = input.into_iter();

    input
        .into_iter()
        .flat_map(|byte| (0u8..8u8).map(move |j| (byte, j)))
        .fold(0, |mut crc, (byte, j)| {
            let sum = (crc ^ (byte >> j)) & 0x01;
            crc >>= 1;
            crc ^ (sum != 0).then_some(0x8C).unwrap_or(0) // more explicit than unwrap_or_default
        })
}
