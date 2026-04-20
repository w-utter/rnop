use crate::Value;
use crate::put;
use crate::take;
use crate::take::TakeValue;

use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
pub struct Table {
    hash: u64,
    entries: VecDeque<Entry>,
}

impl TakeValue for Table {
    fn take_from_raw(bytes: &[u8]) -> Option<(usize, Self)> {
        let mut total_size = 0;
        let (size, hash) = take::uint64(bytes)?;
        total_size += size;
        let bytes = &bytes[size..];
        let (size, entry_count) = take::uint64(bytes)?;
        let mut bytes = &bytes[size..];
        total_size += size;

        let mut entries = vec![];

        for _ in 0..entry_count {
            let (size, entry) = Entry::take_from(bytes)?;
            total_size += size;
            bytes = &bytes[size..];
            entries.push(entry);
        }

        Some((
            total_size,
            Table {
                hash,
                entries: entries.into(),
            },
        ))
    }
}

impl put::WriteValue for Table {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        use put::PutValue;
        let mut written = 0;
        written += writer.write(&[0xb5])?;
        written += self.hash.put().write(writer)?;
        let len = self.entries.len() as u64;
        written += len.put().write(writer)?;
        for entry in &self.entries {
            written += entry.write(writer)?;
        }
        Ok(written)
    }

    fn expected_size(&self) -> u64 {
        use put::PutValue;
        let hash = self.hash.put();
        let entries = (self.entries.len() as u64).put();
        1 + hash.expected_size()
            + entries.expected_size()
            + self
                .entries
                .iter()
                .map(|entry| entry.expected_size())
                .sum::<u64>()
    }
}

#[derive(Debug, PartialEq)]
pub struct Entry {
    id: u64,
    val: Value,
}

impl Entry {
    fn take_from(bytes: &[u8]) -> Option<(usize, Self)> {
        let mut total_size = 0;
        let (size, id) = take::uint64(bytes)?;
        total_size += size;
        let bytes = &bytes[size..];
        let (size, len) = take::uint64(bytes)?;
        total_size += size;
        let bytes = &bytes[size..(size + len as usize)];
        let (_, val) = Value::take_from(bytes)?;
        total_size += len as usize;
        Some((total_size, Entry { id, val }))
    }

    fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        use put::PutValue;
        let mut written = 0;
        let id = self.id.put();
        written += id.write(writer)?;
        let len = self.val.expected_size().put();
        written += len.write(writer)?;
        written += self.val.write(writer)?;

        Ok(written)
    }

    fn expected_size(&self) -> u64 {
        use put::PutValue;
        let id = self.id.put();
        let val_len = self.val.expected_size();
        let size = val_len.put();
        id.expected_size() + size.expected_size() + val_len
    }
}

#[derive(Debug, PartialEq)]
pub struct Error {
    inner: i64,
}

impl TakeValue for Error {
    fn take_from_raw(bytes: &[u8]) -> Option<(usize, Self)> {
        let (size, inner) = take::int64(&bytes[1..])?;
        Some((size, Self { inner }))
    }
}

impl put::WriteValue for Error {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        use put::PutValue;
        let mut written = 0;

        let err = self.inner.put();
        written += writer.write(&[0xb6])?;
        written += err.write(writer)?;

        Ok(written)
    }

    fn expected_size(&self) -> u64 {
        use put::PutValue;
        let err = self.inner.put();
        1 + err.expected_size()
    }
}

#[derive(Debug, PartialEq)]
pub struct Handle {
    value: Box<Value>,
    ref_num: i64,
}

impl TakeValue for Handle {
    fn take_from_raw(bytes: &[u8]) -> Option<(usize, Self)> {
        let bytes = &bytes[1..];
        let mut total_size = 0;
        let (size, value) = Value::take_from(bytes)?;
        total_size += size;
        let bytes = &bytes[size..];
        let (size, ref_num) = take::int64(bytes)?;
        total_size += size;

        Some((
            total_size,
            Self {
                value: Box::new(value),
                ref_num,
            },
        ))
    }
}

impl put::WriteValue for Handle {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        use put::PutValue;
        let mut written = 0;

        written += writer.write(&[0xb7])?;
        written += self.value.write(writer)?;
        let ref_num = self.ref_num.put();
        written += ref_num.write(writer)?;

        Ok(written)
    }

    fn expected_size(&self) -> u64 {
        use put::PutValue;
        let ref_num = self.ref_num.put();
        1 + self.value.expected_size() + ref_num.expected_size()
    }
}

#[derive(Debug, PartialEq)]
pub struct Variant {
    pub(crate) inner: Option<(i64, Box<Value>)>,
}

impl Variant {
    pub fn new_none() -> Self {
        Self { inner: None }
    }

    pub fn new_some(val: Value) -> Self {
        Self::new_variant(0, val)
    }

    pub fn new_variant(index: i64, val: Value) -> Self {
        Self {
            inner: Some((index, Box::new(val))),
        }
    }

    #[cfg(feature = "serde")]
    pub(crate) fn tuple_variant(&mut self) -> Option<&mut VecDeque<Value>> {
        let (_, val) = self.inner.as_mut()?;

        match val.as_mut() {
            Value::Array(Array { values }) => Some(values),
            _ => None,
        }
    }

    #[cfg(feature = "serde")]
    pub(crate) fn struct_variant(&mut self) -> Option<&mut VecDeque<Value>> {
        let (_, val) = self.inner.as_mut()?;

        match val.as_mut() {
            Value::Structure(Structure { values }) => Some(values),
            _ => None,
        }
    }
}

impl TakeValue for Variant {
    fn take_from_raw(bytes: &[u8]) -> Option<(usize, Self)> {
        let mut total_size = 0;
        let (size, variant) = take::int64(bytes)?;
        total_size += size;
        let bytes = &bytes[size..];
        let (size, value) = Value::take_from(bytes)?;
        total_size += size;
        let inner = if variant < -1 {
            return None;
        } else if variant < 0 {
            None
        } else {
            Some((variant, Box::new(value)))
        };
        Some((total_size, Self { inner }))
    }
}

impl Variant {
    fn vs(&self) -> (&i64, &Value) {
        self.inner
            .as_ref()
            .map(|(v1, v2)| (v1, v2.as_ref()))
            .unwrap_or((&-1i64, &Value::Nil))
    }
}

impl put::WriteValue for Variant {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        use put::PutValue;
        let mut written = 0;

        written += writer.write(&[0xb8])?;

        let (variant, value) = self.vs();
        let variant = variant.put();
        written += variant.write(writer)?;
        written += value.write(writer)?;

        Ok(written)
    }

    fn expected_size(&self) -> u64 {
        use put::PutValue;
        let (variant, value) = self.vs();
        let variant = variant.put();
        1 + variant.expected_size() + value.expected_size()
    }
}

#[derive(Debug, PartialEq)]
pub struct Structure {
    pub(crate) values: VecDeque<Value>,
}

impl Structure {
    pub fn new() -> Self {
        Self {
            values: vec![].into(),
        }
    }
}

impl TakeValue for Structure {
    fn take_from_raw(bytes: &[u8]) -> Option<(usize, Self)> {
        let (size, values) = take::values(bytes)?;

        Some((
            size,
            Self {
                values: values.into(),
            },
        ))
    }
}

impl put::WriteValue for Structure {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        let mut written = 0;

        written += writer.write(&[0xb9])?;
        written += put::write_values(&self.values, writer)?;

        Ok(written)
    }

    fn expected_size(&self) -> u64 {
        1 + put::values_expected_size(&self.values)
    }
}

#[derive(Debug, PartialEq)]
pub struct Array {
    pub(crate) values: VecDeque<Value>,
}

impl From<Vec<Value>> for Array {
    fn from(f: Vec<Value>) -> Self {
        Self { values: f.into() }
    }
}

impl TakeValue for Array {
    fn take_from_raw(bytes: &[u8]) -> Option<(usize, Self)> {
        let (size, values) = take::values(bytes)?;

        Some((
            size,
            Array {
                values: values.into(),
            },
        ))
    }
}

impl put::WriteValue for Array {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        let mut written = 0;

        written += writer.write(&[0xba])?;
        written += put::write_values(&self.values, writer)?;

        Ok(written)
    }

    fn expected_size(&self) -> u64 {
        1 + put::values_expected_size(&self.values)
    }
}

#[derive(Debug, PartialEq)]
pub struct Map {
    pub(crate) entries: VecDeque<(Value, Value)>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            entries: vec![].into(),
        }
    }
}

impl TakeValue for Map {
    fn take_from_raw(bytes: &[u8]) -> Option<(usize, Self)> {
        let mut total_size = 0;
        let (size, entry_count) = take::uint64(bytes)?;
        let mut bytes = &bytes[size..];
        total_size += size;

        let mut entries = vec![];

        for _ in 0..entry_count {
            let (size, key) = Value::take_from(bytes)?;
            bytes = &bytes[size..];
            total_size += size;
            let (size, value) = Value::take_from(bytes)?;
            bytes = &bytes[size..];
            total_size += size;
            entries.push((key, value))
        }

        Some((
            total_size,
            Self {
                entries: entries.into(),
            },
        ))
    }
}

impl put::WriteValue for Map {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        use put::PutValue;

        let mut written = 0;

        written += writer.write(&[0xbb])?;

        let len = (self.entries.len() as u64).put();
        written += len.write(writer)?;

        for (k, v) in &self.entries {
            written += k.write(writer)?;
            written += v.write(writer)?;
        }
        Ok(written)
    }

    fn expected_size(&self) -> u64 {
        use put::PutValue;
        let len = (self.entries.len() as u64).put();
        1 + len.expected_size()
            + self
                .entries
                .iter()
                .map(|(k, v)| k.expected_size() + v.expected_size())
                .sum::<u64>()
    }
}

#[derive(Debug, PartialEq)]
pub struct Bytes {
    pub(crate) inner: Vec<u8>,
}

impl From<Vec<u8>> for Bytes {
    fn from(f: Vec<u8>) -> Self {
        Self { inner: f }
    }
}

impl TakeValue for Bytes {
    fn take_from_raw(bytes: &[u8]) -> Option<(usize, Self)> {
        let (len, inner) = take::bytes(bytes)?;
        Some((len, Self { inner }))
    }
}

impl put::WriteValue for Bytes {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        let mut written = 0;

        written += writer.write(&[0xbc])?;
        written += put::write_bytes(&self.inner, writer)?;

        Ok(written)
    }

    fn expected_size(&self) -> u64 {
        1 + put::bytes_expected_size(&self.inner)
    }
}

#[derive(Debug, PartialEq)]
pub struct String {
    pub(crate) inner: Vec<u8>,
}

impl From<std::string::String> for String {
    fn from(f: std::string::String) -> String {
        Self {
            inner: f.into_bytes(),
        }
    }
}

impl From<std::string::String> for Value {
    fn from(f: std::string::String) -> Value {
        let s: String = f.into();
        s.into()
    }
}

impl TakeValue for String {
    fn take_from_raw(bytes: &[u8]) -> Option<(usize, Self)> {
        let (len, inner) = take::bytes(bytes)?;
        Some((len, Self { inner }))
    }
}

impl put::WriteValue for String {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        let mut written = 0;

        written += writer.write(&[0xbd])?;
        written += put::write_bytes(&self.inner, writer)?;

        Ok(written)
    }

    fn expected_size(&self) -> u64 {
        1 + put::bytes_expected_size(&self.inner)
    }
}
