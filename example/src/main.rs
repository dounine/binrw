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
        async move { unimplemented!() }
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
    let list = b"";
    dbg!(list);
    let mut data = Cursor::new(vec![1, 2, 3, 4]);
    let mut output = Cursor::new(vec![]);
    binrw::io::copy(&mut data, &mut output).await?;
    // let age: (u32, u32) = data.read_type(Endian::Little).await?;
    // let value: u32 = 18;
    // let list:Vec<User2> = data.read_type_args(Endian::Little,(1,1)).await?;
    // let users: (User1, User2) = data
    //     .read_type_args(Endian::Little, (&value, value))
    //     .await?;
    // data.write_type(&users,Endian::Little).await?;
    // data.write_le(&1_u32).await?;
    // data.seek(SeekFrom::Start(0)).await?;
    // let mut bytes = vec![0u8; 4];
    // data.read_exact(bytes.as_mut_slice()).await?;
    // dbg!(bytes);
    // let data = [1, 2, 3, 4, 5, 6, 7];

    // // 1.94.0 稳定: 遍历长度为 3 的固定窗口
    // // 编译器自动推断 N=3，类型为 &[i32; 3]
    // for &[a, b, ..] in data.array_windows::<3>() {
    //     println!("窗口: ({}, {})", a, b);
    // }
    Ok(())
}
