use crate::io::seek::Seek;
use crate::io::write::Write;
use crate::{BinResult, BinWrite, BinWriterExt, Endian};
use std::marker::PhantomData;

macro_rules! binwrite_num_impl {
    ($($type_name:ty),*$(,)?) => {
        $(
            impl BinWrite for $type_name {
                type Args<'a> = ();

                 fn write_options<'a, 'w, W>(&'a self, writer: &'w mut W, endian: Endian, _args: Self::Args<'a>) -> impl Future<Output=BinResult<()>> + Send + 'w
                    where
                        'a: 'w,
                        W: Write + Seek + Send,
                        Self: Sync + 'a
                {
                    async move {
                       writer.write_all(&match endian {
                            Endian::Big => self.to_be_bytes(),
                            Endian::Little => self.to_le_bytes(),
                        }).await.map_err(Into::into)
                    }
                }
            }
        )*
    };
}

binwrite_num_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl<T, const N: usize> BinWrite for [T; N]
where
    T: BinWrite + Sync,
    for<'a> T::Args<'a>: std::clone::Clone,
{
    type Args<'a>
        = T::Args<'a>
    where
        Self: 'a;

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
            if std::mem::size_of::<T>() == 1 && std::mem::align_of::<T>() == 1 {
                // 可能是 u8、i8 等单字节类型
                // 安全转换到 [u8]
                let slice = unsafe {
                    std::slice::from_raw_parts(self.as_ptr() as *const u8, std::mem::size_of_val(self))
                };
                writer.write_all(slice).await?;
            } else {
                for item in self {
                    T::write_options(item, writer, endian, args.clone()).await?;
                }
            }
            Ok(())
        }
    }
}
impl<T> BinWrite for [T]
where
    T: BinWrite + Sync,
    for<'a> T::Args<'a>: std::clone::Clone,
{
    type Args<'a>
        = T::Args<'a>
    where
        Self: 'a;

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
            for item in self {
                T::write_options(item, writer, endian, args.clone()).await?;
            }
            Ok(())
        }
    }
}
impl<T> BinWrite for Vec<T>
where
    T: BinWrite + Sync,
    for<'a> T::Args<'a>: std::clone::Clone,
{
    type Args<'a>
        = T::Args<'a>
    where
        Self: 'a;

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
            // if let Some(this) = <dyn Any>::downcast_ref::<Vec<u8>>(self) {
            //     writer.write_all(this).await?;
            // } else if let Some(this) = <dyn Any>::downcast_ref::<Vec<i8>>(self) {
            //     writer
            //         .write_all(bytemuck::cast_slice(this.as_slice()))
            //         .await?;
            // } else {
            for item in self {
                T::write_options(item, writer, endian, args.clone()).await?;
            }
            // }
            Ok(())
        }
    }
}
impl<T: BinWrite + Sync + ?Sized> BinWrite for &T {
    type Args<'a>
        = T::Args<'a>
    where
        Self: 'a;

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
        async move { (**self).write_options(writer, endian, args).await }
    }
}
impl<T: BinWrite + Sync + ?Sized> BinWrite for Box<T> {
    type Args<'a>
        = T::Args<'a>
    where
        Self: 'a;

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
        async move { (**self).write_options(writer, endian, args).await }
    }
    //
    //     async fn write_options<W: Write + Seek + Send>(
    //         &self,
    //         writer: &mut W,
    //         endian: Endian,
    //         args: Self::Args<'_>,
    //     ) -> BinResult<()> {
    //         if let Some(this) = <dyn Any>::downcast_ref::<Box<[u8]>>(self) {
    //             writer.write_all(this).await?;
    //         } else {
    //             (**self).write_options(writer, endian, args).await?;
    //         }
    //
    //         Ok(())
    //     }
}
impl<T: BinWrite + Sync> BinWrite for Option<T> {
    type Args<'a>
        = T::Args<'a>
    where
        Self: 'a;

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
            match self {
                Some(inner) => {
                    writer.write_type(&true, endian).await?;
                    inner.write_options(writer, endian, args).await
                }
                None => {
                    writer.write_type(&false, endian).await?;
                    Ok(())
                }
            }
        }
    }
}
impl<T> BinWrite for PhantomData<T> {
    type Args<'a>
        = ()
    where
        T: 'a;

    fn write_options<'a, 'w, W>(
        &'a self,
        _writer: &'w mut W,
        _endian: Endian,
        _args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
    {
        async move { Ok(()) }
    }
}
//
impl BinWrite for () {
    type Args<'a> = ();

    fn write_options<'a, 'w, W>(
        &'a self,
        _writer: &'w mut W,
        _endian: Endian,
        _args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
    {
        async move { Ok(()) }
    }
}
macro_rules! write_tuple_impl {
    ($type1:ident $(, $types:ident)*) => {
        #[allow(non_camel_case_types)]
        impl<$type1, $($types),*> BinWrite for ($type1, $($types,)*)
        where
            $type1: BinWrite + Sync,
            $($types: BinWrite + Sync),*
        {
            type Args<'a> = ($type1::Args<'a>, $($types::Args<'a>,)*)
            where
                $type1: 'a,
                $($types: 'a),*;

            fn write_options<'a, 'w, W>(
                &'a self,
                writer: &'w mut W,
                endian: Endian,
                ($type1, $($types,)*): Self::Args<'a>,
            ) -> impl Future<Output = BinResult<()>> + Send + 'w
            where
                'a: 'w,
                W: Write + Seek + Send,
                Self: Sync + 'a,
            {
                async move {
                    write_tuple_impl!(@write self, writer, endian, ($type1, $($types,)*); 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31);
                    Ok(())
                }
            }
        }

        write_tuple_impl!($($types),*);
    };

    () => {};

    (@write $self:ident, $writer:ident, $endian:ident, ($type1:ident, $($types:ident,)*); $idx1:tt $($indices:tt)*) => {
        $type1::write_options(&$self.$idx1, $writer, $endian, $type1).await?;
        write_tuple_impl!(@write $self, $writer, $endian, ($($types,)*); $($indices)*);
    };

    (@write $self:ident, $writer:ident, $endian:ident, (); $($indices:tt)*) => {};
}

write_tuple_impl!(
    b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19, b20, b21,
    b22, b23, b24, b25, b26, b27, b28, b29, b30, b31, b32
);
impl BinWrite for String {
    type Args<'a> = ();

    fn write_options<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
        endian: Endian,
        _args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
    {
        async move {
            let bytes = self.as_bytes();
            writer.write_type(&(bytes.len() as u64), endian).await?;
            writer.write_all(bytes).await?;
            Ok(())
        }
    }
}

impl BinWrite for bool {
    type Args<'a> = ();

    fn write_options<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
        _endian: Endian,
        _args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
    {
        async move {
            writer.write_all(&[*self as u8]).await?;
            Ok(())
        }
    }
}
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use anyhow::Result;
//     use std::io::Cursor;
//
//     #[tokio::test]
//     async fn test_write_slice() -> Result<()> {
//         let mut data = Cursor::new(Vec::new());
//         data.write_all(&[1, 2, 3]).await?;
//         assert_eq!(data.into_inner(), vec![1, 2, 3]);
//         Ok(())
//     }
// }
