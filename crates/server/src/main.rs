mod db;
mod handler;
use std::{env, net::SocketAddr};

pub use db::Database;
pub use handler::handle_client;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr_str =
        env::var("MUREX_SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1:6739".to_string());

    let addr: SocketAddr = addr_str.parse()?;

    let listener = TcpListener::bind(addr).await?;
    println!("MurexDB Server listening on {}", addr);

    let db = Database::new();

    tokio::select! {
        res = async {
            loop {
                let (socket, peer_addr) = listener.accept().await?;
                println!("Accepted client connection from {}", peer_addr);

                let db_clone = db.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_client(socket, db_clone).await {
                        eprintln!("Error handling client {}: {}", peer_addr, e);
                    }
                    println!("Connection closed: {}", peer_addr);
                });
            }
            #[allow(unreachable_code)]
            Ok::<(), std::io::Error>(())
        } => {

            if let Err(e) = res {
                eprintln!("Listener error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Shutting down server...");
        },
    };

    Ok(())
}
