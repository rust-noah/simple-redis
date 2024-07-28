use anyhow::Result;
use simple_redis::{network, Backend};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:6389";
    let listener = TcpListener::bind(addr).await?;
    info!("Simple-Redis-Server is listening on {}", addr);

    let backend = Backend::new();
    loop {
        let (stream, remote_addr) = listener.accept().await?;
        info!("Accepted connection from {}", remote_addr);
        let cloned_backend = backend.clone();
        tokio::spawn(async move {
            // handling of stream
            match network::stream_handler(stream, cloned_backend).await {
                Ok(_) => info!("Connection from {} closed", remote_addr),
                Err(e) => info!("Connection from {} closed with error: {}", remote_addr, e),
            }
        });
    }
}
