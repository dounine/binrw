use std::io::Cursor;
use std::io::SeekFrom;

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> impl Future<Output = std::io::Result<u64>> + Send;

    fn seek_relative(&mut self, pos: i64) -> impl Future<Output = std::io::Result<()>> + Send
    where
        Self: Send,
    {
        async move {
            self.seek(SeekFrom::Current(pos)).await?;
            Ok(())
        }
    }
    fn stream_position(&mut self) -> impl Future<Output = std::io::Result<u64>> + Send
    where
        Self: Send,
    {
        async {
            self.seek(SeekFrom::Current(0)).await
        }
    }
}

impl Seek for Cursor<Vec<u8>> {
    async fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        std::io::Seek::seek(self, pos)
    }
}
