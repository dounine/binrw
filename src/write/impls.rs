use crate::io::seek::Seek;
use crate::io::write::Write;
use crate::{BinResult, BinWrite, BinWriterExt, Endian};
use std::any::Any;
use std::marker::PhantomData;

macro_rules! binwrite_num_impl {
    ($($type_name:ty),*$(,)?) => {
        $(
            impl BinWrite for $type_name {
                type Args<'a> = ();

                async fn write_options<W: Write + Seek + Send>(
                    &self,
                    writer: &mut W,
                    endian: Endian,
                    (): Self::Args<'_>,
                ) -> BinResult<()> {
                    writer.write_all(&match endian {
                        Endian::Big => self.to_be_bytes(),
                        Endian::Little => self.to_le_bytes(),
                    }).await.map_err(Into::into)
                }
            }
        )*
    };
}

binwrite_num_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl<T, const N: usize> BinWrite for [T; N]
where
    T: BinWrite + Sync + 'static,
    for<'a> T::Args<'a>: Clone,
{
    type Args<'a> = T::Args<'a>;

    async fn write_options<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        if let Some(this) = <dyn Any>::downcast_ref::<[u8; N]>(self) {
            writer.write_all(&this[..]).await?;
        } else {
            for item in self {
                T::write_options(item, writer, endian, args.clone()).await?;
            }
        }

        Ok(())
    }
}

impl<T> BinWrite for [T]
where
    T: BinWrite + Sync,
    for<'a> T::Args<'a>: Clone,
{
    type Args<'a> = T::Args<'a>;

    async fn write_options<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        for item in self {
            T::write_options(item, writer, endian, args.clone()).await?;
        }

        Ok(())
    }
}

impl<T> BinWrite for Vec<T>
where
    T: BinWrite + Sync + 'static,
    for<'a> T::Args<'a>: Clone,
{
    type Args<'a> = T::Args<'a>;

    async fn write_options<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        if let Some(this) = <dyn Any>::downcast_ref::<Vec<u8>>(self) {
            writer.write_all(this).await?;
        } else if let Some(this) = <dyn Any>::downcast_ref::<Vec<i8>>(self) {
            writer
                .write_all(bytemuck::cast_slice(this.as_slice()))
                .await?;
        } else {
            for item in self {
                T::write_options(item, writer, endian, args.clone()).await?;
            }
        }

        Ok(())
    }
}

impl<T: BinWrite + Sync + ?Sized> BinWrite for &T {
    type Args<'a> = T::Args<'a>;

    fn write_options<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<()>> + Send {
        async move { (**self).write_options(writer, endian, args).await }
    }
}

impl<T: BinWrite + Sync + ?Sized + 'static> BinWrite for Box<T> {
    type Args<'a> = T::Args<'a>;

    async fn write_options<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        if let Some(this) = <dyn Any>::downcast_ref::<Box<[u8]>>(self) {
            writer.write_all(this).await?;
        } else {
            (**self).write_options(writer, endian, args).await?;
        }

        Ok(())
    }
}
// impl BinWrite for Option<String> {
//     type Args<'a> = ();
//
//     fn write_options<W: Write + Seek + Send>(
//         &self,
//         writer: &mut W,
//         endian: Endian,
//         args: Self::Args<'_>,
//     ) -> impl std::future::Future<Output = BinResult<()>> + Send
//     where
//         Self: Sync,
//     {
//         async move {
//             if let Some(inner) = self {
//                 writer.write_type(&true, endian).await?;
//                 let data = inner.as_bytes();
//                 writer.write_type(&(data.len() as u64), endian).await?;
//                 writer.write_all(data).await?;
//             } else {
//                 writer.write_type(&false, endian).await?;
//             }
//             Ok(())
//         }
//     }
// }
impl<T: BinWrite + Sync> BinWrite for Option<T> {
    type Args<'a> = T::Args<'a>;

    async fn write_options<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
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

impl<T> BinWrite for PhantomData<T> {
    type Args<'a> = ();

    async fn write_options<W: Write + Seek + Send>(
        &self,
        _: &mut W,
        _: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<()> {
        Ok(())
    }
}

impl BinWrite for () {
    type Args<'a> = ();

    async fn write_options<W: Write + Seek + Send>(
        &self,
        _: &mut W,
        _: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<()> {
        Ok(())
    }
}

macro_rules! write_tuple_impl {
    ($type1:ident $(, $types:ident)*) => {
        #[allow(non_camel_case_types)]
        impl<Args: Clone + Send,
            $type1: for<'a> BinWrite<Args<'a> = Args> + Sync, $($types: for<'a> BinWrite<Args<'a> = Args> + Sync),*
        > BinWrite for ($type1, $($types),*) {
            type Args<'a> = Args;

            async fn write_options<W: Write + Seek + Send>(
                &self,
                writer: &mut W,
                endian: Endian,
                args: Self::Args<'_>,
            ) -> BinResult<()> {
                let ($type1, $(
                    $types
                ),*) = self;

                $type1.write_options(writer, endian, args.clone()).await?;
                $(
                    $types.write_options(writer, endian, args.clone()).await?;
                )*

                Ok(())
            }
        }

        write_tuple_impl!($($types),*);
    };

    () => {};
}

write_tuple_impl!(
    b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19, b20, b21,
    b22, b23, b24, b25, b26, b27, b28, b29, b30, b31, b32
);
impl BinWrite for String {
    type Args<'a> = ();

    fn write_options<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
    {
        async move {
            let bytes = self.as_bytes();
            writer.write_type(&(bytes.len() as u64), endian).await?;
            writer.write_all(&bytes).await?;
            Ok(())
        }
    }
}
impl BinWrite for bool {
    type Args<'a> = ();

    async fn write_options<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        _endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {
        writer.write_all(&[*self as u8]).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_write_slice() -> Result<()> {
        let mut data = Cursor::new(Vec::new());
        data.write_all(&[1, 2, 3]).await?;
        assert_eq!(data.into_inner(), vec![1, 2, 3]);
        Ok(())
    }
}
