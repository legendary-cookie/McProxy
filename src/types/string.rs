use std::io::Cursor;

use anyhow::Result;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt, AsyncWrite};

use super::read_varint;

pub async fn read_string<T>(s: &mut T) -> Result<String>
where
    T: AsyncRead + std::marker::Unpin,
{
    let len = read_varint(s).await?;
    let mut buf = vec![0u8; len as usize];
    s.read_exact(&mut buf).await?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

pub async fn write_string<T>(s: &mut T, string: &str) -> Result<()>
where
    T: AsyncWrite + std::marker::Unpin,
{
    let mut temp: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    super::write_varint(&mut temp, 0).await?;
    super::write_varint(&mut temp, string.len() as i32).await?;
    temp.write_all(&string.as_bytes()).await?;
    let temp = temp.into_inner();
    super::write_varint(s, temp.len() as i32).await?;
    s.write_all(&temp).await?;
    Ok(())
}
