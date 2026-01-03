use std::cmp;
use std::fs::File;

pub trait Read {
    fn read_byte(&mut self) -> impl Future<Output = std::io::Result<u8>> + Send
    where
        Self: Send,
    {
        async move {
            let mut value = [0u8; 1];
            Self::read_exact(self, &mut value).await?;
            Ok(value[0])
        }
    }
    fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = std::io::Result<usize>> + Send;
    fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send;
    fn read_exact(&mut self, buf: &mut [u8]) -> impl Future<Output = std::io::Result<()>> + Send
    where
        Self: Send,
    {
        async {
            let mut n = 0;
            while n < buf.len() {
                let count = self.read(&mut buf[n..]).await?;
                if count == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "failed to fill whole buffer",
                    ));
                }
                n += count;
            }
            Ok(())
        }
    }
    fn read_to_end(
        &mut self,
        buf: &mut Vec<u8>,
    ) -> impl Future<Output = std::io::Result<usize>> + Send
    where
        Self: Send,
    {
        async {
            let start_len = buf.len();
            let mut buffer = [0u8; 1024 * 8]; // 临时缓冲区
            loop {
                let count = self.read(&mut buffer).await?;
                if count == 0 {
                    break;
                }
                buf.extend_from_slice(&buffer[..count]);
            }
            Ok(buf.len() - start_len)
        }
    }
}

pub trait ReadExt {
    // 消耗 self 所有权的 take
    fn take(self, limit: u64) -> Take<Self>
    where
        Self: Sized,
    {
        Take { inner: self, limit }
    }

    // 不消耗 self 所有权，而是借用 self 的 take_ref (类似于 std::io::Read::by_ref().take())
    // 这里的命名为了避免冲突，使用 take_borrowed 或者类似的
    // 但更标准的做法是让 Take 支持 &mut R
}

impl<R: Read> ReadExt for R {}

pub struct Take<R> {
    inner: R,
    limit: u64,
}
// impl Read for Take<Arc<async_lock::Mutex<Cursor<Vec<u8>>>>> {
//     fn read(&mut self, buf: &mut [u8]) -> impl Future<Output=std::io::Result<usize>> + Send {
//         async move {
//             if self.limit == 0 {
//                 return Ok(0);
//             }
//             let max = std::cmp::min(buf.len() as u64, self.limit) as usize;
//             let mut inner = self.inner.lock().await;
//             let n = inner.read(&mut buf[..max]).await?;
//             self.limit -= n as u64;
//             Ok(n)
//         }
//     }
//
//     fn flush(&mut self) -> impl Future<Output=std::io::Result<()>> + Send {
//         async move{
//             Ok(())
//         }
//     }
// }
impl<R: Read + Send> Read for Take<R> {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.limit == 0 {
            return Ok(0);
        }

        let max = std::cmp::min(buf.len() as u64, self.limit) as usize;
        let n = self.inner.read(&mut buf[..max]).await?;
        self.limit -= n as u64;
        Ok(n)
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush().await
    }
}
#[cfg(test)]
mod take_tests {
    use crate::io::read::ReadExt;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_take() {
        let mut data: Cursor<Vec<u8>> = Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        data.set_position(3);
        let mut data = data.take(3);
        let mut new_data = Cursor::new(Vec::new());
        crate::io::copy(&mut data, &mut new_data).await.unwrap();
        assert_eq!(new_data.into_inner(), vec![4, 5, 6]);
    }
}
// 这里的关键是为 &mut R 实现 Read，这样 R 就可以被借用了
impl<R: Read + ?Sized + Send> Read for &mut R {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        (**self).read(buf).await
    }
    async fn flush(&mut self) -> std::io::Result<()> {
        (**self).flush().await
    }
}

impl<T> Read for std::io::Cursor<T>
where
    std::io::Cursor<T>: std::io::Read + Unpin,
    T: Send,
{
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        std::io::Read::read(self, buf)
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        std::io::Read::read_exact(self, buf)
    }
}
impl Read for File {
    fn read(
        &mut self,
        buf: &mut [u8],
    ) -> impl std::future::Future<Output = std::io::Result<usize>> + Send {
        async { std::io::Read::read(self, buf) }
    }

    fn flush(&mut self) -> impl std::future::Future<Output = std::io::Result<()>> + Send {
        async { std::io::Write::flush(self) }
    }
}
impl Read for &[u8] {
    fn read(
        &mut self,
        buf: &mut [u8],
    ) -> impl std::future::Future<Output = std::io::Result<usize>> + Send {
        async {
            let amt = cmp::min(buf.len(), self.len());
            let (a, b) = self.split_at(amt);
            if amt == 1 {
                buf[0] = a[0];
            } else {
                buf[..amt].copy_from_slice(a);
            }

            *self = b;
            Ok(amt)
        }
    }

    fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send {
        async { Ok(()) }
    }
}
pub trait ReadAt: Send + Sync {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;
    fn size(&self) -> u64;
}

impl ReadAt for File {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            FileExt::read_at(self, buf, offset)
        }
        #[cfg(target_os = "wasi")]
        {
            use std::os::wasi::fs::FileExt;
            FileExt::read_at(self, buf, offset)
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::FileExt;
            FileExt::seek_read(self, buf, offset)
        }
        #[cfg(not(any(unix, target_os = "wasi", windows)))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "read_at is not supported on this platform",
            ))
        }
    }
    fn size(&self) -> u64 {
        self.metadata().map(|m| m.len()).unwrap_or(0)
    }
}

impl ReadAt for Vec<u8> {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        let offset = offset as usize;
        if offset >= self.len() {
            return Ok(0);
        }
        let end = std::cmp::min(offset + buf.len(), self.len());
        let slice = &self[offset..end];
        buf[..slice.len()].copy_from_slice(slice);
        Ok(slice.len())
    }
    fn size(&self) -> u64 {
        self.len() as u64
    }
}
