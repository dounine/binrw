use std::pin::Pin;

use crate::io::read::Read;
use crate::io::write::Write;

pub async fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> std::io::Result<u64>
where
    R: Read + Send,
    W: Write + Send,
{
    let mut pos = 0;
    let mut buf = [0u8; 8192];
    loop {
        let len = reader.read(&mut buf).await?;
        if len == 0 {
            break;
        }
        writer.write_all(&buf[..len]).await?;
        pos += len as u64;
    }
    Ok(pos)
}
pub type ReadBytesCallback<'a> =
    dyn FnMut(u64) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + 'a;
pub async fn copy_callback<'f, R: ?Sized, W: ?Sized>(
    reader: &mut R,
    writer: &mut W,
    callback: &mut ReadBytesCallback<'f>,
) -> std::io::Result<u64>
where
    R: Read + Send,
    W: Write + Send,
{
    let mut pos = 0;
    let mut buf = [0u8; 8192];
    loop {
        let len = reader.read(&mut buf).await?;
        if len == 0 {
            break;
        }
        writer.write_all(&buf[..len]).await?;
        callback(len as u64).await;
        pos += len as u64;
    }
    Ok(pos)
}
