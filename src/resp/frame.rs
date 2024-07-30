use bytes::BytesMut;
use enum_dispatch::enum_dispatch;

use super::{
    BulkString, RespArray, RespDecode, RespError, RespMap, RespNull, RespSet, SimpleError,
    SimpleString,
};

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
    // NullBulkString(RespNullBulkString),
    Array(RespArray),
    // NullArray(RespNullArray),
    Null(RespNull),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}
// NOTE: 这里需要 impl RespDecode, RespEncode 不需要是因为使用 enum_dispatch 宏的时候, 会自动实现这些 trait
// RespDecode 不能使用 enum_dispatch, 因为不支持 trait 中带有 associated type/ const 的情况
impl RespDecode for RespFrame {
    const PREFIX: &'static str = "";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                let frame = SimpleString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'-') => {
                let frame = SimpleError::decode(buf)?;
                Ok(frame.into())
            }
            Some(b':') => {
                let frame = i64::decode(buf)?;
                Ok(frame.into())
            }
            // NOTE: refactor -> delete NullBulkString and NullArray
            // Some(b'$') => {
            //     // try null bulk string first
            //     match RespNullBulkString::decode(buf) {
            //         Ok(frame) => Ok(frame.into()),
            //         Err(RespError::NotComplete) => Err(RespError::NotComplete),
            //         Err(_) => {
            //             let frame = BulkString::decode(buf)?;
            //             Ok(frame.into())
            //         }
            //     }
            // }
            // Some(b'*') => {
            //     // try null array first
            //     match RespNullArray::decode(buf) {
            //         Ok(frame) => Ok(frame.into()),
            //         Err(RespError::NotComplete) => Err(RespError::NotComplete),
            //         Err(_) => {
            //             let frame = RespArray::decode(buf)?;
            //             Ok(frame.into())
            //         }
            //     }
            // }
            Some(b'$') => {
                let frame = BulkString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'*') => {
                let frame = RespArray::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'_') => {
                let frame = RespNull::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'#') => {
                let frame = bool::decode(buf)?;
                Ok(frame.into())
            }
            Some(b',') => {
                let frame = f64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'%') => {
                let frame = RespMap::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'~') => {
                let frame = RespSet::decode(buf)?;
                Ok(frame.into())
            }
            None => Err(RespError::NotComplete),
            _ => Err(RespError::InvalidFrameType(format!(
                "expect_length: unknown frame type: {:?}",
                buf
            ))),
        }
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'*') => RespArray::expect_length(buf),
            Some(b'~') => RespSet::expect_length(buf),
            Some(b'%') => RespMap::expect_length(buf),
            Some(b'$') => BulkString::expect_length(buf),
            Some(b':') => i64::expect_length(buf),
            Some(b'+') => SimpleString::expect_length(buf),
            Some(b'-') => SimpleError::expect_length(buf),
            Some(b'#') => bool::expect_length(buf),
            Some(b',') => f64::expect_length(buf),
            Some(b'_') => RespNull::expect_length(buf),
            _ => Err(RespError::NotComplete),
        }
    }
}

impl From<&str> for RespFrame {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string()).into()
    }
}

impl From<&[u8]> for RespFrame {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec()).into()
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_resp_frame_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"+OK\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("OK".to_string()).into());

        buf.extend_from_slice(b"-Error message\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, SimpleError::new("Error message".to_string()).into());

        buf.extend_from_slice(b":1000\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, 1000i64.into());

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello").into());

        buf.extend_from_slice(b"$-1\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        // assert_eq!(frame, RespNullBulkString.into());
        assert_eq!(frame, BulkString::new(vec![]).into());

        buf.extend_from_slice(b"*2\r\n$4\r\necho\r\n$5\r\nhello\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespArray::new([
                BulkString::new("echo").into(),
                BulkString::new("hello").into()
            ])
            .into()
        );

        buf.extend_from_slice(b"*-1\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        // assert_eq!(frame, RespNullArray.into());
        assert_eq!(frame, RespArray::new(vec![]).into());

        buf.extend_from_slice(b"_\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, RespNull.into());

        buf.extend_from_slice(b"#t\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, true.into());

        buf.extend_from_slice(b"#f\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, false.into());

        buf.extend_from_slice(b",1.23\r\n");
        let frame = RespFrame::decode(&mut buf)?;
        assert_eq!(frame, 1.23f64.into());

        buf.extend_from_slice(b"%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n");
        let frame = RespMap::decode(&mut buf)?;
        let mut map = RespMap::new();
        map.insert(
            "hello".to_string(),
            BulkString::new(b"world".to_vec()).into(),
        );
        map.insert("foo".to_string(), BulkString::new(b"bar".to_vec()).into());
        assert_eq!(frame, map);

        Ok(())
    }
}
