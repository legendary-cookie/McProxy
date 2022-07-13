use crate::types::*;
use anyhow::Result;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug)]
pub struct Packet {
    pub size: i32,
    pub id: i32,
}

impl Packet {
    pub async fn read_uncompressed<T>(s: &mut T) -> Result<Self>
    where
        T: AsyncRead + std::marker::Unpin,
    {
        let size = read_varint(s).await?;
        let id = read_varint(s).await?;
        Ok(Self { size, id })
    }
}

#[derive(Debug)]
pub struct HandshakePacket {
    pub version: i32,
    pub host: String,
    pub port: u16,
    pub next: State, // the next state
}

#[derive(Debug, PartialEq, Eq, Default)]
pub enum State {
    #[default]
    Handshake,
    Status,
    Login,
}

impl State {
    fn from_i32(state: i32) -> Result<Self> {
        match state {
            1 => return Ok(Self::Status),
            2 => return Ok(Self::Login),
            _ => return Err(anyhow!("Invalid state")),
        }
    }
}

impl HandshakePacket {
    pub async fn read<T>(s: &mut T) -> Result<Self>
    where
        T: AsyncRead + std::marker::Unpin + std::marker::Send,
    {
        let version = read_varint(s).await?;
        let host = read_string(s).await?;
        let port = s.read_u16().await?;
        let next = State::from_i32(read_varint(s).await?)?;
        Ok(Self {
            version,
            host,
            port,
            next,
        })
    }
}
