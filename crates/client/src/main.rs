use std::{
    env,
    io::{self, Write},
};

use murex_client::MurexClient;
use murex_protocol::{Command, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr =
        env::var("MUREX_SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1:6739".to_string());

    let args: Vec<String> = env::args().collect();

    let mut client = match MurexClient::connect(&server_addr).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            std::process::exit(1);
        }
    };

    // if command is provided in args then execute and exit
    if args.len() > 1 {
        let cmd_str = args[1..].join(" ");
        if let Some(cmd) = parse_line(&cmd_str) {
            match client.send(cmd).await {
                Ok(resp) => print_response(&resp),
                Err(e) => eprintln!("Failed to send command: {}", e),
            }
        }
        return Ok(());
    }

    // otherwise enter interactive mode
    println!(
        "Connected to MurexDB Server at {}. Type 'HELP' for list of commands or 'exit' to quit.",
        server_addr
    );

    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("murex> ");
        io::stdout().flush()?;
        input.clear();

        if stdin.read_line(&mut input)? == 0 {
            // EOF reached, exit the loop
            break;
        }

        let cmd_str = input.trim();
        if cmd_str.is_empty() {
            continue;
        }

        if cmd_str.eq_ignore_ascii_case("exit") {
            break;
        }

        if let Some(cmd) = parse_line(cmd_str) {
            match client.send(cmd).await {
                Ok(resp) => print_response(&resp),
                Err(e) => eprintln!("Failed to send command: {}", e),
            }
        }
    }

    Ok(())
}

fn parse_line(line: &str) -> Option<Command> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    match parts[0].to_uppercase().as_str() {
        "PING" => {
            if parts.len() > 1 {
                Some(Command::Ping(Some(parts[1..].join(" ").into_bytes())))
            } else {
                Some(Command::Ping(None))
            }
        }
        "GET" => {
            if parts.len() < 2 {
                eprintln!("Usage: GET <key>");
                None
            } else {
                Some(Command::Get(parts[1].as_bytes().to_vec()))
            }
        }
        "SET" => {
            if parts.len() < 3 {
                eprintln!("Usage: SET <key> <value>");
                None
            } else {
                let key = parts[1].as_bytes().to_vec();
                let val = parts[2..].join(" ").into_bytes();
                Some(Command::Set(key, val))
            }
        }
        "DELETE" | "DEL" => {
            if parts.len() < 2 {
                eprintln!("Usage: DELETE <key>");
                None
            } else {
                Some(Command::Delete(parts[1].as_bytes().to_vec()))
            }
        }
        "HELP" => Some(Command::Help),
        other => {
            eprintln!("Unknown command: {}", other);
            None
        }
    }
}

fn print_response(resp: &Response) {
    match resp {
        Response::Ok(Some(val)) => {
            let str_val = String::from_utf8_lossy(val);
            println!("OK: \"{}\"", str_val);
        }
        Response::Ok(None) => println!("OK"),
        Response::NotFound => println!("(nil)"),
        Response::Error(err) => println!("(error) {}", err),
        Response::Help(text) => println!("{}", text),
    }
}
