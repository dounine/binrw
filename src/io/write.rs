use std::fs::File;
use std::io::Cursor;

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<usize>> + Send;
    fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send;
    fn write_all(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<()>> + Send
    where
        Self: Send,
    {
        Box::pin(async move {
            let mut n = 0;
            while n < buf.len() {
                let count = self.write(&buf[n..]).await?;
                if count == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::WriteZero,
                        "failed to write whole buffer",
                    ));
                }
                n += count;
            }
            Ok(())
        })
    }
}
impl<T: AsMut<[u8]> + AsRef<[u8]> + Send> Write for Cursor<T>
where
    Cursor<T>: std::io::Write,
{
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<usize>> + Send {
        async { std::io::Write::write(self, buf) }
    }

    fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send {
        async { std::io::Write::flush(self) }
    }

    fn write_all(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<()>> + Send {
        async { std::io::Write::write_all(self, buf) }
    }
}
impl Write for File {
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<usize>> + Send {
        async { std::io::Write::write(self, buf) }
    }

    fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send {
        async { std::io::Write::flush(self) }
    }
}
impl Write for [u8] {
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<usize>> + Send {
        async move {
            self.copy_from_slice(buf);
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send {
        async { Ok(()) }
    }
}
// impl Write for Vec<u8> {
//     fn write(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<usize>> + Send {
//         async move {
//             self.copy_from_slice(buf);
//             Ok(buf.len())
//         }
//     }

//     fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send {
//         async move { Ok(()) }
//     }
// }

// 为 &mut W 实现 Write trait，这样任何实现了 Write 的类型都可以通过引用使用
impl<W: Write + ?Sized + Send> Write for &mut W {
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<usize>> + Send {
        (**self).write(buf)
    }

    fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send {
        (**self).flush()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::io::{Cursor, SeekFrom};
    use tokio::io::AsyncSeekExt;

    #[tokio::test]
    async fn test_write_cursor() -> Result<()> {
        let mut data = Cursor::new(vec![4, 5, 6]);
        let mut buffer = vec![1, 2, 3];
        crate::io::copy(&mut data, &mut buffer).await?;
        assert_eq!(buffer, vec![4, 5, 6]);
        data.seek(SeekFrom::Start(3)).await?;
        crate::io::copy(&mut data, &mut buffer).await?;
        assert_eq!(buffer, vec![4, 5, 6]);
        Ok(())
    }
}
