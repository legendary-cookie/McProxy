use anyhow::Result;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn write_varint<T>(s: &mut T, num: i32) -> Result<()>
where
    T: AsyncWrite + std::marker::Unpin,
{
    let mut value = num;
    loop {
        let mut temp: i16 = (value & 0b0111_1111) as i16;
        value >>= 7;
        if value != 0 {
            temp |= 0b1000_0000;
        }
        s.write_i8(temp as i8).await?;
        if value == 0 {
            break Ok(());
        }
    }
}

pub async fn read_varint<T>(stream: &mut T) -> Result<i32>
where
    T: AsyncRead + std::marker::Unpin,
{
    let mut result: i32 = 0;
    let mut num_read: i32 = 0;
    loop {
        let read = stream.read_u8().await? as i32;
        let value = read & 0b0111_1111;
        result |= value << (7 * num_read);
        num_read += 1;
        if num_read > 5 {
            return Err(anyhow!("VarInt too big!"));
        }
        if (read & 0b1000_0000) == 0 {
            break;
        }
    }
    Ok(result)
}
