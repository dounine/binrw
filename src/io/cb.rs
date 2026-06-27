use std::pin::Pin;

use crate::{
    BinResult,
    io::{Read, Seek, Write},
};
pub trait Callback {
    fn callback(&mut self, bytes: u64) -> impl Future<Output = std::io::Result<()>> + Send;
}
pub struct ReadCallback<R, C>(R, C);
impl<R, C> ReadCallback<R, C> {
    pub fn new(r: R, c: C) -> Self {
        Self(r, c)
    }
}

impl<R, C> Read for ReadCallback<R, C>
where
    R: Read + Send,
    C: FnMut(u64) -> Pin<Box<dyn Future<Output = BinResult<()>> + Send>> + Send,
{
    fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = std::io::Result<usize>> + Send {
        async move {
            let result = self.0.read(buf).await?;
            (self.1)(result as u64)
                .await
                .map(|_| result)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }

    fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send {
        self.0.flush()
    }
}
impl<R, C> Callback for ReadCallback<R, C>
where
    R: Read + Send,
    C: FnMut(u64) -> Pin<Box<dyn Future<Output = BinResult<()>> + Send>> + Send,
{
    fn callback(&mut self, bytes: u64) -> impl Future<Output = std::io::Result<()>> + Send {
        async move {
            (self.1)(bytes)
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }
}
impl<R, C> Seek for ReadCallback<R, C>
where
    R: Seek,
    C: FnMut(u64) -> Pin<Box<dyn Future<Output = BinResult<()>> + Send>> + Send,
{
    fn seek(
        &mut self,
        pos: std::io::SeekFrom,
    ) -> impl Future<Output = std::io::Result<u64>> + Send {
        self.0.seek(pos)
    }
}
pub struct WriteCallback<W, C>(W, C);
impl<W, C> WriteCallback<W, C> {
    pub fn new(w: W, c: C) -> Self {
        Self(w, c)
    }
}
impl<W, C> Write for WriteCallback<W, C>
where
    W: Write + Send,
    C: FnMut(u64) -> Pin<Box<dyn Future<Output = BinResult<()>> + Send>> + Send,
{
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<usize>> + Send {
        async move {
            let result = self.0.write(buf).await?;
            (self.1)(result as u64)
                .await
                .map(|_| result)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }

    fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send {
        self.0.flush()
    }
}
impl<W, C> Callback for WriteCallback<W, C>
where
    W: Write + Send,
    C: FnMut(u64) -> Pin<Box<dyn Future<Output = BinResult<()>> + Send>> + Send,
{
    fn callback(&mut self, bytes: u64) -> impl Future<Output = std::io::Result<()>> + Send {
        async move {
            (self.1)(bytes)
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }
}
impl<W, C> Seek for WriteCallback<W, C>
where
    W: Seek,
    C: FnMut(u64) -> Pin<Box<dyn Future<Output = BinResult<()>> + Send>> + Send,
{
    fn seek(
        &mut self,
        pos: std::io::SeekFrom,
    ) -> impl Future<Output = std::io::Result<u64>> + Send {
        self.0.seek(pos)
    }
}
