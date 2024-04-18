use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
};

use anyhow::Context;
use clap::Args;

use crate::{clipboard, protocol::Connection};

#[derive(Args, Debug)]
pub struct ServerCli {
    #[clap(short, long, default_value = "127.0.0.1:14573")]
    listen: SocketAddr,
}

fn handle_command<R: Read, W: Write>(
    command: &str,
    conn: &mut Connection<R, W>,
) -> anyhow::Result<()> {
    match command {
        "PASTE" => {
            conn.send_clipboard(&clipboard::get()?)?;
        }
        "COPY NONE" => {
            clipboard::clear()?;
        }
        "COPY TEXT" => {
            let text = conn.read_string()?;
            clipboard::set(clipboard::Content::Text(text))?;
        }
        "COPY IMAGE" => {
            let image = conn.read_image()?;
            clipboard::set(clipboard::Content::Image(image))?;
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }

    Ok(())
}

fn handle_client(stream: TcpStream) -> anyhow::Result<()> {
    let mut conn = Connection::new(&stream, &stream);

    loop {
        let command = match conn.read_command() {
            Ok(command) => command,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    return Ok(());
                }
                return Err(e).context("Failed to read command");
            }
        };
        let result = handle_command(&command, &mut conn);
        if let Err(e) = result {
            eprintln!("Error handling command: {}", e);
            conn.send_error(&e.to_string())?;
        }
    }
}

pub fn run(args: ServerCli) -> anyhow::Result<()> {
    let listen = TcpListener::bind(args.listen)
        .with_context(|| format!("Failed to bind to {}", args.listen))?;
    eprintln!("Listening on {}", args.listen);

    for stream in listen.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        };
        let peer = match stream.peer_addr() {
            Ok(endpoint) => endpoint,
            Err(e) => {
                eprintln!("Failed to get peer address: {}", e);
                continue;
            }
        };

        thread::spawn(move || {
            println!("Connection from: {}", peer);
            if let Err(e) = handle_client(stream) {
                eprintln!("Error on connection {}: {:?}", peer, e);
            } else {
                eprintln!("Connection closed: {}", peer);
            }
        });
    }

    Ok(())
}
