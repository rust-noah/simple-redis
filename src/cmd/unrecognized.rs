use crate::{Backend, RespArray, RespFrame};

use super::{command::CommandError, CommandExecutor, RESP_OK};

#[derive(Debug)]
pub struct Unrecognized;

impl CommandExecutor for Unrecognized {
    fn execute(self, _: &Backend) -> RespFrame {
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for Unrecognized {
    type Error = CommandError;
    fn try_from(_value: RespArray) -> Result<Self, Self::Error> {
        Ok(Unrecognized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RespArray, RespDecode};
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_unrecognized_command() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*1\r\n$3\r\nfoo\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: Unrecognized = frame.try_into()?;
        assert_eq!(result.execute(&Backend::new()), RESP_OK.clone());

        Ok(())
    }
}
