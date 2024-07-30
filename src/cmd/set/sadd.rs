use crate::{
    cmd::{command::CommandError, extract_args, validate_command, CommandExecutor},
    RespArray, RespFrame,
};

#[derive(Debug)]
pub struct SAdd {
    key: String,
    members: Vec<String>,
}

impl CommandExecutor for SAdd {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let (key, members) = (self.key, self.members);
        let cnt = backend.sadd(key, members);
        // RespFrame::Integer(cnt)
        // RespFrame::BulkString(format!("{}(integer)", cnt).into())
        cnt.into()
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sadd"], usize::MAX)?;
        let args = extract_args(value, 1)?.into_iter();
        let mut data = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                RespFrame::BulkString(s) => {
                    let s = String::from_utf8(s.0)?;
                    data.push(s);
                }
                _ => {
                    return Err(CommandError::InvalidArgument(
                        "Invalid key or member".to_string(),
                    ));
                }
            }
        }
        Ok(SAdd {
            key: data.remove(0),
            members: data,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::RespDecode;

    use super::*;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_sadd_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$4\r\nsadd\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: SAdd = frame.try_into()?;
        assert_eq!(result.key, "set");
        assert_eq!(result.members, vec!["hello", "world"]);

        Ok(())
    }

    #[test]
    fn test_sadd_command() {
        let backend = crate::Backend::new();
        let cmd = SAdd {
            key: "set".to_string(),
            members: vec!["hello".to_string(), "world".to_string()],
        };
        let frame = cmd.execute(&backend);
        assert_eq!(frame, RespFrame::Integer(2));
    }
}
