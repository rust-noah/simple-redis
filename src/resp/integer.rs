use bytes::BytesMut;

use super::{extract_simple_frame_data, RespDecode, RespEncode, RespError, CRLF_LEN};

// - integer: ":[<+|->]<value>\r\n"
// NOTE: 实际测试正数不需要+号，负数需要-号
impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        // let sign = if self < 0 { "" } else { "+" }; // -1 => -1, 1 => +1
        // format!(":{}{}\r\n", sign, self).into_bytes()
        format!(":{}\r\n", self).into_bytes()
    }
}

// - integer: ":[<+|->]<value>\r\n"
impl RespDecode for i64 {
    const PREFIX: &'static str = ":";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        // split the buffer
        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);
        Ok(s.parse()?)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespFrame;
    use anyhow::Result;

    #[test]
    fn test_integer_encode() {
        let frame: RespFrame = 1.into();
        assert_eq!(frame.encode(), b":1\r\n");

        let frame: RespFrame = 123.into();
        assert_eq!(frame.encode(), b":123\r\n");

        let frame: RespFrame = (-123).into();
        assert_eq!(frame.encode(), b":-123\r\n");
    }

    #[test]
    fn test_integer_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b":123\r\n");

        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, 123);

        buf.extend_from_slice(b":-123\r\n");

        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, -123);

        Ok(())
    }
}
