use byteorder::{BigEndian, NetworkEndian, WriteBytesExt};
use serde::{ser, Serialize};

use crate::serialize::{DeserError, Result};

pub struct Serializer {
    output: Vec<u8>,
}

pub fn to_bytes<T: Serialize>(value: T) -> Vec<u8> {
    let mut serializer = Serializer { output: Vec::new() };
    value.serialize(&mut serializer).unwrap(); // Serialization in our protocol is infallible
    serializer.output
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = DeserError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_u8(if v { 1 } else { 0 })?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.output.write_i8(v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.output.write_i16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.output.write_i32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output.write_i64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.write_u8(v)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.output.write_u16::<NetworkEndian>(v)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.output.write_u32::<NetworkEndian>(v)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output.write_u64::<NetworkEndian>(v)?;
        Ok(())
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        unimplemented!()
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        unimplemented!()
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        unimplemented!()
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.serialize_bytes(v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        let length: u8 = v.len().try_into()?;
        self.serialize_u8(length)?;
        self.output.extend_from_slice(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_u8(0)?;
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.serialize_u8(1)?;
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_unit(self) -> Result<()> {
        self.serialize_u8(0)?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        let variant_index: u8 = variant_index.try_into()?;
        self.serialize_u8(variant_index)?;
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        let variant_index: u8 = variant_index.try_into()?;
        self.serialize_u8(variant_index)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if let Some(len) = len {
            let len: u32 = len.try_into()?;
            self.serialize_u32(len)?;
            Ok(self)
        } else {
            Err(DeserError::UnknownLength)
        }
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_struct_variant(_name, variant_index, _variant, _len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.serialize_seq(len)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let variant_index: u8 = variant_index.try_into()?;
        self.serialize_u8(variant_index)?;
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = DeserError;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = DeserError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = DeserError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = DeserError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = DeserError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = DeserError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = DeserError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialize_u8() {
        let value: u8 = 0xFF;
        let serialized = to_bytes(&value);
        assert_eq!(serialized, [0xFF]);
    }

    #[test]
    fn test_serialize_u16() {
        let value: u16 = 0x1234;
        let serialized = to_bytes(&value);
        assert_eq!(serialized, [0x12, 0x34]);
    }

    #[test]
    fn test_serialize_u32() {
        let value: u32 = 0x12345678;
        let serialized = to_bytes(&value);
        assert_eq!(serialized, [0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_serialize_enum() {
        #[derive(Serialize)]
        enum Message {
            Hello(String),
            Goodbye(String),
        }

        let message = Message::Hello("world".to_string());
        let serialized = to_bytes(&message);
        assert_eq!(serialized, [0, 5, b'w', b'o', b'r', b'l', b'd']);

        let message = Message::Goodbye("worlds".to_string());
        let serialized = to_bytes(&message);
        assert_eq!(serialized, [1, 6, b'w', b'o', b'r', b'l', b'd', b's']);
    }

    #[test]
    fn test_serialize_struct() {
        #[derive(Serialize)]
        struct Message {
            message: String,
        }

        let message = Message {
            message: "hello".to_string(),
        };
        let serialized = to_bytes(&message);
        assert_eq!(serialized, [5, b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn test_serialize_nested_enum() {
        #[derive(Serialize)]
        enum Message {
            Hello,
            Goodbye,
        }

        #[derive(Serialize)]
        enum NestedMessage {
            Variant1(Message),
            Variant2 { message: Message, num: u16 },
        }

        let message = NestedMessage::Variant1(Message::Hello);
        let serialized = to_bytes(&message);
        assert_eq!(serialized, [0, 0]);

        let message = NestedMessage::Variant1(Message::Goodbye);
        let serialized = to_bytes(&message);
        assert_eq!(serialized, [0, 1]);

        let message = NestedMessage::Variant2 {
            message: Message::Hello,
            num: 0x1234,
        };
        let serialized = to_bytes(&message);
        assert_eq!(serialized, [1, 0, 0x12, 0x34]);
    }

    #[test]
    fn test_serialize_vec() {
        #[derive(Serialize)]
        enum Message {
            Hello(String),
            Goodbye,
        }
        let vec = vec![
            Message::Hello("world".to_string()),
            Message::Goodbye,
            Message::Hello("something".to_string()),
        ];
        let serialized = to_bytes(&vec);
        assert_eq!(
            serialized,
            [
                0, 0, 0, 3, 0, 5, b'w', b'o', b'r', b'l', b'd', 1, 0, 9, b's', b'o', b'm', b'e',
                b't', b'h', b'i', b'n', b'g'
            ]
        );
    }

    #[test]
    fn test_serialize_map() {
        #[derive(Serialize)]
        enum Message {
            Hello(String),
            Goodbye,
        }
        let map = vec![
            ("hello".to_string(), Message::Hello("world".to_string())),
            ("goodbye".to_string(), Message::Goodbye),
        ];
        let serialized = to_bytes(&map);
        assert_eq!(
            serialized,
            [
                0, 0, 0, 2, 5, b'h', b'e', b'l', b'l', b'o', 0, 5, b'w', b'o', b'r', b'l', b'd', 7,
                b'g', b'o', b'o', b'd', b'b', b'y', b'e', 1
            ]
        );
    }

    #[test]
    fn test_serialize_tuple_enum() {
        #[derive(Serialize)]
        enum Message {
            Hello(u16, u16),
            Goodbye,
        }
        let vec = vec![Message::Hello(0x1234, 0x5678), Message::Goodbye];
        let serialized = to_bytes(&vec);
        assert_eq!(serialized, [0, 0, 0, 2, 0, 0x12, 0x34, 0x56, 0x78, 1]);
    }

    #[test]
    fn test_serialize_tuple_struct() {
        #[derive(Serialize)]
        struct Message(u16, u16);
        let vec = vec![Message(0x1234, 0x5678), Message(0x9abc, 0xdef0)];
        let serialized = to_bytes(&vec);
        assert_eq!(
            serialized,
            [0, 0, 0, 2, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]
        );
    }
}
