use murex_protocol::{
    Command::{Delete, Get, Help, Ping, Set},
    Response::{self},
    read_command, write_response,
};
use tokio::net::TcpStream;

use crate::db::Database;

pub async fn handle_client(mut stream: TcpStream, db: Database) -> murex_common::Result<()> {
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
                db.set(key, item).await;
                Response::Ok(None)
            }
            Delete(key) => {
                if db.delete(&key).await {
                    Response::Ok(None)
                } else {
                    Response::NotFound
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
