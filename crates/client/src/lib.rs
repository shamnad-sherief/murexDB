use murex_protocol::{Command, Response, read_response, write_command};
use tokio::net::TcpStream;

pub struct MurexClient {
    stream: TcpStream,
}

impl MurexClient {
    // connect to murex server
    pub async fn connect(addr: &str) -> murex_common::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }

    pub async fn send(&mut self, cmd: Command) -> murex_common::Result<Response> {
        write_command(&mut self.stream, &cmd).await?;
        read_response(&mut self.stream).await
    }
}
