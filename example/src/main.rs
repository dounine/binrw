use binrw::io::{BufReader, BufWriter};
use binrw::io::{Read, Seek, Write};
use binrw::{BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt, Endian};
use std::io::Cursor;

pub struct Data {
    age: u32,
}

// impl BinRead for Data {
//     type Args<'a>
//     where
//         Self: 'a,
//     = (&'a [u8], u32) where Self: 'a;
//
//     fn read_options<'a, 'r, R: Read + Seek + Send>(
//         reader: &'r mut R,
//         endian: Endian,
//         args: Self::Args<'a>,
//     ) -> impl Future<Output = BinResult<Self>> + Send + 'r
//     where
//         'a: 'r,
//         Self: Send + 'a,
//     {
//         async move {
//             Ok(Self{
//                 age: 0,
//             })
//         }
//     }
// }
pub struct User1 {
    age: u32,
    value: u32,
}
impl BinWrite for User1 {
    type Args<'a> = ();

    fn write_options<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
        endian: Endian,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
    {
        async move { unimplemented!() }
    }
}
impl BinRead for User1 {
    type Args<'a> = &'a u32;

    fn read_options<'a, 'r, R: Read + Seek + Send>(
        reader: &'r mut R,
        endian: Endian,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<Self>> + Send + 'r
    where
        'a: 'r,
        Self: Send + 'a,
    {
        async move {
            let age = args;
            Ok(Self {
                age: reader.read_type(endian).await?,
                value: *age,
            })
        }
    }
}
pub struct User2 {
    age: u32,
    value: u32,
}
impl BinWrite for User2 {
    type Args<'a> = ();

    fn write_options<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
        endian: Endian,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
    {
        async move {
            let mut writer = BufWriter::new(writer);
            writer.write_type(&self.age, endian).await?;
            writer.write_type(&self.value, endian).await?;
            writer.flush().await?;
            Ok(())
        }
    }
}
impl BinRead for User2 {
    type Args<'a> = u32;

    fn read_options<'a, 'r, R: Read + Seek + Send>(
        reader: &'r mut R,
        endian: Endian,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<Self>> + Send + 'r
    where
        'a: 'r,
        Self: Send + 'a,
    {
        async move {
            let age = args;
            Ok(Self {
                age: reader.read_type(endian).await?,
                value: age,
            })
        }
    }
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 演示 BufReader 的使用
    let list = b"";
    dbg!(list);
    // let mut data = Cursor::new(vec![1, 2, 3, 4]);
    // let mut reader = BufReader::new(data);
    // let age: u32 = reader.read_type(Endian::Little).await?;
    let mut data = Cursor::new(vec![1, 2, 3, 4, 5]);
    {
        let mut reader = BufReader::new(&mut data);
        let mut reader = BufReader::new(&mut reader);
        reader.read_exact(&mut [0_u8; 1]).await?;
        // 从最内层到最外层依次回退 position
        reader.rewind_position().await?;
        reader.get_mut().rewind_position().await?;
    }
    let pos = data.position();
    assert!(pos == 1);
    // dbg!(pos);

    // // 演示 BufWriter 的使用
    // let output_data = Cursor::new(Vec::new());
    // let mut writer = BufWriter::with_capacity(4, output_data);

    // // 写入一些数据
    // writer.write_all(&[5, 6, 7]).await?;
    // writer.write_all(&[8, 9, 10, 11, 12]).await?; // 大于缓冲区容量，直接写入

    // // 刷新并获取写入后的数据
    // let output = writer.into_inner_flush().await?;
    // dbg!(output.into_inner());

    Ok(())
}
