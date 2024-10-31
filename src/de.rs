use serde::de::{self, EnumAccess, IntoDeserializer, SeqAccess, VariantAccess};

use crate::{buffer::ReadBuffer, macros::unimpl, macros::unimpl_de, UcPackError};

/// A `serde` compatible Deserializer which works
/// on a [ReadBuffer]
pub struct Deserializer<B: ReadBuffer> {
    buffer: B,
}

impl<B: ReadBuffer> Deserializer<B> {
    pub fn new(buffer: B) -> Self {
        Self { buffer }
    }

    fn read_u16(&mut self) -> Result<u16, UcPackError> {
        self.buffer.read_n().map(u16::from_le_bytes)
    }
}

impl<'de, 'a, B: ReadBuffer> de::Deserializer<'de> for &'a mut Deserializer<B> {
    type Error = UcPackError;

    fn deserialize_any<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimpl!(name = "any")
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let a = match self.buffer.read_u8()? {
            0 => false,
            1 => true,
            _ => return Err(UcPackError::InvalidData),
        };

        visitor.visit_bool(a)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i8(self.buffer.read_u8()? as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i16(self.read_u16()? as i16)
    }

    unimpl_de!(deserialize_i32, i32);
    unimpl_de!(deserialize_i64, i64);

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u8(self.buffer.read_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u16(self.read_u16()?)
    }

    unimpl_de!(deserialize_u32, u32);
    unimpl_de!(deserialize_u64, u64);

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let float = self.buffer.read_n().map(f32::from_le_bytes)?;
        visitor.visit_f32(float)
    }

    unimpl_de!(deserialize_f64, f64);
    unimpl_de!(deserialize_char, char);
    unimpl_de!(deserialize_str, &str);
    unimpl_de!(deserialize_string, name = "String");
    unimpl_de!(deserialize_bytes, &[u8]);
    unimpl_de!(deserialize_byte_buf, name = "byte_buf");
    unimpl_de!(deserialize_option, name = "option");
    unimpl_de!(deserialize_unit, name = "unit");

    fn deserialize_unit_struct<V>(self, name: &'static str, _: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimpl!(name = name)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    unimpl_de!(deserialize_seq, name = "seq");

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SeriesAccess::new(self, len))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    unimpl_de!(deserialize_map, name = "map");

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_u8(visitor)
    }

    fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimpl!(name = "ignored type")
    }
}

impl<'a, 'de, B: ReadBuffer> VariantAccess<'a> for &'de mut Deserializer<B> {
    type Error = UcPackError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'a>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'a>,
    {
        de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'a>,
    {
        de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}

impl<'a, 'de, B: ReadBuffer> EnumAccess<'a> for &'de mut Deserializer<B> {
    type Error = UcPackError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'a>,
    {
        let variant = self.buffer.read_u8()?;
        let v = seed.deserialize(variant.into_deserializer())?;
        Ok((v, self))
    }
}

struct SeriesAccess<'a, B: ReadBuffer + 'a> {
    deserializer: &'a mut Deserializer<B>,
    remaining: usize,
}

impl<'a, B: ReadBuffer + 'a> SeriesAccess<'a, B> {
    fn new(deserializer: &'a mut Deserializer<B>, len: usize) -> Self {
        Self {
            deserializer,
            remaining: len,
        }
    }
}

impl<'a, 'seq, B: ReadBuffer> SeqAccess<'seq> for SeriesAccess<'a, B> {
    type Error = UcPackError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'seq>,
    {
        // check if remaining
        if self.remaining <= 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        seed.deserialize(&mut *self.deserializer).map(Some)
    }
}
