use std::cmp;
use std::fs::File;

pub trait Read {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    async fn flush(&mut self) -> std::io::Result<()>;
    async fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
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
    async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
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

pub trait ReadExt: Read {
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

impl<R: Read> Read for Take<R> {
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

// 这里的关键是为 &mut R 实现 Read，这样 R 就可以被借用了
impl<R: Read + ?Sized> Read for &mut R {
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
{
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        std::io::Read::read(self, buf)
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        // Cursor doesn't really flush, but we satisfy the trait
        Ok(())
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        std::io::Read::read_exact(self, buf)
    }
}
impl Read for File {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        std::io::Read::read(self, buf)
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        std::io::Write::flush(self)
    }
}
impl Read for &[u8] {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
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

    async fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
