use crate::Value;

pub(crate) trait PutValue {
    fn put(self) -> Value;
}

pub(crate) trait WriteValue {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize>;
    fn expected_size(&self) -> u64;
    fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        let expected = self.expected_size() as usize;
        if self.write_to(writer)? != expected {
            panic!()
        }
        Ok(expected)
    }
}

macro_rules! num_range {
    ($i:ty) => {
        (<$i>::MIN as _)..=(<$i>::MAX as _)
    };
}

macro_rules! fixint_range {
    () => {
        -64..=127
    };
}

macro_rules! fixuint_range {
    () => {
        0..=127
    };
}

impl PutValue for i8 {
    fn put(self) -> Value {
        if fixint_range!().contains(&self) {
            Value::FixInt(self)
        } else {
            Value::I8(self)
        }
    }
}

impl PutValue for i16 {
    fn put(self) -> Value {
        if fixint_range!().contains(&self) {
            Value::FixInt(self as _)
        } else if num_range!(i8).contains(&self) {
            Value::I8(self as _)
        } else {
            Value::I16(self)
        }
    }
}

impl PutValue for i32 {
    fn put(self) -> Value {
        if fixint_range!().contains(&self) {
            Value::FixInt(self as _)
        } else if num_range!(i8).contains(&self) {
            Value::I8(self as _)
        } else if num_range!(i16).contains(&self) {
            Value::I16(self as _)
        } else {
            Value::I32(self)
        }
    }
}

impl PutValue for i64 {
    fn put(self) -> Value {
        if fixint_range!().contains(&self) {
            Value::FixInt(self as _)
        } else if num_range!(i8).contains(&self) {
            Value::I8(self as _)
        } else if num_range!(i16).contains(&self) {
            Value::I16(self as _)
        } else if num_range!(i32).contains(&self) {
            Value::I32(self as _)
        } else {
            Value::I64(self)
        }
    }
}

// unsigned

impl PutValue for u8 {
    fn put(self) -> Value {
        if fixuint_range!().contains(&self) {
            Value::FixInt(self as _)
        } else {
            Value::U8(self)
        }
    }
}

impl PutValue for u16 {
    fn put(self) -> Value {
        if fixuint_range!().contains(&self) {
            Value::FixInt(self as _)
        } else if num_range!(u8).contains(&self) {
            Value::U8(self as _)
        } else {
            Value::U16(self)
        }
    }
}

impl PutValue for u32 {
    fn put(self) -> Value {
        if fixuint_range!().contains(&self) {
            Value::FixInt(self as _)
        } else if num_range!(u8).contains(&self) {
            Value::U8(self as _)
        } else if num_range!(u16).contains(&self) {
            Value::U16(self as _)
        } else {
            Value::U32(self)
        }
    }
}

impl PutValue for u64 {
    fn put(self) -> Value {
        if fixuint_range!().contains(&self) {
            Value::FixInt(self as _)
        } else if num_range!(u8).contains(&self) {
            Value::U8(self as _)
        } else if num_range!(u16).contains(&self) {
            Value::U16(self as _)
        } else if num_range!(u32).contains(&self) {
            Value::U32(self as _)
        } else {
            Value::U64(self)
        }
    }
}

use std::collections::VecDeque;

pub(crate) fn write_values<W: std::io::Write>(
    values: &VecDeque<Value>,
    writer: &mut W,
) -> std::io::Result<usize> {
    let mut written = 0;
    let len = (values.len() as u64).put();
    written += len.write(writer)?;
    for value in values {
        written += value.write(writer)?;
    }
    Ok(written)
}

pub(crate) fn values_expected_size(values: &VecDeque<Value>) -> u64 {
    let len = (values.len() as u64).put();
    len.expected_size()
        + values
            .iter()
            .map(|value| value.expected_size())
            .sum::<u64>()
}

pub(crate) fn write_bytes<W: std::io::Write>(
    bytes: &[u8],
    writer: &mut W,
) -> std::io::Result<usize> {
    let mut written = 0;
    let len = (bytes.len() as u64).put();
    written += len.write(writer)?;
    written += writer.write(bytes)?;
    Ok(written)
}

pub(crate) fn bytes_expected_size(bytes: &[u8]) -> u64 {
    let len = bytes.len() as u64;
    len.put().expected_size() + len
}
