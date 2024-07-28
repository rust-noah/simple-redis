use anyhow::Result;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::info;

use crate::{
    cmd::{Command, CommandExecutor},
    Backend, RespDecode, RespEncode, RespError, RespFrame,
};

#[derive(Debug)]
struct RedisRequest {
    frame: RespFrame,
    backend: Backend,
}

#[derive(Debug)]
struct RedisResponse {
    frame: RespFrame,
}

#[derive(Debug)]
struct RespFrameCodec;

pub async fn stream_handler(stream: TcpStream, backend: Backend) -> Result<()> {
    // how to get a frame from the stream?
    let mut framed = Framed::new(stream, RespFrameCodec);
    loop {
        match framed.next().await {
            Some(Ok(frame)) => {
                info!("Received frame: {:?}", frame);
                let request = RedisRequest {
                    frame,
                    backend: backend.clone(),
                };
                let response = request_handler(request).await?;
                info!("Sending response: {:?}", response.frame);

                // NOTE: When dealing with a large amount of concurrent data,
                // flushing the sink each time will incur performance overhead.
                // framed.send(response.frame).await?;

                // 使用 feed 方法添加响应
                framed.feed(response.frame).await?;
                // 在合适的时候调用 flush 方法
                framed.flush().await?;
            }
            Some(Err(e)) => return Err(e),
            None => return Ok(()),
        }
    }
}

// NOTE: need a backend to process the frame
// async fn request_handler(request: RespFrame) -> Result<RespFrame> {
//     todo!()
// }
async fn request_handler(request: RedisRequest) -> Result<RedisResponse> {
    let (frame, backend) = (request.frame, request.backend);
    let cmd = Command::try_from(frame)?;
    info!("Executing command: {:?}", cmd);
    let frame = cmd.execute(&backend);
    Ok(RedisResponse { frame })
}

impl Encoder<RespFrame> for RespFrameCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RespFrame, dst: &mut bytes::BytesMut) -> Result<()> {
        let encoded = item.encode();
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}

impl Decoder for RespFrameCodec {
    type Item = RespFrame;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<RespFrame>> {
        match RespFrame::decode(src) {
            Ok(frame) => Ok(Some(frame)),
            Err(RespError::NotComplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
