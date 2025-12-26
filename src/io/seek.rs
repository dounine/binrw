use async_trait::async_trait;
use std::io::Cursor;
use std::io::SeekFrom;

pub trait Seek {
    async fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64>;
    async fn rewind(&mut self) -> std::io::Result<()> {
        self.seek(SeekFrom::Start(0)).await?;
        Ok(())
    }
    async fn seek_relative(&mut self, pos: i64) -> std::io::Result<()> {
        self.seek(SeekFrom::Current(pos)).await?;
        Ok(())
    }
    async fn stream_position(&mut self) -> std::io::Result<u64> {
        self.seek(SeekFrom::Current(0)).await
    }
}

impl Seek for Cursor<Vec<u8>> {
    async fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        std::io::Seek::seek(self, pos)
    }
}
