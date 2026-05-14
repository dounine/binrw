use binrw::BinWriterExt;
use binrw::io::{Read, Seek};
use std::io::{Cursor, SeekFrom};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let list = b"";
    dbg!(list);
    // println!("{}",list);
    let mut data = Cursor::new(vec![1,2,3,4]);
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
