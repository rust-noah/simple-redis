use crate::{Backend, RespArray, RespFrame};

use super::{extract_args, validate_command, CommandError, CommandExecutor};

// echo: https://redis.io/docs/latest/commands/echo/

#[derive(Debug)]
pub struct Echo {
    message: String,
}

impl CommandExecutor for Echo {
    fn execute(self, _backend: &Backend) -> RespFrame {
        RespFrame::BulkString(self.message.into())
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["echo"], 1)?; // validate get

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(message)) => Ok(Echo {
                message: String::from_utf8(message.0)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid message".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BulkString, RespDecode};

    use super::*;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_echo_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$4\r\necho\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: Echo = frame.try_into()?;
        assert_eq!(result.message, "hello");

        Ok(())
    }

    #[test]
    fn test_echo_command() -> Result<()> {
        // let backend = Backend::new();
        // let cmd = Echo {
        //     message: "hello world".to_string(),
        // };
        // let result = cmd.execute(&backend);
        // assert_eq!(result, RespFrame::BulkString(b"hello world".into()));

        // Ok(())

        let command = Echo::try_from(RespArray::new([
            BulkString::new("echo").into(),
            BulkString::new("hello").into(),
        ]))?;
        assert_eq!(command.message, "hello");

        let backend = Backend::new();
        let result = command.execute(&backend);
        assert_eq!(result, RespFrame::BulkString(b"hello".into()));
        Ok(())
    }
}
