use crate::{
    cmd::{command::CommandError, extract_args, validate_command, CommandExecutor},
    RespArray, RespFrame,
};

#[derive(Debug)]
pub struct SIsMember {
    key: String,
    member: String,
}

impl CommandExecutor for SIsMember {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let (key, member) = (self.key, self.member);
        let res = backend.sismember(&key, &member);
        (res as i64).into()
    }
}

impl TryFrom<RespArray> for SIsMember {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sismember"], 2)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(member))) => {
                Ok(SIsMember {
                    key: String::from_utf8(key.0)?,
                    member: String::from_utf8(member.0)?,
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or member".to_string(),
            )),
        }
    }
}
