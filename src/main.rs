#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use anyhow::Result;
use std::env;
use std::{error::Error, net::SocketAddr};
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use crate::packets::HandshakePacket;
use crate::types::write_string;

mod packets;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    // Initialize TCP Socket & Stream
    let mut listener = TcpListener::bind("127.0.0.1:25565").await?;
    loop {
        let client = accept_client(&mut listener).await;
        let (stream, addr) = client?;
        tokio::spawn(async move {
            handle_client(stream, &addr)
                .await
                .expect("An error happened whilst handling client connection");
        });
    }
}

#[derive(Debug, Default)]
struct Connection {
    state: packets::State,
    compression: bool,
}

async fn handle_client(s: TcpStream, addr: &SocketAddr) -> Result<()> {
    let uuid = "abcd1234";
    info!("[{}] Client connected from addr {}", uuid, addr);
    let mut s = BufReader::new(s);
    let mut conn = Connection::default();
    loop {
        let head: packets::Packet = packets::Packet::read_uncompressed(&mut s).await?;
        match conn.state {
            packets::State::Handshake => {
                let handshake = HandshakePacket::read(&mut s).await?;
                conn.state = handshake.next;
            }
            packets::State::Status => {
                match head.id {
                    // Status request
                    0x00 => {
                        write_string(
                            &mut s,
                            r#"{
                                "version": {
                                    "name": "1.19",
                                    "protocol": 759
                                },
                                "players": {
                                    "max": 100,
                                    "online": 5,
                                    "sample": [
                                        {
                                            "name": "thinkofdeath",
                                            "id": "4566e69f-c907-48ee-8d71-d7ba5aa00d20"
                                        }
                                    ]
                                },
                                "description": {
                                    "text": "Hello world"
                                },
                                "favicon": "data:image/png;base64,<data>",
                                "previewsChat": true
                            }"#,
                        )
                        .await?;
                    }
                    // Ping packet
                    0x01 => {
                        let val = s.read_u64().await?;
                    }
                    _ => warn!("client sent invalid id: {}", head.id),
                }
            }
            _ => todo!("State not yet implemented"),
        }
    }
}

async fn accept_client(
    listener: &mut TcpListener,
) -> Result<(TcpStream, SocketAddr), Box<dyn Error>> {
    let client = listener.accept().await?;
    client.0.set_nodelay(true)?;
    Ok(client)
}
