use binrw::BinWriterExt;
use binrw::io::{Read, Seek};
use std::io::{Cursor, SeekFrom};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut data = Cursor::new(Vec::new());
    data.write_le(&1_u32).await?;
    data.seek(SeekFrom::Start(0)).await?;
    let mut bytes = vec![0u8; 4];
    data.read_exact(bytes.as_mut_slice()).await?;
    dbg!(bytes);
    Ok(())
}
