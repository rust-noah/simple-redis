use std::sync::LazyLock;

use command::CommandError;
use enum_dispatch::enum_dispatch;

use crate::{Backend, RespArray, RespFrame, SimpleString};

mod command;
mod echo;
mod hmap;
mod map;
mod set;
mod unrecognized;

pub use {
    command::Command,
    echo::Echo,
    hmap::{HGet, HGetAll, HMGet, HSet},
    map::{Get, Set},
    set::{SAdd, SIsMember},
    unrecognized::Unrecognized,
};

// NOTE: you could also use once_cell instead of lazy_static
// lazy_static:
// 1. init in runtime
// 2. thread safe
// 3. improve performance
// lazy_static! {
//     static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
// }

// NOTE: > Rust 1.80.0
// https://blog.rust-lang.org/2024/07/25/Rust-1.80.0.html
static RESP_OK: LazyLock<RespFrame> = LazyLock::new(|| SimpleString::new("OK").into());

#[enum_dispatch]
pub trait CommandExecutor {
    // fn execute(&self) -> RespFrame;
    fn execute(self, backend: &Backend) -> RespFrame;
}

fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if n_args != usize::MAX && value.len() != n_args + names.len() {
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
