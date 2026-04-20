pub(crate) mod ser {
    use serde::{ser, Serialize};
    use crate::{Value, values};

    pub(crate) struct Serializer {
        pub(crate) value: Value,
    }

    impl Serializer {
        pub(crate) fn new() -> Self {
            Self {
                value: Value::Nil,
            }
        }
    }

    type Result<T> = std::result::Result<T, Error>;

    use crate::put::PutValue;

    #[derive(Debug)]
    pub enum Error {
        Mismatch,
        Other,
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            <Self as std::fmt::Debug>::fmt(self, f)
        }
    }

    impl std::error::Error for Error { }

    impl ser::Error for Error {
        fn custom<T>(_: T) -> Self {
            Error::Other
        }
    }

    impl<'a> ser::Serializer for &'a mut Serializer {
        type Ok = ();
        type Error = Error;

        type SerializeSeq = Self;
        type SerializeTuple = Self;
        type SerializeTupleStruct = Self;
        type SerializeTupleVariant = Self;
        type SerializeMap = Self;
        type SerializeStruct = Self;
        type SerializeStructVariant = Self;

        fn serialize_bool(self, v: bool) -> Result<()> {
            let inner = i8::from(v);
            self.value = Value::FixInt(inner);
            Ok(())
        }

        fn serialize_i8(self, v: i8) -> Result<()> {
            self.value = v.put();
            Ok(())
        }

        fn serialize_i16(self, v: i16) -> Result<()> {
            self.value = v.put();
            Ok(())
        }

        fn serialize_i32(self, v: i32) -> Result<()> {
            self.value = v.put();
            Ok(())
        }

        fn serialize_i64(self, v: i64) -> Result<()> {
            self.value = v.put();
            Ok(())
        }

        fn serialize_u8(self, v: u8) -> Result<()> {
            self.value = v.put();
            Ok(())
        }

        fn serialize_u16(self, v: u16) -> Result<()> {
            self.value = v.put();
            Ok(())
        }

        fn serialize_u32(self, v: u32) -> Result<()> {
            self.value = v.put();
            Ok(())
        }

        fn serialize_u64(self, v: u64) -> Result<()> {
            self.value = v.put();
            Ok(())
        }

        fn serialize_f32(self, v: f32) -> Result<()> {
            self.value = Value::F32(v);
            Ok(())
        }

        fn serialize_f64(self, v: f64) -> Result<()> {
            self.value = Value::F64(v);
            Ok(())
        }

        fn serialize_char(self, v: char) -> Result<()> {
            self.serialize_u32(v as _)
        }

        fn serialize_str(self, v: &str) -> Result<()> {
            self.value = v.to_string().into();
            Ok(())
        }

        fn serialize_bytes(self, v: &[u8]) -> Result<()> {
            self.value = Value::Bytes(v.to_vec().into());
            Ok(())
        }    

        fn serialize_none(self) -> Result<()> {
            self.value = values::Variant::new_none().into();
            Ok(())
        }

        fn serialize_some<T>(self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            let mut this = Serializer::new();
            value.serialize(&mut this)?;

            self.value = values::Variant::new_some(this.value).into();
            Ok(())
        }

        fn serialize_unit(self) -> Result<()> {
            self.value = Value::Nil;
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
            let variant = variant_index as i32;
            self.value = variant.put();
            Ok(())
        }

        fn serialize_newtype_struct<T>(
            self,
            _name: &'static str,
            value: &T,
        ) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            value.serialize(self)
        }

        fn serialize_newtype_variant<T>(
            self,
            _name: &'static str,
            variant_index: u32,
            _variant: &'static str,
            value: &T,
        ) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            let index = variant_index as i64;

            let mut this = Serializer::new();
            value.serialize(&mut this)?;

            self.value = values::Variant::new_variant(index, this.value).into();
            Ok(())
        }

        fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
            self.value = Value::Array(vec![].into());
            Ok(self)
        }

        // tuples are teated exactly like arrays
        fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
            self.serialize_seq(Some(len))
        }

        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleStruct> {
            self.serialize_seq(Some(len))
        }

        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleVariant> {
            let variant = variant_index as i64;
            self.value = values::Variant::new_variant(variant, Value::Array(vec![].into())).into();
            Ok(self)
        }

        fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
            self.value = values::Map::new().into();
            Ok(self)
        }

        fn serialize_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStruct> {
            self.value = values::Structure::new().into();
            Ok(self)
        }

        fn serialize_struct_variant(
            self,
            _name: &'static str,
            variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStructVariant> {
            let variant = variant_index as i64;
            self.value = values::Variant::new_variant(variant, values::Structure::new().into()).into();
            Ok(self)
        }
    }

    impl<'a> ser::SerializeSeq for &'a mut Serializer {
        type Ok = ();
        type Error = Error;

        // Serialize a single element of the sequence.
        fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            let mut this = Serializer::new();
            value.serialize(&mut this)?;

            let Value::Array(values::Array { values }) = &mut self.value else {
                return Err(Error::Mismatch);
            };
            values.push_back(this.value);
            Ok(())
        }

        fn end(self) -> Result<()> {
            Ok(())
        }
    }

    impl<'a> ser::SerializeTuple for &'a mut Serializer {
        type Ok = ();
        type Error = Error;

        fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            let mut this = Serializer::new();
            value.serialize(&mut this)?;

            let Value::Array(values::Array { values }) = &mut self.value else {
                return Err(Error::Mismatch);
            };
            values.push_back(this.value);
            Ok(())
        }

        fn end(self) -> Result<()> {
            Ok(())
        }
    }

    impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
        type Ok = ();
        type Error = Error;

        fn serialize_field<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            let mut this = Serializer::new();
            value.serialize(&mut this)?;

            let Value::Array(values::Array { values }) = &mut self.value else {
                return Err(Error::Mismatch);
            };
            values.push_back(this.value);
            Ok(())
        }

        fn end(self) -> Result<()> {
            Ok(())
        }
    }

    impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
        type Ok = ();
        type Error = Error;

        fn serialize_field<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            let mut this = Serializer::new();
            value.serialize(&mut this)?;

            let Value::Variant(v) = &mut self.value else {
                return Err(Error::Mismatch);
            };

            let Some(values) = v.tuple_variant() else {
                return Err(Error::Mismatch);
            };

            values.push_back(this.value);
            Ok(())
        }

        fn end(self) -> Result<()> {
            Ok(())
        }
    }

    impl<'a> ser::SerializeMap for &'a mut Serializer {
        type Ok = ();
        type Error = Error;

        fn serialize_key<T>(&mut self, key: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            let mut this = Serializer::new();
            key.serialize(&mut this)?;

            let Value::Map(values::Map { entries }) = &mut self.value else {
                return Err(Error::Mismatch);
            };
            entries.push_back((this.value, Value::Nil));
            Ok(())
        }

        fn serialize_value<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            let mut this = Serializer::new();
            value.serialize(&mut this)?;

            let Value::Map(values::Map { entries }) = &mut self.value else {
                return Err(Error::Mismatch);
            };

            let Some((_, v)) = entries.back_mut() else {
                return Err(Error::Mismatch);
            };
            *v = this.value;
            Ok(())
        }

        fn end(self) -> Result<()> {
            Ok(())
        }
    }

    impl<'a> ser::SerializeStruct for &'a mut Serializer {
        type Ok = ();
        type Error = Error;

        fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            let mut this = Serializer::new();
            value.serialize(&mut this)?;

            let Value::Structure(values::Structure { values }) = &mut self.value else {
                return Err(Error::Mismatch);
            };
            values.push_back(this.value);
            Ok(())
        }

        fn end(self) -> Result<()> {
            Ok(())
        }
    }

    impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
        type Ok = ();
        type Error = Error;

        fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            let mut this = Serializer::new();
            value.serialize(&mut this)?;

            let Value::Variant(v) = &mut self.value else {
                return Err(Error::Mismatch);
            };

            let Some(values) = v.struct_variant() else {
                return Err(Error::Mismatch);
            };

            values.push_back(this.value);
            Ok(())
        }

        fn end(self) -> Result<()> {
            Ok(())
        }
    }
}

pub(crate) mod de {
    use serde::de::{
        self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess,
        VariantAccess, Visitor,
    };

    use crate::{Value, values};
    pub(crate) struct Deserializer<'a> {
        pub(crate) input: Option<&'a mut Value>,
    }

    impl <'a>  Deserializer<'a> {
        pub(crate) fn new(value: &'a mut Value) -> Self {
            Self {
                input: Some(value)
            }
        }
    }

    type Result<T> = std::result::Result<T, Error>;

    use crate::put::PutValue;

    #[derive(Debug)]
    pub enum Error {
        Mismatch,
        Other,
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            <Self as std::fmt::Debug>::fmt(self, f)
        }
    }

    impl std::error::Error for Error { }

    impl de::Error for Error {
        fn custom<T>(_: T) -> Self {
            Error::Other
        }
    }

    impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'a> {
        type Error = Error;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.input.take() {
                Some(Value::FixInt(v) | Value::I8(v)) => visitor.visit_i8(*v),
                Some(Value::I16(v)) => visitor.visit_i16(*v),
                Some(Value::I32(v)) => visitor.visit_i32(*v),
                Some(Value::I64(v)) => visitor.visit_i64(*v),
                Some(Value::U8(v)) => visitor.visit_u8(*v),
                Some(Value::U16(v)) => visitor.visit_u16(*v),
                Some(Value::U32(v)) => visitor.visit_u32(*v),
                Some(Value::U64(v)) => visitor.visit_u64(*v),
                Some(Value::F32(v)) => visitor.visit_f32(*v),
                Some(Value::F64(v)) => visitor.visit_f64(*v),
                Some(Value::Table(_)) => unimplemented!(),
                Some(Value::Error(_)) => unimplemented!(),
                Some(Value::Handle(_)) => unimplemented!(),
                Some(v @ Value::Variant(_)) => {
                    self.input = Some(v);
                    self.deserialize_enum("", &[], visitor)
                },
                Some(Value::Structure(a)) => visitor.visit_seq(a),
                Some(Value::Array(a)) => visitor.visit_seq(a),
                Some(Value::Map(a)) => visitor.visit_map(a),
                Some(Value::Bytes(a)) => visitor.visit_bytes(&a.inner),
                Some(Value::String(s)) => {
                    let str = str::from_utf8(&s.inner).map_err(|_| Error::Mismatch)?;
                    visitor.visit_str(str)
                }
                Some(Value::Nil) => visitor.visit_unit(),
                _ => return Err(Error::Mismatch),
            }
        }

        fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let Some(Value::FixInt(v)) = self.input.take() else {
                return Err(Error::Mismatch);
            };

            let inner = if *v == 0 {
                true
            } else if *v == 1 {
                false
            } else {
                return Err(Error::Mismatch);
            };
            visitor.visit_bool(inner)
        }

        fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.input.take() {
                Some(Value::FixInt(v) | Value::I8(v)) => visitor.visit_i8(*v),
                _ => return Err(Error::Mismatch),
            }
        }

        fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.input.take() {
                Some(Value::FixInt(v) | Value::I8(v)) => visitor.visit_i16(*v as _),
                Some(Value::I16(v)) => visitor.visit_i16(*v),
                _ => return Err(Error::Mismatch),
            }
        }

        fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.input.take() {
                Some(Value::FixInt(v) | Value::I8(v)) => visitor.visit_i32(*v as _),
                Some(Value::I16(v)) => visitor.visit_i32(*v as _),
                Some(Value::I32(v)) => visitor.visit_i32(*v),
                _ => return Err(Error::Mismatch),
            }
        }

        fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.input.take() {
                Some(Value::FixInt(v) | Value::I8(v)) => visitor.visit_i64(*v as _),
                Some(Value::I16(v)) => visitor.visit_i64(*v as _),
                Some(Value::I32(v)) => visitor.visit_i64(*v as _),
                Some(Value::I64(v)) => visitor.visit_i64(*v),
                _ => return Err(Error::Mismatch),
            }
        }

        fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.input.take() {
                Some(Value::FixInt(v)) if *v >= 0 => visitor.visit_u8(*v as _),
                Some(Value::U8(v)) => visitor.visit_u8(*v),
                _ => return Err(Error::Mismatch),
            }
        }

        fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.input.take() {
                Some(Value::FixInt(v)) if *v >= 0 => visitor.visit_u16(*v as _),
                Some(Value::U8(v)) => visitor.visit_u16(*v as _),
                Some(Value::U16(v)) => visitor.visit_u16(*v),
                _ => return Err(Error::Mismatch),
            }
        }

        fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.input.take() {
                Some(Value::FixInt(v)) if *v >= 0 => visitor.visit_u32(*v as _),
                Some(Value::U8(v)) => visitor.visit_u32(*v as _),
                Some(Value::U16(v)) => visitor.visit_u32(*v as _),
                Some(Value::U32(v)) => visitor.visit_u32(*v),
                _ => return Err(Error::Mismatch),
            }
        }

        fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.input.take() {
                Some(Value::FixInt(v)) if *v >= 0 => visitor.visit_u64(*v as _),
                Some(Value::U8(v)) => visitor.visit_u64(*v as _),
                Some(Value::U16(v)) => visitor.visit_u64(*v as _),
                Some(Value::U32(v)) => visitor.visit_u64(*v as _),
                Some(Value::U64(v)) => visitor.visit_u64(*v),
                _ => return Err(Error::Mismatch),
            }
        }

        fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            if let Some(Value::F32(v)) = self.input.take() {
                visitor.visit_f32(*v)
            } else {
                return Err(Error::Mismatch);
            }
        }

        fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            if let Some(Value::F64(v)) = self.input.take() {
                visitor.visit_f64(*v)
            } else {
                return Err(Error::Mismatch);
            }
        }

        fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.input.take() {
                Some(Value::FixInt(v)) if *v >= 0 => visitor.visit_char(*v as u8 as char),
                Some(Value::U8(v)) => visitor.visit_char(*v as _),
                Some(Value::U16(v)) => visitor.visit_char(char::from_u32(*v as _).ok_or(Error::Mismatch)?),
                Some(Value::U32(v)) => visitor.visit_char(char::from_u32(*v).ok_or(Error::Mismatch)?),
                _ => return Err(Error::Mismatch),
            }
        }

        fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            if let Some(Value::String(v)) = self.input.take() {
                let str = str::from_utf8(&v.inner).map_err(|_| Error::Mismatch)?;
                visitor.visit_str(str)
            } else {
                return Err(Error::Mismatch)
            }
        }

        fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            if let Some(Value::String(v)) = self.input.take() {
                let str = String::try_from(v.inner.clone()).map_err(|_| Error::Mismatch)?;
                visitor.visit_string(str)
            } else {
                return Err(Error::Mismatch)
            }
        }

        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            if let Some(Value::Bytes(v)) = self.input.take() {
                visitor.visit_bytes(&v.inner)
            } else {
                return Err(Error::Mismatch)
            }
        }

        fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            if let Some(Value::Bytes(v)) = self.input.take() {
                visitor.visit_byte_buf(v.inner.clone())
            } else {
                return Err(Error::Mismatch)
            }
        }

        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let Some(Value::Variant(values::Variant { inner })) = self.input.take() else {
                return Err(Error::Mismatch);
            };

            if let Some((_, val)) = inner {
                let mut this = Deserializer::new(&mut* val);
                visitor.visit_some(&mut this)
            } else {
                visitor.visit_none()
            }
        }

        fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let Some(Value::Nil) = self.input.take() else {
                return Err(Error::Mismatch);
            };
            visitor.visit_unit()
        }

        fn deserialize_unit_struct<V>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            self.deserialize_unit(visitor)
        }

        fn deserialize_newtype_struct<V>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            visitor.visit_newtype_struct(self)
        }

        fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let Some(Value::Array(arr)) = self.input.take() else {
                return Err(Error::Mismatch);
            };
            visitor.visit_seq(arr)
        }

        fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            self.deserialize_seq(visitor)
        }

        fn deserialize_tuple_struct<V>(
            self,
            _name: &'static str,
            _len: usize,
            visitor: V,
        ) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            self.deserialize_seq(visitor)
        }

        fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let Some(Value::Map(map)) = self.input.take() else {
                return Err(Error::Mismatch);
            };

            visitor.visit_map(map)
        }

        fn deserialize_struct<V>(
            self,
            _name: &'static str,
            _fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let Some(Value::Structure(st)) = self.input.take() else {
                return Err(Error::Mismatch);
            };
            visitor.visit_seq(st)
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
            visitor.visit_enum(self)
        }

        fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            self.deserialize_u64(visitor)
        }

        fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            self.deserialize_any(visitor)
        }
    }

    impl <'a, 'de> SeqAccess<'de> for &'a mut values::Array {
        type Error = Error;

        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
            where
                T: DeserializeSeed<'de>,
        { 
            let Some(mut next) = self.values.pop_front() else {
                return Ok(None)
            };

            let mut this = Deserializer::new(&mut next);
            seed.deserialize(&mut this).map(Some)
        }
    }

    impl <'a, 'de> SeqAccess<'de> for &'a mut values::Structure {
        type Error = Error;

        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
            where
                T: DeserializeSeed<'de>,
        { 
            let Some(mut next) = self.values.pop_front() else {
                return Ok(None)
            };

            let mut this = Deserializer::new(&mut next);
            seed.deserialize(&mut this).map(Some)
        }
    }

    impl <'a, 'de> MapAccess<'de> for &'a mut values::Map {
        type Error = Error;

        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where
            K: DeserializeSeed<'de>,
        {
            let Some((k, _)) = self.entries.front_mut() else {
                return Ok(None);
            };
            let mut val = core::mem::replace(k, Value::Nil);
            let mut this = Deserializer::new(&mut val);
            seed.deserialize(&mut this).map(Some)
        }

        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where
            V: DeserializeSeed<'de>,
        {
            let Some((_, mut val)) = self.entries.pop_front() else {
                return Err(Error::Mismatch);
            };

            let mut this = Deserializer::new(&mut val);
            seed.deserialize(&mut this)
        }
    }

    impl <'a, 'de> EnumAccess<'de> for &'a mut Deserializer<'a> {
        type Error = Error;
        type Variant = Self;
        fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
        where
            V: DeserializeSeed<'de>,
        {
            let Some(Value::Variant(values::Variant { inner })) = self.input.take() else {
                return Err(Error::Mismatch);
            };

            let (variant, val) = inner.as_mut().map(|(v1, v2)| (*v1, &mut **v2)).unwrap_or((-1, unsafe { MUT_NIL }));
            self.input = Some(val);
            let mut variant = variant.put();
            let mut this = Deserializer::new(&mut variant);
            let val = seed.deserialize(&mut this)?;

            Ok((val, self))
        }
    }

    static mut MUT_NIL: &'static mut Value = &mut Value::Nil;

    impl<'a, 'de> VariantAccess<'de> for  &'a mut Deserializer<'a> {
        type Error = Error;

        fn unit_variant(self) -> Result<()> {
            Err(Error::Mismatch)
        }

        fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
        where
            T: DeserializeSeed<'de>,
        {
            seed.deserialize(self)
        }

        fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            de::Deserializer::deserialize_seq(self, visitor)
        }

        fn struct_variant<V>(
            self,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            de::Deserializer::deserialize_struct(self, "", fields, visitor)
        }
    }
}
