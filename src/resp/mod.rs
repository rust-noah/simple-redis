use bytes::{Buf, BytesMut};
use enum_dispatch::enum_dispatch;
use std::collections::BTreeMap;
use thiserror::Error;

mod decode;
mod encode;

const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();

// region:    --- Traits
#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

// Sized 表示这个 trait 只能被 [大小确定的类型] 实现
// 因为 decode 方法的返回值是一个 Self, 因此必须将这个 trait 标记为 Sized
pub trait RespDecode: Sized {
    const PREFIX: &'static str;
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}
// endregion: --- Traits

// region:    --- Enum and Structs
#[derive(Error, Debug, PartialEq, Eq)]
pub enum RespError {
    // region:    --- thiserror format usage

    // #[error("{var}")] ⟶ write!("{}", self.var)
    // #[error("{0}")] ⟶ write!("{}", self.0)
    // #[error("{var:?}")] ⟶ write!("{:?}", self.var)
    // #[error("{0:?}")] ⟶ write!("{:?}", self.0)

    // endregion: --- thiserror format usage
    #[error("Invalid frame: {0}")] // 这里的 0 表示 self.0。 会转化为 write!
    InvalidFrame(String),
    #[error("Invalid frame type: {0}")]
    InvalidFrameType(String),
    #[error("Invalid frame length: {0}")]
    InvalidFrameLength(isize),
    #[error("Frame is not complete")]
    NotComplete,

    #[error("Parse error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

// pub trait RespDecode: Sized {
//     const PREFIX: &'static str;
//     fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
//     fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
// }

// 关于 enum 的知识点
// 枚举变体: 直接包含数据, 结构体类型, 无数据

// 元组变体: 当枚举变体直接包含一组命名未指定的值时-> SimpleString(String) 和 Integer(i64),
// 结构体变体: 枚举的变体被定义为包含具有名称的字段-> StructVariant { name: String, id: i32 }
// 单元变体: RespNull

// 之所以要定义一些新的结构体, 是因为要在实现 trait 的时候, 要区分开这些类型
#[enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq)]
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
    Double(f64), // f64 can't derive Eq
    Map(RespMap),
    Set(RespSet),
}

// 这种结构体的主要用途是：
// 1. 类型封装：它为 String 类型创建了一个新的类型。这可以用于增加类型安全性，或者为特定用途的字符串提供一个更有意义的名称。
// 2. 新类型模式：这是 Rust 中常用的一种模式，用于在类型系统层面区分不同用途的相同底层类型。比如，你可能想区分普通的字符串和特定格式的字符串。
// 3. 添加方法：你可以为 SimpleString 实现方法，这些方法特定于这种类型的字符串。
// 4. 语义清晰：在复杂的数据结构中（如你展示的 RespFrame 枚举），使用 SimpleString 而不是直接使用 String 可以使代码的意图更加明确。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleString(pub(crate) String); // Simple String, 用于存储简单字符串
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleError(pub(crate) String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct BulkString(pub(crate) Vec<u8>); // 单个二进制字符串, 用于存储二进制数据(最大512MB)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNullBulkString;
#[derive(Debug, Clone, PartialEq)]
pub struct RespArray(pub(crate) Vec<RespFrame>);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNullArray;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNull;
// 改为 BTreeMap, 用于有序的 key-value 数据
#[derive(Default, Clone, Debug, PartialEq)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);
// pub struct RespSet(HashSet<RespFrame>);
#[derive(Debug, Clone, PartialEq)]
pub struct RespSet(pub(crate) Vec<RespFrame>); // 改为 Vec, 用于有序的集合数据

// endregion: --- Enum and Structs

// region:    --- impls
impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec()).into()
    }
}

impl From<&[u8]> for RespFrame {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec()).into()
    }
}

impl From<&str> for RespFrame {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string()).into()
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(s.as_bytes().to_vec())
    }
}

impl From<String> for BulkString {
    fn from(s: String) -> Self {
        BulkString(s.into_bytes())
    }
}

impl From<&[u8]> for BulkString {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
    }
}
// endregion: --- impls

// region:    --- Functions
// utility functions
fn extract_fixed_data(
    buf: &mut BytesMut,
    expect: &str,
    expect_type: &str,
) -> Result<(), RespError> {
    if buf.len() < expect.len() {
        return Err(RespError::NotComplete);
    }

    if !buf.starts_with(expect.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect: {}, got: {:?}",
            expect_type, buf
        )));
    }

    buf.advance(expect.len());
    Ok(())
}

fn extract_simple_frame_data(buf: &[u8], prefix: &str) -> Result<usize, RespError> {
    if buf.len() < 3 {
        return Err(RespError::NotComplete);
    }

    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect: SimpleString({}), got: {:?}",
            prefix, buf
        )));
    }

    let end = find_crlf(buf, 1).ok_or(RespError::NotComplete)?;

    Ok(end)
}

// find nth CRLF in the buffer
fn find_crlf(buf: &[u8], nth: usize) -> Option<usize> {
    let mut count = 0;
    for i in 1..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            count += 1;
            if count == nth {
                return Some(i);
            }
        }
    }
    None
}

fn parse_length(buf: &[u8], prefix: &str) -> Result<(usize, usize), RespError> {
    let end = extract_simple_frame_data(buf, prefix)?;
    let s = String::from_utf8_lossy(&buf[prefix.len()..end]);
    Ok((end, s.parse()?))
}

fn calc_total_length(buf: &[u8], end: usize, len: usize, prefix: &str) -> Result<usize, RespError> {
    let mut total = end + CRLF_LEN;
    let mut data = &buf[total..];
    match prefix {
        "*" | "~" => {
            // find nth CRLF in the buffer, for array and set, we need to find 1 CRLF for each element
            for _ in 0..len {
                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        "%" => {
            // find nth CRLF in the buffer. For map, we need to find 2 CRLF for each key-value pair
            for _ in 0..len {
                let len = SimpleString::expect_length(data)?;

                data = &data[len..];
                total += len;

                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        _ => Ok(len + CRLF_LEN),
    }
}

// endregion: --- Functions
