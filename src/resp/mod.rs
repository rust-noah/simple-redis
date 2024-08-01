mod array;
mod bool;
mod bulk_string;
mod double;
mod frame;
mod integer;
mod map;
mod null;
mod set;
mod simple_error;
mod simple_string;

use bytes::{Buf, BytesMut};
use enum_dispatch::enum_dispatch;
use thiserror::Error;

pub(crate) use self::{
    array::RespArray, bulk_string::BulkString, frame::RespFrame, map::RespMap, null::RespNull,
    set::RespSet, simple_error::SimpleError, simple_string::SimpleString,
};

const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();
const BUF_CAP: usize = 4096;

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
// region:    --- before refactor
// 改为 Vec, 用于有序的集合数据
// 这种结构体的主要用途是：
// 1. 类型封装：它为 String 类型创建了一个新的类型。这可以用于增加类型安全性，或者为特定用途的字符串提供一个更有意义的名称。
// 2. 新类型模式：这是 Rust 中常用的一种模式，用于在类型系统层面区分不同用途的相同底层类型。比如，你可能想区分普通的字符串和特定格式的字符串。
// 3. 添加方法：你可以为 SimpleString 实现方法，这些方法特定于这种类型的字符串。
// 4. 语义清晰：在复杂的数据结构中（如你展示的 RespFrame 枚举），使用 SimpleString 而不是直接使用 String 可以使代码的意图更加明确。
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// pub struct SimpleString(pub(crate) String); // Simple String, 用于存储简单字符串
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// pub struct SimpleError(pub(crate) String);
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// pub struct BulkString(pub(crate) Vec<u8>); // 单个二进制字符串, 用于存储二进制数据(最大512MB)
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// pub struct RespNullBulkString;
// #[derive(Debug, Clone, PartialEq)]
// pub struct RespArray(pub(crate) Vec<RespFrame>);
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// pub struct RespNullArray;
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// pub struct RespNull;
// // 改为 BTreeMap, 用于有序的 key-value 数据
// #[derive(Default, Clone, Debug, PartialEq)]
// pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);
// // pub struct RespSet(HashSet<RespFrame>);
// #[derive(Debug, Clone, PartialEq)]
// pub struct RespSet(pub(crate) Vec<RespFrame>); // 改为 Vec, 用于有序的集合数据
// endregion: --- before refactor
// endregion: --- Enum and Structs

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

/// The function `find_crlf` searches for the nth occurrence of the CRLF sequence (carriage return
/// followed by line feed) in a byte slice.
///
/// Arguments:
///
/// * `buf`: The `buf` parameter is a slice of bytes (`&[u8]`) that represents the data in which you
///   want to find the occurrence of the CRLF sequence (carriage return followed by line feed).
/// * `nth`: The `nth` parameter in the `find_crlf` function represents the occurrence of the CRLF
///   sequence (carriage return followed by line feed) that you want to find within the given byte buffer
///   `buf`. It specifies which occurrence of the CRLF sequence you are interested in locating within the
///
/// Returns:
///
/// The function `find_crlf` returns an `Option<usize>`. It returns `Some(index)` if the nth occurrence
/// of the CRLF sequence (b'\r\n') is found in the input buffer `buf`, where `index` is the index of the
/// start of the CRLF sequence. If the nth occurrence is not found, it returns `None`.
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

/// The function `parse_length` parses the length of a frame data from a buffer using a specified
/// prefix.
///
/// Arguments:
///
/// * `buf`: The `buf` parameter is a slice of bytes (`&[u8]`) that contains the data to be parsed.
/// * `prefix`: The `prefix` parameter is a string slice (`&str`) that represents the prefix used to
///   extract data from the buffer `buf`. It is used to identify the starting point for extracting the
///   data from the buffer.
///
/// Returns:
///
/// The function `parse_length` is returning a `Result` containing a tuple. The tuple contains two
/// values: the first value is the end index of the extracted data, and the second value is the parsed
/// integer value from the extracted data.
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

// enum_dispatch
// 这里因为 enum_dispatch 的原因, 会自动为变体类型生成, From<xxx> for Enum_Name
// 因此当构造出变体类型的时候, 可以使用 into 方法将其转换为枚举类型, 或者 from
// 因为实现了 from 会自动实现 into, 实现了 into 会自动实现 from
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_calc_array_length() -> Result<()> {
        let buf = b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n";
        let (end, len) = parse_length(buf, "*")?;
        let total_len = calc_total_length(buf, end, len, "*")?;
        assert_eq!(total_len, buf.len());

        let buf = b"*2\r\n$3\r\nset\r\n";
        let (end, len) = parse_length(buf, "*")?;
        let ret = calc_total_length(buf, end, len, "*");
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        Ok(())
    }
}
