mod put;
#[cfg(feature = "serde")]
mod serde;
mod take;
mod values;

#[derive(Debug, PartialEq)]
pub enum Value {
    FixInt(i8),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Table(values::Table),
    Error(values::Error),
    Handle(values::Handle),
    Variant(values::Variant),
    Structure(values::Structure),
    Array(values::Array),
    Map(values::Map),
    Bytes(values::Bytes),
    String(values::String),
    Nil,
}

impl take::TakeValue for Value {
    fn take_from_raw(bytes: &[u8]) -> Option<(usize, Self)> {
        Some(match bytes.get(0)? {
            v @ (0..=0x7f | 0xc0..=0xff) => (1, Value::FixInt(i8::from_le_bytes([*v]))),
            0x80 => u8::take_from(bytes)?,
            0x81 => u16::take_from(bytes)?,
            0x82 => u32::take_from(bytes)?,
            0x83 => u64::take_from(bytes)?,
            0x84 => i8::take_from(bytes)?,
            0x85 => i16::take_from(bytes)?,
            0x86 => i32::take_from(bytes)?,
            0x87 => i64::take_from(bytes)?,
            0x88 => f32::take_from(bytes)?,
            0x89 => f64::take_from(bytes)?,
            0xb5 => values::Table::take_from(bytes)?,
            0xb6 => values::Error::take_from(bytes)?,
            0xb7 => values::Handle::take_from(bytes)?,
            0xb8 => values::Variant::take_from(bytes)?,
            0xb9 => values::Structure::take_from(bytes)?,
            0xba => values::Array::take_from(bytes)?,
            0xbb => values::Map::take_from(bytes)?,
            0xbc => values::Bytes::take_from(bytes)?,
            0xbd => values::String::take_from(bytes)?,
            0xbe => (1, Value::Nil),
            0xbf => unimplemented!("extension type"),
            // in reserved area
            _ => return None,
        })
    }

    fn take_from(bytes: &[u8]) -> Option<(usize, Self)> {
        Self::take_from_raw(bytes)
    }
}

impl put::WriteValue for Value {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        Ok(match self {
            Self::FixInt(n) => writer.write(&n.to_le_bytes())?,
            Self::U8(n) => writer.write(&[0x80])? + writer.write(&n.to_le_bytes())?,
            Self::U16(n) => writer.write(&[0x81])? + writer.write(&n.to_le_bytes())?,
            Self::U32(n) => writer.write(&[0x82])? + writer.write(&n.to_le_bytes())?,
            Self::U64(n) => writer.write(&[0x83])? + writer.write(&n.to_le_bytes())?,
            Self::I8(n) => writer.write(&[0x84])? + writer.write(&n.to_le_bytes())?,
            Self::I16(n) => writer.write(&[0x85])? + writer.write(&n.to_le_bytes())?,
            Self::I32(n) => writer.write(&[0x86])? + writer.write(&n.to_le_bytes())?,
            Self::I64(n) => writer.write(&[0x87])? + writer.write(&n.to_le_bytes())?,
            Self::F32(n) => writer.write(&[0x88])? + writer.write(&n.to_le_bytes())?,
            Self::F64(n) => writer.write(&[0x89])? + writer.write(&n.to_le_bytes())?,

            Self::Table(t) => t.write(writer)?,
            Self::Error(e) => e.write(writer)?,
            Self::Handle(h) => h.write(writer)?,
            Self::Variant(v) => v.write(writer)?,
            Self::Structure(s) => s.write(writer)?,
            Self::Array(a) => a.write(writer)?,
            Self::Map(m) => m.write(writer)?,
            Self::Bytes(b) => b.write(writer)?,
            Self::String(s) => s.write(writer)?,

            Self::Nil => writer.write(&[0xbe])?,
        })
    }

    fn expected_size(&self) -> u64 {
        match self {
            Self::FixInt(_) | Self::Nil => 1,
            Self::U8(_) | Self::I8(_) => 2,
            Self::U16(_) | Self::I16(_) => 3,
            Self::U32(_) | Self::I32(_) | Self::F32(_) => 5,
            Self::U64(_) | Self::I64(_) | Self::F64(_) => 9,

            Self::Table(t) => t.expected_size(),
            Self::Error(e) => e.expected_size(),
            Self::Handle(h) => h.expected_size(),
            Self::Variant(v) => v.expected_size(),
            Self::Structure(s) => s.expected_size(),
            Self::Array(a) => a.expected_size(),
            Self::Map(m) => m.expected_size(),
            Self::Bytes(b) => b.expected_size(),
            Self::String(s) => s.expected_size(),
        }
    }
}

impl Value {
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        use crate::take::TakeValue;
        let (_, val) = Self::take_from(bytes)?;
        Some(val)
    }

    pub fn expected_size(&self) -> u64 {
        <Self as put::WriteValue>::expected_size(self)
    }

    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        <Self as put::WriteValue>::write(self, writer)
    }
}

#[cfg(feature = "serde")]
pub fn to_value<T: ::serde::Serialize>(val: &T) -> Result<Value, crate::serde::ser::Error> {
    let mut serializer = crate::serde::ser::Serializer::new(false);
    val.serialize(&mut serializer)?;
    Ok(serializer.value)
}

/// does the exact same as `to_value`, except that Option<T> is serialized into a Variant<..> instead
/// of Nil for None & the value of T for Some(..)
#[cfg(feature = "serde")]
pub fn to_value_with_options_as_variants<T: ::serde::Serialize>(
    val: &T,
) -> Result<Value, crate::serde::ser::Error> {
    let mut serializer = crate::serde::ser::Serializer::new(true);
    val.serialize(&mut serializer)?;
    Ok(serializer.value)
}

#[cfg(feature = "serde")]
pub fn from_value<T: ::serde::de::DeserializeOwned>(
    mut val: Value,
) -> Result<T, crate::serde::de::Error> {
    let mut deserializer = crate::serde::de::Deserializer::new(&mut val, false);
    T::deserialize(&mut deserializer)
}

/// does the exact same as `to_value`, except that Option<T> is will be expected to be a Variant<..> instead
/// of Nil for None & the value of T for Some(..)
#[cfg(feature = "serde")]
pub fn from_value_with_options_as_variants<T: ::serde::de::DeserializeOwned>(
    mut val: Value,
) -> Result<T, crate::serde::de::Error> {
    let mut deserializer = crate::serde::de::Deserializer::new(&mut val, true);
    T::deserialize(&mut deserializer)
}

#[cfg(test)]
mod tests {
    use super::Value;
    use super::take::TakeValue;
    #[test]
    fn given_example() {
        let bytes = vec![
            0xBA, 0x04, 0xBD, 0x04, 0x61, 0x62, 0x63, 0x64, 0xBD, 0x04, 0x31, 0x32, 0x33, 0x34,
            0xBD, 0x04, 0x41, 0x42, 0x43, 0x44, 0xBD, 0x04, 0x32, 0x34, 0x36, 0x38,
        ];
        let (len, parsed) = Value::take_from(&bytes).unwrap();
        assert_eq!(len, bytes.len());

        let value = super::Value::Array(
            vec![
                String::from("abcd").into(),
                String::from("1234").into(),
                String::from("ABCD").into(),
                String::from("2468").into(),
            ]
            .into(),
        );

        let expected_size = value.expected_size();

        assert_eq!(expected_size, len as u64);
        assert_eq!(parsed, value);

        let mut buf = vec![];
        let written_len = value.write(&mut buf).unwrap();
        assert_eq!(written_len as u64, expected_size);
        assert_eq!(buf, bytes);

        #[cfg(feature = "serde")]
        {
            let s = (
                String::from("abcd"),
                String::from("1234"),
                String::from("ABCD"),
                String::from("2468"),
            );
            let val = super::to_value(&s).unwrap();
            assert_eq!(val, value);
            let de = super::from_value::<(String, String, String, String)>(val).unwrap();
            assert_eq!(de, s);
        }
    }

    #[test]
    fn string() {
        let bytes = vec![0xBD, 0x04, 0x61, 0x62, 0x63, 0x64];
        let (len, parsed) = Value::take_from(&bytes).unwrap();
        assert_eq!(len, bytes.len());
        assert_eq!(parsed, String::from("abcd").into());
    }

    #[test]
    fn variant() {
        let bytes = vec![184, 0, 186, 2, 129, 128, 7, 129, 176, 4];
        let (len, parsed) = Value::take_from(&bytes).unwrap();
        assert_eq!(len, bytes.len());
        assert_eq!(
            parsed,
            crate::values::Variant::new_some(Value::Array(
                vec![Value::U16(1920), Value::U16(1200)].into()
            ))
            .into()
        )
    }
}
