use std::collections::{HashMap, HashSet};

use bytes::BytesMut;

mod encode;

pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

pub trait RespDecode {
    fn decode(buf: Self) -> Result<RespFrame, String>;
}
// 之所以要定义一些新的结构体, 是因为要在实现 trait 的时候, 要区分开这些类型
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    NullBulkString(RespNullBulkString),
    Array(RespArray),
    NullArray(RespNullArray),
    Null(RespNull),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

// 这种结构体的主要用途是：
// 1. 类型封装：它为 String 类型创建了一个新的类型。这可以用于增加类型安全性，或者为特定用途的字符串提供一个更有意义的名称。
// 2. 新类型模式：这是 Rust 中常用的一种模式，用于在类型系统层面区分不同用途的相同底层类型。比如，你可能想区分普通的字符串和特定格式的字符串。
// 3. 添加方法：你可以为 SimpleString 实现方法，这些方法特定于这种类型的字符串。
// 4. 语义清晰：在复杂的数据结构中（如你展示的 RespFrame 枚举），使用 SimpleString 而不是直接使用 String 可以使代码的意图更加明确。
pub struct SimpleString(String);
pub struct SimpleError(String);
pub struct BulkString(Vec<u8>);
pub struct RespNullBulkString;
pub struct RespArray(Vec<RespFrame>);
pub struct RespNullArray;
pub struct RespNull;
pub struct RespMap(HashMap<String, RespFrame>);
pub struct RespSet(HashSet<RespFrame>);

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl RespDecode for BytesMut {
    fn decode(_buf: Self) -> Result<RespFrame, String> {
        todo!()
    }
}

impl RespEncode for RespFrame {
    fn encode(self) -> Vec<u8> {
        todo!()
    }
}
