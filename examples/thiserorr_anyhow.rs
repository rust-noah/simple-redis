use anyhow::{Context, Result};
use std::io::Error as IoError;
use std::num::ParseIntError;
use thiserror::Error;

// 定义自定义错误类型
#[derive(Error, Debug)]
enum MyError {
    #[error("An IO error occurred: {0}")]
    Io(#[from] IoError),

    #[error("A parsing error occurred: {0}")]
    Parse(#[from] ParseIntError),

    #[error("Custom error: {0}")]
    Custom(String),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

// 一个可能返回错误的函数
fn parse_number(input: &str) -> Result<i32, MyError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(MyError::Custom("Input is empty".into()));
    }

    let number: i32 = trimmed
        .parse()
        // .map_err(|e| MyError::Parse(e))
        .map_err(MyError::Parse) // 更好的写法
        .context("Failed to parse number")?;
    Ok(number)
}

fn main() -> Result<(), MyError> {
    // 示例一: 正确的输入
    match parse_number("42") {
        Ok(number) => println!("Parsed number: {}", number),
        Err(e) => eprintln!("Error: {}", e),
    }

    // 示例二: 空输入
    match parse_number("") {
        Ok(number) => println!("Parsed number: {}", number),
        Err(e) => eprintln!("Error: {}", e),
    }

    // 示例三: 无效输入
    match parse_number("abc") {
        Ok(number) => println!("Parsed number: {}", number),
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}
