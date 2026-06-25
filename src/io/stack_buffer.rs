use std::io::SeekFrom;

use crate::io::{Read, Seek, Write};

/// 默认栈缓冲区大小 - 8KB（适合大多数场景）
pub const DEFAULT_STACK_BUF_SIZE: usize = 8 * 1024;

/// 带内部栈缓冲区的读取器，性能更高
/// 使用固定大小的栈数组，避免堆分配
pub struct StackBufReader<R, const N: usize> {
    inner: R,
    buf: [u8; N],
    pos: usize,
    cap: usize,
    inner_pos: Option<u64>,
}


impl<R, const N: usize> StackBufReader<R, N> {
    /// 创建一个默认大小（8KB）的BufReader
    pub fn new(inner: R) -> Self {
        StackBufReader {
            inner,
            buf: [0; N],
            pos: 0,
            cap: 0,
            inner_pos: None,
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

impl<R: Seek + Send, const N: usize> StackBufReader<R, N> {
    /// 将底层读取器回退到正确的位置（即 BufReader 实际消费数据后的位置）
    pub async fn rewind_position(&mut self) -> std::io::Result<()> {
        if self.cap > 0 {
            let buf_remaining = (self.cap - self.pos) as i64;
            if buf_remaining > 0 {
                self.inner.seek(SeekFrom::Current(-buf_remaining)).await?;
                if let Some(pos) = &mut self.inner_pos {
                    *pos -= buf_remaining as u64;
                }
            }
            // 重置缓冲区
            self.pos = 0;
            self.cap = 0;
        }
        Ok(())
    }

    /// 初始化或获取缓存的底层读取器位置
    async fn ensure_inner_pos(&mut self) -> std::io::Result<u64> {
        if let Some(pos) = self.inner_pos {
            Ok(pos)
        } else {
            let pos = self.inner.stream_position().await?;
            self.inner_pos = Some(pos);
            Ok(pos)
        }
    }
}

impl<R: Read + Send, const N: usize> Read for StackBufReader<R, N> {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // 如果内部缓冲区为空，先填充缓冲区
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf).await?;
            self.pos = 0;
            if self.cap == 0 {
                return Ok(0);
            }
            if let Some(pos) = &mut self.inner_pos {
                *pos += self.cap as u64;
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
}

impl<R: Seek + Read + Send, const N: usize> crate::io::seek::Seek for StackBufReader<R, N> {
    fn seek(&mut self, pos: SeekFrom) -> impl Future<Output = std::io::Result<u64>> + Send {
        async move {
            // 使用缓存的 inner_pos
            let current_inner_pos = self.ensure_inner_pos().await?;
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
                    self.inner_pos = Some(end_pos);
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
                let new_pos = self.inner.seek(SeekFrom::Start(target_pos)).await?;
                self.inner_pos = Some(new_pos);
                Ok(new_pos)
            }
        }
    }
}
/// 使用固定大小的栈数组，避免堆分配
pub struct StackBufWriter<W, const N: usize = DEFAULT_STACK_BUF_SIZE> {
    inner: W,
    buf: [u8; N],
    pos: usize,
}

impl<W, const N: usize> StackBufWriter<W, N> {
    /// 创建一个栈缓冲写入器
    pub fn new(inner: W) -> Self {
        StackBufWriter {
            inner,
            buf: [0u8; N],
            pos: 0,
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

impl<W: Write + Send, const N: usize> StackBufWriter<W, N> {
    /// 消耗自身，先刷新缓冲区再返回底层写入器
    pub async fn into_inner_flush(mut self) -> std::io::Result<W> {
        self.flush().await?;
        Ok(self.inner)
    }

    /// 将缓冲区的内容写入到底层写入器
    async fn flush_buf(&mut self) -> std::io::Result<()> {
        if self.pos > 0 {
            self.inner.write_all(&self.buf[..self.pos]).await?;
            self.pos = 0;
        }
        Ok(())
    }
}

impl<W: Write + Send, const N: usize> Write for StackBufWriter<W, N> {
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // 如果写入的数据大于等于缓冲区容量，先刷新缓冲区再直接写入底层写入器
        if buf.len() >= N {
            self.flush_buf().await?;
            return self.inner.write(buf).await;
        }

        // 如果当前缓冲区剩余空间足够，直接写入缓冲区
        if self.pos + buf.len() <= N {
            self.buf[self.pos..self.pos + buf.len()].copy_from_slice(buf);
            self.pos += buf.len();
            return Ok(buf.len());
        }

        // 否则，先刷新缓冲区，再写入当前数据
        self.flush_buf().await?;
        self.buf[..buf.len()].copy_from_slice(buf);
        self.pos = buf.len();
        Ok(buf.len())
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        self.flush_buf().await?;
        self.inner.flush().await
    }
}

impl<W: Seek + Write + Send, const N: usize> crate::io::seek::Seek for StackBufWriter<W, N> {
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
    use crate::io::seek::Seek;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_stack_buf_reader_basic() {
        let data = Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut reader = StackBufReader::<_, 4>::new(data);

        let mut buf = [0u8; 3];
        reader.read_exact(&mut buf).await.unwrap();
        assert_eq!(buf, [1, 2, 3]);

        reader.read_exact(&mut buf).await.unwrap();
        assert_eq!(buf, [4, 5, 6]);
    }

    #[tokio::test]
    async fn test_stack_buf_writer_basic() {
        let data = Cursor::new(Vec::new());
        let mut writer = StackBufWriter::<_, 4>::new(data);

        writer.write_all(&[1, 2, 3]).await.unwrap();
        writer.flush().await.unwrap();

        assert_eq!(writer.into_inner().into_inner(), vec![1, 2, 3]);
    }
}
