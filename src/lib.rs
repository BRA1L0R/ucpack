#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod buffer;
pub mod de;
mod macros;
pub mod ser;

use core::fmt::Display;

use buffer::{SliceCursor, WriteBuffer};
use serde::Deserialize;

#[derive(Debug)]
/// Error returned by the ucpack crate
pub enum UcPackError {
    /// Tried to serialize a variant index bigger than `255`.
    BadVariant,
    /// The cursor does not have any more data to deserialize from.
    Eof,
    /// Serialization / Deserialization of this type is not supported by the ucpack protocol.
    /// If you think this is a mistake, please open an issue.
    NoSupport(&'static str),
    /// Tried to serialize more than 256 bytes of payload data. This is a restriction
    /// imposed by the protocol.
    TooLong,
    /// Tried to serialize more bytes than the buffer could possible handle.
    BufferFull,
    /// There was a serde error during serialization.
    SerError,
    /// There was a serde error during deserialization.
    DeError,
    /// Input data for deserialization has problems finding a representation in a given data format
    ///
    /// For example: a serialized boolean value ∉ {0, 1}
    InvalidData,
    /// Received a message with a wrong/faulty crc. Probably indicates data corruption.
    WrongCrc,
    /// Received a message containing wrong index/indices for the start and stop bytes.
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

#[cfg(feature = "std")]
impl std::error::Error for UcPackError {}

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

/// UcPack structure
pub struct UcPack {
    start_index: u8,
    end_index: u8,
}

impl Default for UcPack {
    fn default() -> Self {
        Self::new(b'A', b'#')
    }
}

impl UcPack {
    pub const fn new(start_index: u8, end_index: u8) -> Self {
        Self {
            start_index,
            end_index,
        }
    }

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
        buffer.push(crc8_slice(&buffer[2..data_end]));

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
        let crc = crc8_slice(&cursor.inner()[2..data_end]);
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
        let packet = is_complete_message(buffer).ok_or(UcPackError::Eof)?;
        let [index, _, payload @ .., end_index, crc] = packet else {
            return Err(UcPackError::Eof);
        };

        if cfg!(feature = "strict") && (*index != self.start_index || *end_index != self.end_index)
        {
            return Err(UcPackError::WrongIndex);
        }

        let expected_crc = crc8_slice(payload);
        if expected_crc != *crc {
            return Err(UcPackError::WrongCrc);
        }

        let mut cursor = SliceCursor::from_slice(payload);
        let mut de = de::Deserializer::new(&mut cursor);
        T::deserialize(&mut de)
    }
}

/// Check a buffer for a message. This method is useful during hardware interrupts,
/// to check whether the received data is a readble message or more data has yet to arrive
///
/// Arguments:
/// - `buffer`: this argument is NOT for the whole buffer to be passed in but
/// rather the slice of the buffer containing the currently received information
///
/// Returns:
/// - `Some`: a slice guaranteed to contain a message
/// - `None`: a full message hasn't yet been received
pub fn is_complete_message(buffer: &[u8]) -> Option<&[u8]> {
    let length: usize = buffer.get(1).map(|&length| length.into())?;
    buffer.get(..(length + 4))
}

/// Helper function to calculate crc8 over byte slices
#[inline]
pub fn crc8_slice(input: &[u8]) -> u8 {
    crc8(input.into_iter().copied())
}

/// Calculates a CRC8 checksum over any `u8` iterator
pub fn crc8(input: impl IntoIterator<Item = u8>) -> u8 {
    let input = input.into_iter();

    input
        .into_iter()
        .flat_map(|byte| (0u8..8u8).map(move |j| (byte, j)))
        .fold(0, |mut crc, (byte, j)| {
            let sum = (crc ^ (byte >> j)) & 0x01;
            crc >>= 1;
            crc ^ (sum != 0).then_some(0x8C).unwrap_or(0) // more explicit than unwrap_or_default
        })
}
