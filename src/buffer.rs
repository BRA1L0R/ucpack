use core::ops::{Deref, DerefMut};

use crate::UcPackError;

pub(crate) trait WriteBuffer {
    fn push_slice(&mut self, bf: &[u8]) -> Result<(), UcPackError>;
    fn push_byte(&mut self, byte: u8) -> Result<(), UcPackError> {
        self.push_slice(&[byte])
    }
}

pub(crate) trait ReadBuffer {
    fn read_n<const N: usize>(&mut self) -> Result<[u8; N], UcPackError>;
    fn read_u8(&mut self) -> Result<u8, UcPackError> {
        self.read_n().map(|[a]| a)
    }
}

pub(crate) struct SliceCursor<T>
where
    T: Deref<Target = [u8]>,
{
    index: usize,
    buffer: T,
}

impl<T: Deref<Target = [u8]>> SliceCursor<T> {
    pub fn from_slice(bf: T) -> Self {
        Self {
            index: 0,
            buffer: bf,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn inner(&self) -> &[u8] {
        &self.buffer
    }
}

impl<'a, T> ReadBuffer for SliceCursor<T>
where
    T: Deref<Target = [u8]>,
{
    fn read_n<const N: usize>(&mut self) -> Result<[u8; N], UcPackError> {
        let a = self
            .buffer
            .get(self.index..(self.index + N))
            .ok_or(UcPackError::Eof)?
            .try_into()
            .unwrap();

        self.index += N;

        Ok(a)
    }
}

#[cfg(test)]
mod test {
    use super::{SliceCursor, WriteBuffer};

    #[test]
    fn full_err() {
        let mut a = [0, 0, 0, 0, 0];
        let mut cursor = SliceCursor::from_slice(&mut a[..]);

        cursor.push_slice(&[1, 2, 3, 4, 5]).unwrap();
        cursor.push_byte(1).unwrap_err();
    }
}

impl<T> WriteBuffer for SliceCursor<T>
where
    T: DerefMut<Target = [u8]>,
{
    fn push_slice(&mut self, data: &[u8]) -> Result<(), UcPackError> {
        let buffer = &mut self.buffer[self.index..];
        if data.len() > buffer.len() {
            return Err(UcPackError::BufferFull);
        }

        buffer[..data.len()].copy_from_slice(data); // copy from data

        self.index += data.len();
        Ok(())
    }
}

#[cfg(feature = "std")]
impl WriteBuffer for Vec<u8> {
    fn push_slice(&mut self, bf: &[u8]) -> Result<(), UcPackError> {
        self.extend_from_slice(bf);
        Ok(())
    }
}

impl<T: WriteBuffer> WriteBuffer for &mut T {
    #[inline]
    fn push_slice(&mut self, bf: &[u8]) -> Result<(), UcPackError> {
        (**self).push_slice(bf)
    }

    #[inline]
    fn push_byte(&mut self, byte: u8) -> Result<(), UcPackError> {
        (**self).push_byte(byte)
    }
}

impl<T: ReadBuffer> ReadBuffer for &mut T {
    #[inline]
    fn read_u8(&mut self) -> Result<u8, UcPackError> {
        (**self).read_u8()
    }

    #[inline]
    fn read_n<const N: usize>(&mut self) -> Result<[u8; N], UcPackError> {
        (**self).read_n()
    }
}
