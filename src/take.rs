use crate::Value;

pub(crate) trait TakeValue: Into<Value> {
    fn take_from_raw(bytes: &[u8]) -> Option<(usize, Self)>;
    fn take_from(bytes: &[u8]) -> Option<(usize, Value)> {
        let (size, this) = Self::take_from_raw(bytes)?;
        Some((size + 1, this.into()))
    }
}

macro_rules! into_value {
    ($($i:ty => $variant:ident),*) => {
        $(
            impl From<$i> for Value {
                fn from(val: $i) -> Value {
                    Value::$variant(val)
                }
            }
         )*
    }
}

macro_rules! impl_int {
    ($($i:ty => $variant:ident),*) => {
        $(
            impl From<$i> for Value {
                fn from(val: $i) -> Value {
                    Value::$variant(val)
                }
            }

            impl TakeValue for $i {
                fn take_from_raw(bytes: &[u8]) -> Option<(usize, Self)> {
                    const SIZE: usize = core::mem::size_of::<$i>();
                    let bytes = bytes[1..].first_chunk::<SIZE>()?;
                    Some((SIZE, <$i>::from_le_bytes(*bytes)))
                }
            }
        )*
    }
}

impl_int! {
    u8 => U8, u16 => U16, u32 => U32, u64 => U64,
    i8 => I8, i16 => I16, i32 => I32, i64 => I64,
    f32 => F32, f64 => F64
}

use crate::values;

into_value! {
    values::Table => Table,
    values::Error => Error,
    values::Handle => Handle,
    values::Variant => Variant,
    values::Structure => Structure,
    values::Array => Array,
    values::Map => Map,
    values::Bytes => Bytes,
    values::String => String
}

pub(crate) fn uint64(bytes: &[u8]) -> Option<(usize, u64)> {
    let bytes = &bytes[1..];
    Some(match bytes.get(0)? {
        v @ 0..=0x7f => (1, *v as u64),
        0x80 => u8::take_from_raw(bytes).map(|(len, val)| (len + 1, val as u64))?,
        0x81 => u16::take_from_raw(bytes).map(|(len, val)| (len + 1, val as u64))?,
        0x82 => u32::take_from_raw(bytes).map(|(len, val)| (len + 1, val as u64))?,
        0x83 => u64::take_from_raw(bytes).map(|(len, val)| (len + 1, val))?,
        _ => return None,
    })
}

pub(crate) fn int64(bytes: &[u8]) -> Option<(usize, i64)> {
    let bytes = &bytes[1..];
    Some(match bytes.get(0)? {
        v @ (0..=0x7f | 0xc0..=0xff) => (1, i8::from_le_bytes([*v]) as i64),
        0x84 => i8::take_from_raw(bytes).map(|(len, val)| (len + 1, val as i64))?,
        0x85 => i16::take_from_raw(bytes).map(|(len, val)| (len + 1, val as i64))?,
        0x86 => i32::take_from_raw(bytes).map(|(len, val)| (len + 1, val as i64))?,
        0x87 => i64::take_from_raw(bytes).map(|(len, val)| (len + 1, val))?,
        _ => return None,
    })
}

pub(crate) fn values(bytes: &[u8]) -> Option<(usize, Vec<super::Value>)> {
    let (size, len) = crate::take::uint64(bytes)?;
    let mut bytes = &bytes[size + 1..];
    let mut values = vec![];
    let mut total_size = size;
    for _ in 0..len {
        let (len, value) = Value::take_from(bytes)?;
        values.push(value);
        bytes = &bytes[len..];
        total_size += len;
    }

    Some((total_size, values))
}

pub(crate) fn bytes(bytes: &[u8]) -> Option<(usize, Vec<u8>)> {
    let (size, len) = crate::take::uint64(bytes)?;
    let total_len = size + len as usize;
    let bytes = bytes.get(size + 1..total_len + 1)?;
    Some((total_len, bytes.to_vec()))
}
