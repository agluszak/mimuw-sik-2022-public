use std::io::{Cursor, Read};

use crate::serialize::DeserError;
use byteorder::{NetworkEndian, ReadBytesExt};
use serde::de::{
    self, DeserializeOwned, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;

use super::error::Result;

pub struct Deserializer<R: Read> {
    input: R,
}

impl<R: Read> Deserializer<R> {
    pub fn new(input: R) -> Self {
        Deserializer { input }
    }

    pub fn deserialize<T: DeserializeOwned>(&mut self) -> Result<T> {
        T::deserialize(self)
    }
}

impl<'de> Deserializer<Cursor<&'de [u8]>> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input: Cursor::new(input),
        }
    }
}

pub fn from_bytes<'de, T>(input: &'de [u8]) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::from_bytes(input);
    let value = T::deserialize(&mut deserializer)?;

    if deserializer.input.position() == deserializer.input.get_ref().len() as u64 {
        Ok(value)
    } else {
        Err(DeserError::TrailingData)
    }
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = DeserError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let byte = self.input.read_i8()?;
        match byte {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            _ => Err(DeserError::InvalidBool),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.input.read_i8()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.input.read_i16::<byteorder::NetworkEndian>()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.input.read_i32::<byteorder::NetworkEndian>()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.input.read_i64::<byteorder::NetworkEndian>()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.input.read_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.input.read_u16::<byteorder::NetworkEndian>()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.input.read_u32::<byteorder::NetworkEndian>()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.input.read_u64::<byteorder::NetworkEndian>()?)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // TODO: this can be improved in terms of borrowing
        let length = self.input.read_u8()?;
        let mut buf = vec![0; length as usize];
        self.input.read_exact(&mut buf)?;
        visitor.visit_str(std::str::from_utf8(&buf)?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let byte = self.input.read_u8()?;
        match byte {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            _ => Err(DeserError::InvalidOption),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let byte = self.input.read_u8()?;
        match byte {
            0 => visitor.visit_unit(),
            _ => Err(DeserError::InvalidUnit),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let length = self.input.read_u32::<NetworkEndian>()?;
        visitor.visit_seq(Counted::new(self, length as usize))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Counted::new(self, len))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let length = self.input.read_u32::<NetworkEndian>()?;
        visitor.visit_map(Counted::new(self, length as usize))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = fields.len();
        visitor.visit_seq(Counted::new(self, len))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(Variant::new(self))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u8(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct Variant<'a, R: Read> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: Read> Variant<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        Variant { de }
    }
}

impl<'de, 'a, R: Read> EnumAccess<'de> for Variant<'a, R> {
    type Error = DeserError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a, R: Read> VariantAccess<'de> for Variant<'a, R> {
    type Error = DeserError;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self.de, len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_struct(self.de, "", fields, visitor)
    }
}

struct Counted<'a, R: Read> {
    de: &'a mut Deserializer<R>,
    index: usize,
    length: usize,
}

impl<'a, R: Read> Counted<'a, R> {
    fn new(de: &'a mut Deserializer<R>, length: usize) -> Self {
        Counted {
            de,
            index: 0,
            length,
        }
    }
}

impl<'de, 'a, R: Read> SeqAccess<'de> for Counted<'a, R> {
    type Error = DeserError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        // Check if there are no more elements.
        if self.index == self.length {
            return Ok(None);
        }
        // Deserialize an array element.
        let element = seed.deserialize(&mut *self.de).map(Some);
        self.index += 1;
        element
    }
}

impl<'de, 'a, R: Read> MapAccess<'de> for Counted<'a, R> {
    type Error = DeserError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // Check if there are no more entries.
        if self.index == self.length {
            return Ok(None);
        }
        // Deserialize a map key.
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self.de);
        self.index += 1;
        value
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize_bool() {
        let buf = vec![0x01];
        let result: bool = from_bytes(&buf).unwrap();
        assert!(result);

        let buf = vec![0x00];
        let result: bool = from_bytes(&buf).unwrap();
        assert!(!result);

        let buf = vec![0x02];
        let result = from_bytes::<bool>(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_u32() {
        let buf = vec![0x12, 0x34, 0x56, 0x78];
        let result: u32 = from_bytes(&buf).unwrap();
        assert_eq!(result, 0x12345678);
    }

    #[test]
    fn test_deserialize_string() {
        let buf = vec![3, b'a', b'b', b'c'];
        let result: String = from_bytes(&buf).unwrap();
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_deserialize_struct() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct Message {
            message: String,
        }

        let message = Message {
            message: "hello".to_string(),
        };
        let buf = [5, b'h', b'e', b'l', b'l', b'o'];
        let result: Message = from_bytes(&buf).unwrap();
        assert_eq!(result, message);
    }

    #[test]
    fn deserialize_struct_tuple() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct Message(String, u32);
        let message = Message("hello".to_string(), 0x12345678);
        let buf = [5, b'h', b'e', b'l', b'l', b'o', 0x12, 0x34, 0x56, 0x78];
        let result: Message = from_bytes(&buf).unwrap();
        assert_eq!(result, message);
    }

    #[test]
    fn test_deserialize_enum() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        enum Message {
            Hello(String),
            Goodbye(String),
        }

        let buf = vec![0, 5, b'w', b'o', b'r', b'l', b'd'];

        let message = Message::Hello("world".to_string());
        let result: Message = from_bytes(&buf).unwrap();
        assert_eq!(result, message);

        let message = Message::Goodbye("worlds".to_string());
        let buf = [1, 6, b'w', b'o', b'r', b'l', b'd', b's'];
        let result: Message = from_bytes(&buf).unwrap();
        assert_eq!(result, message);
    }

    #[test]
    fn test_deserialize_nested_enum() {
        #[derive(Deserialize, Debug, Eq, PartialEq)]
        enum Message {
            Hello,
            Goodbye,
        }

        #[derive(Deserialize, Debug, Eq, PartialEq)]
        enum NestedMessage {
            Variant1(Message),
            Variant2 { message: Message, num: u16 },
        }

        let message = NestedMessage::Variant1(Message::Hello);
        let buf = [0, 0];
        let result: NestedMessage = from_bytes(&buf).unwrap();
        assert_eq!(result, message);

        let message = NestedMessage::Variant1(Message::Goodbye);
        let buf = [0, 1];
        let result: NestedMessage = from_bytes(&buf).unwrap();
        assert_eq!(result, message);

        let message = NestedMessage::Variant2 {
            message: Message::Hello,
            num: 0x1234,
        };
        let buf = [1, 0, 0x12, 0x34];
        let result: NestedMessage = from_bytes(&buf).unwrap();
        assert_eq!(result, message);
    }

    #[test]
    fn test_deserialize_vec() {
        #[derive(Deserialize, Debug, Eq, PartialEq)]
        enum Message {
            Hello(String),
            Goodbye,
        }
        let vec = vec![
            Message::Hello("world".to_string()),
            Message::Goodbye,
            Message::Hello("something".to_string()),
        ];
        let buf = [
            0, 0, 0, 3, 0, 5, b'w', b'o', b'r', b'l', b'd', 1, 0, 9, b's', b'o', b'm', b'e', b't',
            b'h', b'i', b'n', b'g',
        ];
        let result: Vec<Message> = from_bytes(&buf).unwrap();
        assert_eq!(result, vec);
    }

    #[test]
    fn test_serialize_map() {
        #[derive(Deserialize, Debug, Eq, PartialEq)]
        enum Message {
            Hello(String),
            Goodbye,
        }
        let map = vec![
            ("hello".to_string(), Message::Hello("world".to_string())),
            ("goodbye".to_string(), Message::Goodbye),
        ];
        let buf = [
            0, 0, 0, 2, 5, b'h', b'e', b'l', b'l', b'o', 0, 5, b'w', b'o', b'r', b'l', b'd', 7,
            b'g', b'o', b'o', b'd', b'b', b'y', b'e', 1,
        ];
        let result: Vec<(String, Message)> = from_bytes(&buf).unwrap();
        assert_eq!(result, map);
    }
}
