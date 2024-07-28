use std::sync::LazyLock;

use enum_dispatch::enum_dispatch;
use thiserror::Error;

use crate::{Backend, RespArray, RespError, RespFrame, SimpleString};

mod hmap;
mod map;

// you could also use once_cell instead of lazy_static
// lazy_static:
// 1. init in runtime
// 2. thread safe
// 3. improve performance
// lazy_static! {
//     static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
// }

// > Rust 1.80.0
// https://blog.rust-lang.org/2024/07/25/Rust-1.80.0.html
static RESP_OK: LazyLock<RespFrame> = LazyLock::new(|| SimpleString::new("OK").into());

// region:    --- Traits
#[enum_dispatch]
pub trait CommandExecutor {
    // fn execute(&self) -> RespFrame;
    fn execute(self, backend: &Backend) -> RespFrame;
}
// endregion: --- Traits

// region:    --- Enum and Structs
#[enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
    HGetAll(HGetAll),
    // unrecognized command
    Unrecognized(Unrecognized),
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("{0}")]
    RespError(#[from] RespError),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[derive(Debug)]
pub struct Get {
    key: String,
}

#[derive(Debug)]
pub struct Set {
    key: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct HGet {
    key: String,
    field: String,
}

#[derive(Debug)]
pub struct HSet {
    key: String,
    field: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct HGetAll {
    key: String,
    sort: bool,
}

#[derive(Debug)]
pub struct Unrecognized;
// endregion: --- Enum and Structs

// region:    --- impls
impl CommandExecutor for Unrecognized {
    fn execute(self, _: &Backend) -> RespFrame {
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(v: RespArray) -> Result<Self, Self::Error> {
        match v.first() {
            Some(RespFrame::BulkString(ref cmd)) => match cmd.as_ref() {
                b"get" => Ok(Get::try_from(v)?.into()),
                b"set" => Ok(Set::try_from(v)?.into()),
                b"hget" => Ok(HGet::try_from(v)?.into()),
                b"hset" => Ok(HSet::try_from(v)?.into()),
                b"hgetall" => Ok(HGetAll::try_from(v)?.into()),
                _ => Ok(Unrecognized.into()),
            },
            _ => Err(CommandError::InvalidCommand(
                "Command must have a BulkString as the first argument".to_string(),
            )),
        }
    }
}

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;
    fn try_from(v: RespFrame) -> Result<Self, Self::Error> {
        match v {
            RespFrame::Array(array) => array.try_into(),
            _ => Err(CommandError::InvalidCommand(
                "Command must be an Array".to_string(),
            )),
        }
    }
}
// endregion: --- impls

// region:    --- functions
fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() != n_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have exactly {} argument",
            names.join(" "),
            n_args
        )));
    }

    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if cmd.as_ref().to_ascii_lowercase() != name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: expected {}, got {}",
                        name,
                        String::from_utf8_lossy(cmd.as_ref())
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(
                    "Command must have a BulkString as the first argument".to_string(),
                ))
            }
        }
    }
    Ok(())
}

fn extract_args(value: RespArray, start: usize) -> Result<Vec<RespFrame>, CommandError> {
    Ok(value.0.into_iter().skip(start).collect::<Vec<RespFrame>>())
}
// endregion: --- functions
