use murex_protocol::{Command, Response, read_response, write_command};
use murex_server::{Database, handle_client};
use tokio::net::{TcpListener, TcpStream};

async fn setup_test_server() -> String {
    // Bind to port 0 to let OS pick an available free port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let db = Database::new();

    tokio::spawn(async move {
        loop {
            if let Ok((socket, _)) = listener.accept().await {
                let db_clone = db.clone();
                tokio::spawn(async move {
                    let _ = handle_client(socket, db_clone).await;
                });
            }
        }
    });

    addr
}

#[tokio::test]
async fn test_server_set_get_delete() {
    let addr = setup_test_server().await;

    let mut client1 = TcpStream::connect(&addr).await.unwrap();
    let mut client2 = TcpStream::connect(&addr).await.unwrap();

    // 1. SET foo = bar via client1
    let set_cmd = Command::Set(b"foo".to_vec(), b"bar".to_vec());
    write_command(&mut client1, &set_cmd).await.unwrap();
    let resp = read_response(&mut client1).await.unwrap();
    assert_eq!(resp, Response::Ok(None));

    // 2. GET foo via client2 (verifying shared state between clients!)
    let get_cmd = Command::Get(b"foo".to_vec());
    write_command(&mut client2, &get_cmd).await.unwrap();
    let resp = read_response(&mut client2).await.unwrap();
    assert_eq!(resp, Response::Ok(Some(b"bar".to_vec())));

    // 3. DELETE foo via client1
    let del_cmd = Command::Delete(b"foo".to_vec());
    write_command(&mut client1, &del_cmd).await.unwrap();
    let resp = read_response(&mut client1).await.unwrap();
    assert_eq!(resp, Response::Ok(None));

    // 4. GET foo again via client2 -> expect NotFound
    write_command(&mut client2, &get_cmd).await.unwrap();
    let resp = read_response(&mut client2).await.unwrap();
    assert_eq!(resp, Response::NotFound);
}
