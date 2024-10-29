use crate::UcPackError;

pub trait Buffer {
    fn push_slice(&mut self, bf: &[u8]) -> Result<(), UcPackError>;
    fn push_byte(&mut self, byte: u8) -> Result<(), UcPackError> {
        self.push_slice(&[byte])
    }
}

pub struct SliceCursor<'a> {
    index: usize,
    bf: &'a mut [u8],
}

impl<'a> SliceCursor<'a> {
    pub fn from_slice(bf: &'a mut [u8]) -> Self {
        Self { index: 0, bf }
    }

    pub fn written(&self) -> usize {
        self.index
    }

    pub fn inner(&self) -> &[u8] {
        &self.bf
    }
}

impl Buffer for SliceCursor<'_> {
    fn push_slice(&mut self, data: &[u8]) -> Result<(), UcPackError> {
        let buffer = &mut self.bf[self.index..];
        if data.len() > buffer.len() {
            return Err(UcPackError::BufferFull);
        }

        buffer[..data.len()].copy_from_slice(data); // copy from data

        // self.bf = &mut self.bf[data.len()..]; // advance cursor

        self.index += data.len();
        Ok(())
    }
}

#[cfg(feature = "std")]
impl Buffer for Vec<u8> {
    fn push_slice(&mut self, bf: &[u8]) -> Result<(), UcPackError> {
        self.extend_from_slice(bf);
        Ok(())
    }
}

impl<T: Buffer> Buffer for &mut T {
    fn push_slice(&mut self, bf: &[u8]) -> Result<(), UcPackError> {
        (**self).push_slice(bf)
    }

    fn push_byte(&mut self, byte: u8) -> Result<(), UcPackError> {
        (**self).push_byte(byte)
    }
}
