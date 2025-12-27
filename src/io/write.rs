use std::fs::File;
use std::io::Cursor;

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<usize>> + Send;
    fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send;
    fn write_all(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<()>> + Send
    where
        Self: Send,
    {
        async {
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
        }
    }
}
impl Write for &mut Cursor<Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<usize>> + Send {
        async move { std::io::Write::write(self, buf) }
    }

    fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send {
        async { std::io::Write::flush(self) }
    }
}
impl Write for Cursor<Vec<u8>> {
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
// impl Write for Vec<u8> {
//     fn write(&mut self, buf: &[u8]) -> impl Future<Output=std::io::Result<usize>> + Send {
//         todo!()
//     }
//
//     fn flush(&mut self) -> impl Future<Output=std::io::Result<()>> + Send {
//         todo!()
//     }
//     // async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//     //     std::io::Write::write(self, buf)
//     // }
//     //
//     // async fn flush(&mut self) -> std::io::Result<()> {
//     //     Ok(())
//     // }
// }
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_write_cursor() -> Result<()> {
        let mut data = Cursor::new(Vec::new());
        data.write_all(&[1, 2, 3]).await?;
        assert_eq!(data.into_inner(), vec![1, 2, 3]);
        Ok(())
    }
}
