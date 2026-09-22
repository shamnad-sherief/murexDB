use murex_protocol::{
    Command::{Delete, Get, Help, Ping, Set},
    Response::{self},
    read_command, write_response,
};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::Mutex};

use crate::{db::Database, wal::WalWriter};

pub async fn handle_client(
    mut stream: TcpStream,
    db: Database,
    wal_writer: Arc<Mutex<WalWriter>>,
) -> murex_common::Result<()> {
    let (mut reader, mut writer) = stream.split();

    loop {
        // read the command
        let cmd = match read_command(&mut reader).await {
            Ok(cmd) => cmd,
            Err(murex_common::MurexError::IOError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                // client closed socket (EOF) , exit the loop
                break;
            }
            Err(e) => {
                let err_resp = Response::Error(format!("Invalid argument frame: {}", e));

                let _ = write_response(&mut writer, &err_resp).await;
                break;
            }
        };

        // execute the command operation on db and return response

        let response = match cmd {
            Ping(msg) => Response::Ok(msg),
            Get(key) => match db.get(&key).await {
                Some(val) => Response::Ok(Some(val)),
                None => Response::NotFound,
            },
            Set(key, item) => {

                // write it to WAL first
               let mut wal_guard = wal_writer.lock().await;

               if let Err(e) =  wal_guard.append_set(&key, &item){
                let err_resp = Response::Error(format!("WAL write failed {}", e));
                write_response(&mut writer, &err_resp).await?;
                continue;
               }

               // only after successfull WAL, mutate the in memory db
                db.set(key, item).await;
                Response::Ok(None)
            }
            Delete(key) => {

                if db.get(&key).await.is_none(){
                    Response::NotFound
                }else {
                // write it to WAL first
                let mut wal_guard = wal_writer.lock().await;
                if let Err(e) = wal_guard.append_delete(&key){
                     let err_resp = Response::Error(format!("WAL write failed {}", e));
                     write_response(&mut writer, &err_resp).await?;
                     continue;
                }
                db.delete(&key).await;
                Response::Ok(None)
                }
            }
            Help => Response::Help(
                "MurexDB Commands:\n  PING [msg]\n  GET <key>\n  SET <key> <val>\n  DELETE <key>\n  HELP".into()
            ),
        };

        // send response frame back to client

        if let Err(e) = write_response(&mut writer, &response).await {
            eprint!("Failed to send response: {}", e);
            break;
        }
    }

    Ok(())
}
