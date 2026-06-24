use std::io::SeekFrom;

use crate::io::{Read, Seek, Write};

/// 带内部缓冲的读取器，减少对底层读取器的系统调用次数
pub struct BufReader<R> {
    inner: R,
    buf: Vec<u8>,
    pos: usize,
    cap: usize,
}

impl<R> BufReader<R> {
    /// 创建一个默认大小（8KB）的BufReader
    pub fn new(inner: R) -> Self {
        Self::with_capacity(8 * 1024, inner)
    }

    /// 创建一个指定容量的BufReader
    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        BufReader {
            inner,
            buf: vec![0; capacity],
            pos: 0,
            cap: 0,
        }
    }

    /// 获取底层读取器的引用
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// 获取底层读取器的可变引用
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// 消耗自身，返回底层读取器
    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Seek + Send> BufReader<R> {
    /// 将底层读取器回退到正确的位置（即 BufReader 实际消费数据后的位置）
    pub async fn rewind_position(&mut self) -> std::io::Result<()> {
        if self.cap > 0 {
            let buf_remaining = (self.cap - self.pos) as i64;
            if buf_remaining > 0 {
                self.inner.seek(SeekFrom::Current(-buf_remaining)).await?;
            }
            // 重置缓冲区
            self.pos = 0;
            self.cap = 0;
        }
        Ok(())
    }
}

impl<R: Read + Send> Read for BufReader<R> {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // 如果内部缓冲区为空，先填充缓冲区
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf).await?;
            self.pos = 0;
            if self.cap == 0 {
                return Ok(0);
            }
        }

        // 从内部缓冲区复制数据到用户提供的缓冲区
        let available = self.cap - self.pos;
        let to_copy = std::cmp::min(available, buf.len());
        buf[..to_copy].copy_from_slice(&self.buf[self.pos..self.pos + to_copy]);
        self.pos += to_copy;

        Ok(to_copy)
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush().await
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        let mut remaining = buf;
        while !remaining.is_empty() {
            let n = self.read(remaining).await?;
            if n == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "failed to fill whole buffer",
                ));
            }
            remaining = &mut remaining[n..];
        }
        Ok(())
    }
}

impl<R: Seek + Read + Send> crate::io::seek::Seek for BufReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> impl Future<Output = std::io::Result<u64>> + Send {
        async move {
            // 先获取当前底层位置
            let current_inner_pos = self.inner.stream_position().await?;
            let buf_remaining = (self.cap - self.pos) as i64;
            let current_pos = current_inner_pos - buf_remaining as u64;

            // 计算最终要 seek 到的绝对位置
            let target_pos = match pos {
                SeekFrom::Current(0) => {
                    return Ok(current_pos);
                }
                SeekFrom::Current(delta) => (current_pos as i64 + delta) as u64,
                SeekFrom::Start(new_pos) => new_pos,
                SeekFrom::End(delta) => {
                    let end_pos = self.inner.seek(SeekFrom::End(0)).await?;
                    (end_pos as i64 + delta) as u64
                }
            };

            // 计算目标位置在缓冲区范围内的情况
            let buf_start = current_inner_pos - self.cap as u64;
            let buf_end = current_inner_pos;

            if target_pos >= buf_start && target_pos < buf_end {
                // 目标位置在缓冲区内
                let offset_in_buf = (target_pos - buf_start) as usize;
                self.pos = offset_in_buf;
                Ok(target_pos)
            } else {
                // 目标位置不在缓冲区，清空缓冲区并直接 seek
                self.pos = 0;
                self.cap = 0;
                self.inner.seek(SeekFrom::Start(target_pos)).await
            }
        }
    }
}

/// 带内部缓冲的写入器，减少对底层写入器的系统调用次数
pub struct BufWriter<W> {
    inner: W,
    buf: Vec<u8>,
    cap: usize,
}

impl<W> BufWriter<W> {
    /// 创建一个默认大小（8KB）的BufWriter
    pub fn new(inner: W) -> Self {
        Self::with_capacity(8 * 1024, inner)
    }

    /// 创建一个指定容量的BufWriter
    pub fn with_capacity(capacity: usize, inner: W) -> Self {
        BufWriter {
            inner,
            buf: Vec::with_capacity(capacity),
            cap: capacity,
        }
    }

    /// 获取底层写入器的引用
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// 获取底层写入器的可变引用
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// 消耗自身，返回底层写入器。注意：缓冲区的数据会被丢弃！
    /// 使用 into_inner_flush() 来保留缓冲区数据
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write + Send> BufWriter<W> {
    /// 消耗自身，先刷新缓冲区再返回底层写入器
    pub async fn into_inner_flush(mut self) -> std::io::Result<W> {
        self.flush().await?;
        Ok(self.inner)
    }

    /// 将缓冲区的内容写入到底层写入器
    async fn flush_buf(&mut self) -> std::io::Result<()> {
        if !self.buf.is_empty() {
            self.inner.write_all(&self.buf).await?;
            self.buf.clear();
        }
        Ok(())
    }
}

impl<W: Write + Send> Write for BufWriter<W> {
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // 如果缓冲区为空且写入的数据大于等于缓冲区容量，直接写入底层写入器
        if self.buf.is_empty() && buf.len() >= self.cap {
            return self.inner.write(buf).await;
        }

        // 如果当前缓冲区剩余空间足够，直接写入缓冲区
        if self.buf.len() + buf.len() <= self.cap {
            self.buf.extend_from_slice(buf);
            return Ok(buf.len());
        }

        // 否则，先刷新缓冲区，再处理写入
        self.flush_buf().await?;

        // 如果写入的数据仍然大于等于缓冲区容量，直接写入底层写入器
        if buf.len() >= self.cap {
            self.inner.write(buf).await
        } else {
            // 否则，写入缓冲区
            self.buf.extend_from_slice(buf);
            Ok(buf.len())
        }
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        self.flush_buf().await?;
        self.inner.flush().await
    }

    async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        let mut remaining = buf;
        while !remaining.is_empty() {
            let n = self.write(remaining).await?;
            if n == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "failed to write whole buffer",
                ));
            }
            remaining = &remaining[n..];
        }
        Ok(())
    }
}

impl<W: Seek + Write + Send> crate::io::seek::Seek for BufWriter<W> {
    fn seek(&mut self, pos: SeekFrom) -> impl Future<Output = std::io::Result<u64>> + Send {
        async move {
            // 先刷新缓冲区
            self.flush_buf().await?;
            // 然后直接 seek 到底层写入器
            self.inner.seek(pos).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BinWriterExt, io::seek::Seek};
    use std::io::{Cursor, SeekFrom};

    #[tokio::test]
    async fn test_buf_writer_basic() {
        let data = Cursor::new(Vec::new());
        let mut writer = BufWriter::with_capacity(4, data);

        writer.write_all(&[1, 2, 3]).await.unwrap();
        // 数据还在缓冲区，没有写入到下层
        assert_eq!(writer.get_ref().get_ref(), &[]);

        writer.flush().await.unwrap();
        // 刷新后数据会写入
        assert_eq!(writer.get_ref().get_ref(), &[1, 2, 3]);

        // 写入大于缓冲区的数据
        writer.write_all(&[4, 5, 6, 7, 8]).await.unwrap();
        // 大的数据会直接写入，不经过缓冲区
        assert_eq!(writer.get_ref().get_ref(), &[1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[tokio::test]
    async fn test_buf_writer_u32() {
        let mut data = Cursor::new(Vec::new());
        let mut writer = BufWriter::with_capacity(4, &mut data);
        writer.write_le(&8_u32).await.unwrap();
    }

    #[tokio::test]
    async fn test_buf_writer_flush_on_drop() {
        let data = Cursor::new(Vec::new());
        let mut writer = BufWriter::with_capacity(4, data);
        writer.write_all(&[1, 2, 3]).await.unwrap();

        // 使用 into_inner_flush 来刷新并获取底层写入器
        let mut inner = writer.into_inner_flush().await.unwrap();
        assert_eq!(inner.get_ref(), &[1, 2, 3]);
    }

    #[tokio::test]
    async fn test_buf_writer_seek() {
        let data = Cursor::new(Vec::new());
        let mut writer = BufWriter::with_capacity(4, data);

        writer.write_all(&[1, 2, 3, 4, 5]).await.unwrap();

        // Seek 会先刷新缓冲区
        let pos = writer.seek(SeekFrom::Start(2)).await.unwrap();
        assert_eq!(pos, 2);

        writer.write_all(&[6, 7]).await.unwrap();
        writer.flush().await.unwrap();

        assert_eq!(writer.get_ref().get_ref(), &[1, 2, 6, 7, 5]);
    }
}
