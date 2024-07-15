use anyhow::Result;
use bytes::BytesMut;
fn main() -> Result<()> {
    let a = "hello"; // a.as_bytes() = b"hello"

    // bytes_mut
    let mut bytes_mut = BytesMut::new();
    bytes_mut.extend_from_slice(a.as_bytes());
    println!("bytes_mut: {:?}", bytes_mut);

    let b = bytes_mut.split_to(3);
    println!("b: {:?}", b);
    println!("after split_to(3) -> bytes_mut: {:?}", bytes_mut);
    Ok(())
}
