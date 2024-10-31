use core::fmt::Display;

use serde::ser;
use serde::ser::Impossible;

use crate::{buffer::WriteBuffer, macros::unimpl, UcPackError};

pub struct Serializer<B: WriteBuffer> {
    buffer: B,
}

impl<B: WriteBuffer> Serializer<B> {
    pub fn new(buffer: B) -> Serializer<B> {
        Self { buffer }
    }
}

impl<'a, B: WriteBuffer> ser::Serializer for &'a mut Serializer<B> {
    type Ok = ();
    type Error = UcPackError;

    type SerializeSeq = Impossible<(), UcPackError>;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Impossible<(), UcPackError>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.serialize_u8(v as u8)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.buffer.push_u8(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u8(v as u8)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        let bytes = v.to_le_bytes();
        self.buffer.push_slice(&bytes)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u16(v as u16)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let bytes = v.to_le_bytes();
        self.buffer.push_slice(&bytes)
    }

    unimpl!(serialize_u32, u32);
    unimpl!(serialize_i32, i32);
    unimpl!(serialize_u64, u64);
    unimpl!(serialize_i64, i64);
    unimpl!(serialize_u128, u128);
    unimpl!(serialize_i128, i128);
    unimpl!(serialize_f64, f64);
    unimpl!(serialize_char, char);
    unimpl!(serialize_str, &str);
    unimpl!(serialize_bytes, &[u8]);
    unimpl!(serialize_none);
    unimpl!(serialize_unit);
    unimpl!(serialize_unit_struct, &'static str);
    // unimpl!(serialize_seq, Option<usize>);

    fn collect_str<T>(self, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        unimpl!(name = "string")
    }

    fn serialize_some<T>(self, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        unimpl!(name = "Some")
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        // no clear way of doing it so up to implementor's
        // ability to use serialize_with attributes

        unimpl!(name = name)
    }

    fn serialize_newtype_struct<T>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        idx: u32,
        _: &'static str,
        obj: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let idx = u8::try_from(idx).map_err(|_| UcPackError::BadVariant)?;
        self.serialize_u8(idx)?;
        obj.serialize(self)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unimpl!(name = "sequence")
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        idx: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let idx = u8::try_from(idx).map_err(|_| UcPackError::BadVariant)?;
        self.serialize_u8(idx)?;
        Ok(self)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unimpl!(name = "map")
    }

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        idx: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_tuple_variant(name, idx, variant, len)
    }
}

impl<'a, B: WriteBuffer> ser::SerializeTuple for &'a mut Serializer<B> {
    type Ok = ();
    type Error = UcPackError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, B: WriteBuffer> ser::SerializeStruct for &'a mut Serializer<B> {
    type Ok = ();
    type Error = UcPackError;

    fn serialize_field<T>(&mut self, _: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, B: WriteBuffer> ser::SerializeTupleStruct for &'a mut Serializer<B> {
    type Ok = ();
    type Error = UcPackError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, B: WriteBuffer> ser::SerializeTupleVariant for &'a mut Serializer<B> {
    type Ok = ();
    type Error = UcPackError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, B: WriteBuffer> ser::SerializeStructVariant for &'a mut Serializer<B> {
    type Ok = ();

    type Error = UcPackError;

    fn serialize_field<T>(&mut self, _: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
