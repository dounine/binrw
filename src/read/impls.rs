use crate::io::read::Read;
use crate::io::seek::Seek;
use crate::{BinRead, BinReaderExt};
use crate::{BinResult, Endian};
use async_trait::async_trait;
use std::io::SeekFrom;

macro_rules! read_impl {
    ($($type_name:ty),*$(,)?) => {
        $(
            impl crate::BinRead for $type_name {
                type Args<'a> = ();

                async fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, (): Self::Args<'_>) -> BinResult<Self> {
                    let mut val = [0; core::mem::size_of::<$type_name>()];
                    let pos = reader.stream_position().await?;

                    let result = reader.read_exact(&mut val).await;
                    if let Err(e) = result {
                       return Err(crate::private::restore_position(reader, pos).await(e));
                    }
                    Ok(match endian {
                        Endian::Big => {
                            <$type_name>::from_be_bytes(val)
                        }
                        Endian::Little => {
                            <$type_name>::from_le_bytes(val)
                        }
                    })
                }
            }
        )*
    }
}
// impl BinRead for u8 {
//     type Args<'a> = ();
//
//     async fn read_options<R: Read + Seek>(
//         reader: &mut R,
//         endian: Endian,
//         args: Self::Args<'_>,
//     ) -> BinResult<Self> {
//         let mut val = [0; core::mem::size_of::<u8>()];
//         let pos = reader.stream_position().await?;
//
//         let result = reader.read_exact(&mut val).await;
//         if let Err(e) = result {
//             reader.seek(SeekFrom::Start(pos)).await?;
//             return Err(crate::private::restore_position(reader, pos).await(e));
//         }
//         Ok(u8::from_be_bytes(val))
//     }
//
//     // async fn read_options<R: Read + Seek>(
//     //     reader: &mut R,
//     //     endian: Endian,
//     //     args: Self::Args<'_>,
//     // ) -> BinResult<Self> {
//     //     let mut val = [0; core::mem::size_of::<u8>()];
//     //     let pos = reader.stream_position().await?;
//     //
//     //     let result = reader.read_exact(&mut val).await;
//     //     if let Err(e) = result {
//     //         reader.seek(SeekFrom::Start(pos)).await?;
//     //         return Err(crate::private::restore_position(reader, pos).await(e));
//     //     }
//     //     Ok(u8::from_be_bytes(val))
//     // }
// }
read_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
impl<B> BinRead for Vec<B>
where
    B: BinRead + 'static,
    for<'a> B::Args<'a>: Clone + Default,
{
    type Args<'a> = usize;

    async fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let count = args;
        if core::any::TypeId::of::<B>() == core::any::TypeId::of::<u8>() {
            let mut list = vec![0u8; count];
            reader
                .read_exact(&mut list)
                .await
                .map_err(|e| crate::error::Error::Io(e))?;
            return Ok(unsafe { core::mem::transmute(list) });
        }
        let mut list = Vec::with_capacity(count);
        let b_args = B::Args::default();
        for _ in 0..count {
            list.push(B::read_options(reader, endian, b_args.clone()).await?);
        }
        Ok(list)
    }
}

impl<B, const N: usize> BinRead for [B; N]
where
    B: BinRead,
    for<'a> B::Args<'a>: Clone,
{
    type Args<'a> = B::Args<'a>;

    async fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        if !core::mem::needs_drop::<B>() {
            let mut array: core::mem::MaybeUninit<[B; N]> = core::mem::MaybeUninit::uninit();
            let mut ptr_i = array.as_mut_ptr() as *mut B;

            unsafe {
                for _ in 0..N {
                    let value_i = B::read_options(reader, endian, args.clone()).await?;
                    ptr_i.write(value_i);
                    ptr_i = ptr_i.add(1);
                }
                Ok(array.assume_init())
            }
        } else {
            struct UnsafeDropSliceGuard<Item> {
                base_ptr: *mut Item,
                initialized_count: usize,
            }

            impl<Item> Drop for UnsafeDropSliceGuard<Item> {
                fn drop(self: &'_ mut Self) {
                    unsafe {
                        core::ptr::drop_in_place(core::slice::from_raw_parts_mut(
                            self.base_ptr,
                            self.initialized_count,
                        ));
                    }
                }
            }
            unsafe {
                let mut array: core::mem::MaybeUninit<[B; N]> = core::mem::MaybeUninit::uninit();
                let mut ptr_i = array.as_mut_ptr() as *mut B;
                let mut panic_guard = UnsafeDropSliceGuard {
                    base_ptr: ptr_i,
                    initialized_count: 0,
                };

                for i in 0..N {
                    panic_guard.initialized_count = i;
                    let value_i = B::read_options(reader, endian, args.clone()).await?;
                    ptr_i.write(value_i);
                    ptr_i = ptr_i.add(1);
                }
                core::mem::forget(panic_guard);
                Ok(array.assume_init())
            }
        }
    }
}

macro_rules! binread_tuple_impl {
    ($type1:ident $(, $types:ident)*) => {
        #[allow(non_camel_case_types)]
        impl<Args: Clone, $type1: for<'a> BinRead<Args<'a> = Args>, $($types: for<'a> BinRead<Args<'a> = Args>),*> BinRead for ($type1, $($types),*) {
            type Args<'a> = Args;

            async fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, args: Self::Args<'_>) -> BinResult<Self> {
                Ok((
                    $type1::read_options(reader, endian, args.clone()).await?,
                    $(
                        <$types>::read_options(reader, endian, args.clone()).await?
                    ),*
                ))
            }
        }

        binread_tuple_impl!($($types),*);
    };

    () => {};
}

binread_tuple_impl!(
    b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19, b20, b21,
    b22, b23, b24, b25, b26, b27, b28, b29, b30, b31, b32
);

impl BinRead for () {
    type Args<'a> = ();

    async fn read_options<R: Read + Seek>(
        _: &mut R,
        _: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        Ok(())
    }
}

impl<T: BinRead> BinRead for Box<T> {
    type Args<'a> = T::Args<'a>;

    async fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        Ok(Box::new(T::read_options(reader, endian, args).await?))
    }
}

impl<T: BinRead> BinRead for Option<T> {
    type Args<'a> = T::Args<'a>;

    async fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        Ok(Some(T::read_options(reader, endian, args).await?))
    }
}

impl<T> BinRead for core::marker::PhantomData<T> {
    type Args<'a> = ();

    async fn read_options<R: Read + Seek>(
        _: &mut R,
        _: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        Ok(core::marker::PhantomData)
    }
}
impl BinRead for bool {
    type Args<'a> = ();
    async fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let value: u8 = reader.read_type_args(endian, ()).await?;
        Ok(value != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_read_slice() -> Result<()> {
        Ok(())
    }
}
