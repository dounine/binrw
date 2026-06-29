
use crate::{
    io::{Read, Seek, Write, bytes::BytesCallback},
};
pub trait Callback {
    fn callback(&mut self, bytes: u64) -> impl Future<Output = std::io::Result<()>> + Send;
}
pub struct ReadCallback<R, C>(R, C);
impl<R, C> ReadCallback<R, C> {
    pub fn new(r: R, c: C) -> Self {
        Self(r, c)
    }
    pub fn into_inner(self) -> R {
        self.0
    }
    pub fn into_parts(self) -> (R, C) {
        (self.0, self.1)
    }
}

impl<R, C> Read for ReadCallback<R, C>
where
    R: Read + Send,
    C: BytesCallback + Send,
{
    fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = std::io::Result<usize>> + Send {
        async move {
            let result = self.0.read(buf).await?;
            self.1.call(result as u64)
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
    C: BytesCallback + Send,
{
    fn callback(&mut self, bytes: u64) -> impl Future<Output = std::io::Result<()>> + Send {
        async move {
            self.1.call(bytes)
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }
}
impl<R, C> Seek for ReadCallback<R, C>
where
    R: Seek,
    C: BytesCallback + Send,
{
    fn seek(
        &mut self,
        pos: std::io::SeekFrom,
    ) -> impl Future<Output = std::io::Result<u64>> + Send {
        self.0.seek(pos)
    }
}
pub struct WriteCallback<W, C>(pub W, pub C);
impl<W, C> WriteCallback<W, C> {
    pub fn new(w: W, c: C) -> Self {
        Self(w, c)
    }
    pub fn into_inner(self) -> W {
        self.0
    }
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.0
    }
    pub fn into_parts(self) -> (W, C) {
        (self.0, self.1)
    }
}
impl<W, C> Write for WriteCallback<W, C>
where
    W: Write + Send,
    C: BytesCallback + Send,
{
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<usize>> + Send {
        async move {
            let result = self.0.write(buf).await?;
            self.1.call(result as u64)
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
    C: BytesCallback + Send,
{
    fn callback(&mut self, bytes: u64) -> impl Future<Output = std::io::Result<()>> + Send {
        async move {
            self.1.call(bytes)
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }
}
impl<W, C> Seek for WriteCallback<W, C>
where
    W: Seek,
    C: BytesCallback + Send,
{
    fn seek(
        &mut self,
        pos: std::io::SeekFrom,
    ) -> impl Future<Output = std::io::Result<u64>> + Send {
        self.0.seek(pos)
    }
}
