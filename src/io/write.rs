use std::fs::File;
use std::io::Cursor;

pub trait Write {
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    async fn flush(&mut self) -> std::io::Result<()>;
    async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
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

impl Write for Cursor<Vec<u8>> {
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::Write::write(self, buf)
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        std::io::Write::flush(self)
    }

    async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        std::io::Write::write_all(self, buf)
    }
}
impl Write for File {
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::Write::write(self, buf)
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        std::io::Write::flush(self)
    }
}
impl Write for Vec<u8> {
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::Write::write(self, buf)
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
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
    #[tokio::test]
    async fn test_write_mut_vec() -> Result<()> {
        let mut data = Vec::new();
        data.write_all(&[1, 2, 3]).await?;
        assert_eq!(data, vec![1, 2, 3]);
        Ok(())
    }
}
