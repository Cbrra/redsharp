use std::collections::BTreeMap;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<NbtValue>),
    Compound(BTreeMap<String, NbtValue>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}