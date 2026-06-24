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
        async { self.seek(SeekFrom::Current(0)).await }
    }
    fn seek_start(&mut self) -> impl Future<Output = std::io::Result<()>> + Send
    where
        Self: Send,
    {
        self.set_position(0)
    }
    fn seek_end(&mut self) -> impl Future<Output = std::io::Result<u64>> + Send {
        self.seek(SeekFrom::End(0))
    }
    fn length(&mut self) -> impl Future<Output = std::io::Result<u64>> + Send
    where
        Self: Send,
    {
        async move {
            let pos = self.position().await?;
            let length = self.seek(SeekFrom::End(0)).await?;
            self.seek(SeekFrom::Start(pos)).await?;
            Ok(length)
        }
    }
    fn position(&mut self) -> impl Future<Output = std::io::Result<u64>> + Send
    where
        Self: Send,
    {
        self.stream_position()
    }
    fn set_position(&mut self, pos: u64) -> impl Future<Output = std::io::Result<()>> + Send
    where
        Self: Send,
    {
        async move {
            self.seek(SeekFrom::Start(pos)).await?;
            Ok(())
        }
    }
}
impl Seek for std::fs::File {
    fn seek(&mut self, pos: SeekFrom) -> impl Future<Output = std::io::Result<u64>> + Send {
        async move { std::io::Seek::seek(self, pos) }
    }
}

impl<T: AsRef<[u8]> + Send> Seek for Cursor<T>
where
    Cursor<T>: std::io::Seek,
{
    async fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        std::io::Seek::seek(self, pos)
    }
}

impl<R: Seek + ?Sized + Send> Seek for &mut R {
    async fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        (**self).seek(pos).await
    }
}
