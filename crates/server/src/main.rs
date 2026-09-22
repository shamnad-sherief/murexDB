use murex_server::snapshot::load_snapshot;
use murex_server::wal::{WalReader, WalWriter};
use murex_server::{handle_client, snapshot::save_snapshot};
use std::fs::File;
use std::sync::Arc;
use std::{env, net::SocketAddr};
use tokio::{net::TcpListener, sync::Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr_str =
        env::var("MUREX_SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1:6739".to_string());

    let addr: SocketAddr = addr_str.parse()?;

    let listener = TcpListener::bind(addr).await?;
    println!("MurexDB Server listening on {}", addr);

    let db_path = env::var("MUREX_DB_PATH").unwrap_or_else(|_| "data.db".to_string());
    let db = load_snapshot(&db_path).await?;

    let wal_path = env::var("MUREX_WAL_PATH").unwrap_or_else(|_| "wal.log".to_string());

    let mut next_lsn = 0;

    if std::path::Path::new(&wal_path).exists() {
        let file = File::open(&wal_path)?;

        // check the file has atleast 8-byte header

        if file.metadata()?.len() >= 8 {
            let mut reader = WalReader::new(file)?;

            // Replay log to recover state
            let count = reader.replay_into(&db).await?;
            println!("Replayed {} operations from WAL", count);

            if let Some(last) = reader.last_lsn {
                next_lsn = last + 1;
            }
        }
    }

    // 2. Open writer and resume from last_lsn + 1 (or 0 if log was empty)
    let mut writer = WalWriter::open(&wal_path)?;
    writer.next_lsn = next_lsn;

    let wal_writer: Arc<Mutex<WalWriter>> = Arc::new(Mutex::new(writer));

    println!("Database state loaded from {}", db_path);

    tokio::select! {
        res = async {
            loop {
                let (socket, peer_addr) = listener.accept().await?;
                println!("Accepted client connection from {}", peer_addr);

                let db_clone = db.clone();
                let wal_writer = wal_writer.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_client(socket, db_clone, wal_writer).await {
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
            println!("Shutdown signal recieved. Svaing snapshot to {}", db_path);
            if let Err(e) =
            save_snapshot(&db, &db_path).await{
                eprintln!("Failed to save snapshot: {}", e)

            }else{
                println!("Snapshot saved successfully!");
                let mut wal_guard = wal_writer.lock().await;
                *wal_guard =  WalWriter::checkpoint(&wal_path)?;
            }
        },
    };

    Ok(())
}
