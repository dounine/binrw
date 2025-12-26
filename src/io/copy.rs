use crate::io::read::Read;
use crate::io::write::Write;

pub async fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> std::io::Result<u64>
where
    R: Read,
    W: Write,
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