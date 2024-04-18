use std::{net::SocketAddr, io::{Write, Read}};

use base64::{prelude::BASE64_STANDARD, Engine};
use clap::Args;
use image::{codecs::bmp::BmpEncoder, ImageEncoder};

use crate::{protocol::Connection, clipboard::{Image, Content}};

#[derive(Args, Debug)]
struct ClientCli {
    #[arg(long, default_value = "127.0.0.1:14573")]
    target: SocketAddr,
}

#[derive(Args, Debug)]
pub struct PasteCli {
    #[command(flatten)]
    client: ClientCli,

    /// Use iTerm2 escape codes for image data
    #[arg(long)]
    iterm2: bool,
}

#[derive(Args, Debug)]
pub struct CopyCli {
    #[command(flatten)]
    client: ClientCli,
}

fn is_tmux() -> bool {
    std::env::var("TERM").map_or(false, |term| term.starts_with("tmux") || term.starts_with("screen"))
}

fn print_image(args: &PasteCli, image: Image) {
    let mut bmp = Vec::new();
    BmpEncoder::new(&mut bmp)
        .write_image(&image.bytes, image.width as u32, image.height as u32, image::ExtendedColorType::Rgba8)
        .unwrap();

    let mut stdout = std::io::stdout().lock();
    let is_tmux = is_tmux();

    if args.iterm2 {
        if is_tmux {
            stdout.write("\x1bPtmux;\x1b\x1b]".as_bytes()).unwrap();
        } else {
            stdout.write("\x1b]".as_bytes()).unwrap();
        }
        writeln!(&mut stdout, "1337;File=size={};inline=1:", bmp.len()).unwrap();
    }

    stdout.write_all(BASE64_STANDARD.encode(&bmp).as_bytes()).unwrap();

    if args.iterm2 {
        if is_tmux {
            stdout.write("\x07\x1b\\".as_bytes()).unwrap();
        } else {
            stdout.write("\x07".as_bytes()).unwrap();
        }
    }
}

pub fn paste(args: PasteCli) -> anyhow::Result<()> { 
    let stream = std::net::TcpStream::connect(args.client.target).unwrap();
    let mut conn = Connection::new(&stream, &stream);

    conn.send_command("PASTE")?;
    match conn.read_command()?.as_str() {
        "COPY TEXT" => {
            let text = conn.read_string()?;
            println!("{}", text);
        },
        "COPY IMAGE" => {
            let image = conn.read_image()?;
            print_image(&args, image);
        },
        response => {
            return Err(anyhow::anyhow!("Unexpected response: {}", response));
        }
    }

    Ok(())
}

pub fn copy(args: CopyCli) -> anyhow::Result<()> {
    let stream = std::net::TcpStream::connect(args.client.target).unwrap();
    let mut conn = Connection::new(&stream, &stream);

    let mut content = String::new();
    std::io::stdin().read_to_string(&mut content)?;
    conn.send_clipboard(&Some(Content::Text(content)))?;

    Ok(())
}
