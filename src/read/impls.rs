use crate::io::read::Read;
use crate::io::seek::Seek;
use crate::{BinRead, BinReaderExt};
use crate::{BinResult, Endian};

macro_rules! read_impl {
    ($($type_name:ty),*$(,)?) => {
        $(
            impl crate::BinRead for $type_name {
                type Args<'a> = ();

                fn read_options<R: Read + Seek + Send>(reader: &mut R, endian: Endian, (): Self::Args<'_>) -> impl Future<Output = BinResult<Self>> + Send {
                    async move {
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
            }
        )*
    }
}
read_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

macro_rules! read_tuple_impl {
    ($type1:ident $(, $types:ident)*) => {
        #[allow(non_camel_case_types)]
        impl<Args: Clone + Send, $type1: for<'a> BinRead<Args<'a> = Args> + Send, $($types: for<'a> BinRead<Args<'a> = Args> + Send),*> BinRead for ($type1, $($types),*) {
            type Args<'a> = Args;

            fn read_options<R: Read + Seek + Send>(reader: &mut R, endian: Endian, args: Self::Args<'_>) -> impl Future<Output = BinResult<Self>> + Send {
                async move {
                    Ok((
                        $type1::read_options(reader, endian, args.clone()).await?,
                        $(
                            <$types>::read_options(reader, endian, args.clone()).await?
                        ),*
                    ))
                }
            }
        }

        read_tuple_impl!($($types),*);
    };

    () => {};
}

read_tuple_impl!(
    b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19, b20, b21,
    b22, b23, b24, b25, b26, b27, b28, b29, b30, b31, b32
);
impl<B> BinRead for Vec<B>
where
    B: BinRead + Send + 'static,
    for<'a> B::Args<'a>: Clone + Default,
{
    type Args<'a> = usize;

    fn read_options<R: Read + Seek + Send>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send {
        async move {
            let count = args;
            if core::any::TypeId::of::<B>() == core::any::TypeId::of::<u8>() {
                let mut list = vec![0u8; count];
                reader
                    .read_exact(list.as_mut_slice())
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
}

impl<B, const N: usize> BinRead for [B; N]
where
    B: BinRead + Send,
    for<'a> B::Args<'a>: Clone,
{
    type Args<'a> = B::Args<'a>;

    fn read_options<R: Read + Seek + Send>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send {
        async move {
            let mut list = Vec::with_capacity(N);
            for _ in 0..N {
                list.push(B::read_options(reader, endian, args.clone()).await?);
            }
            Ok(list.try_into().ok().unwrap())
        }
    }
}

impl BinRead for () {
    type Args<'a> = ();

    fn read_options<R: Read + Seek + Send>(
        _: &mut R,
        _: Endian,
        (): Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send {
        async move { Ok(()) }
    }
}

impl<T: BinRead + Send> BinRead for Box<T> {
    type Args<'a> = T::Args<'a>;

    fn read_options<R: Read + Seek + Send>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send {
        async move { Ok(Box::new(T::read_options(reader, endian, args).await?)) }
    }
}

impl<T: BinRead + Send> BinRead for Option<T> {
    type Args<'a> = T::Args<'a>;

    fn read_options<R: Read + Seek + Send>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send {
        async move { Ok(Some(T::read_options(reader, endian, args).await?)) }
    }
}

impl<T: Send> BinRead for core::marker::PhantomData<T> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek + Send>(
        _: &mut R,
        _: Endian,
        (): Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send {
        async move { Ok(core::marker::PhantomData) }
    }
}
impl BinRead for bool {
    type Args<'a> = ();
    fn read_options<R: Read + Seek + Send>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send {
        async move {
            let value: u8 = reader.read_type_args(endian, ()).await?;
            Ok(value != 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{Read, Seek, Write};
    use crate::{BinRead, BinReaderExt, BinResult, Endian};
    use anyhow::Result;
    use std::io::{Cursor, SeekFrom};
    use std::marker::PhantomData;

    #[tokio::test]
    async fn test_read_slice() -> Result<()> {
        Ok(())
    }
    trait Config: Sync + Send + Clone {
        fn size(&self) -> usize;
    }
    trait StreamDefault: Sized {
        type Config;
        fn config(&self) -> &Self::Config;
        fn from_config(config: &Self::Config) -> impl Future<Output = BinResult<Self>> + Send;
        fn from_ref_config(
            _pos: u64,
            _size: u64,
            config: &Self::Config,
        ) -> impl Future<Output = BinResult<(Self, bool)>> + Send {
            let fut = Self::from_config(config);
            async move {
                let ret = fut.await?;
                Ok((ret, true))
            }
        }
    }
    struct Dir<T>
    where
        T: Read + Write + Seek + StreamDefault,
        T::Config: Config + 'static,
    {
        _marker: std::marker::PhantomData<T>,
        age: u32,
        b: String,
        data: Arc<async_lock::Mutex<T>>,
    }
    pub struct MyData {
        config: MyConfig,
        inner: Cursor<Vec<u8>>,
    }
    impl Default for MyData {
        fn default() -> Self {
            MyData {
                config: Default::default(),
                inner: Cursor::new(vec![]),
            }
        }
    }
    impl Write for MyData {
        fn write(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<usize>> + Send {
            async { std::io::Write::write(&mut self.inner, buf) }
        }

        fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send {
            async { Ok(()) }
        }
    }
    impl StreamDefault for MyData {
        type Config = MyConfig;

        fn config(&self) -> &Self::Config {
            &self.config
        }

        fn from_config(config: &Self::Config) -> impl Future<Output = BinResult<Self>> + Send {
            async move {
                Ok(Self {
                    inner: Cursor::new(vec![]),
                    config: config.clone(),
                })
            }
        }
    }
    impl Seek for MyData {
        fn seek(&mut self, pos: SeekFrom) -> impl Future<Output = std::io::Result<u64>> + Send {
            async move { std::io::Seek::seek(&mut self.inner, pos) }
        }
    }
    impl Read for MyData {
        fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = std::io::Result<usize>> + Send {
            async { std::io::Read::read(&mut self.inner, buf) }
        }

        fn flush(&mut self) -> impl Future<Output = std::io::Result<()>> + Send {
            async { Ok(()) }
        }
    }
    #[derive(Default, Clone)]
    struct MyConfig {
        size: usize,
    }
    impl Config for MyConfig {
        fn size(&self) -> usize {
            self.size
        }
    }
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};

    pub type ReadBytesFun<'a> =
        dyn FnMut(u64) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + 'a;
    impl<T> BinRead for Dir<T>
    where
        T: Read + Write + Seek + Send + StreamDefault,
        T::Config: Config + 'static,
    {
        type Args<'a> = (u16, &'a T::Config, &'a mut ReadBytesFun<'a>);

        fn read_options<R: Read + Seek + Send>(
            reader: &mut R,
            endian: Endian,
            args: Self::Args<'_>,
        ) -> impl Future<Output = BinResult<Self>> + Send
        where
            Self: Send,
        {
            async move {
                let (index, config, callback) = args;
                callback(1).await;
                // let mut config = config.clone();
                let (mut data, _) = T::from_ref_config(0, 0, config).await?;
                let mut data = T::from_config(config).await?;
                use crate::io::write::Write;
                data.write_all(vec![1u8].as_slice()).await?;
                // let reader = data.take(1);
                let dir = Dir {
                    _marker: PhantomData,
                    age: 0,
                    b: "".to_string(),
                    data: Arc::new(async_lock::Mutex::new(data)),
                };
                Self::write_data(dir.data.clone()).await;
                Ok(dir)
            }
        }
    }
    // unsafe impl<T> Sync for Dir<T>
    // where
    //     T: Read + Write + Seek + Send + StreamDefault,
    //     T::Config: Config + 'static,
    // {
    // }
    impl<T> Dir<T>
    where
        T: Read + Write + Seek + Send + StreamDefault,
        T::Config: Config + 'static,
    {
        pub fn write_data(data: Arc<async_lock::Mutex<T>>) -> impl Future<Output = ()> + Send {
            async move {
                let mut d = data.lock().await;
                let pos = d.stream_position().await.unwrap();
            }
        }
        // #[async_recursion(?Send)]
        // pub fn try_clone<'a>(
        //     &'a self,
        //     config: &'a T::Config,
        // ) -> Pin<Box<dyn Future<Output = BinResult<()>> + Send + 'a>> {
        //     let data = self.data.clone();
        //     // let config = config.clone(); // config is reference, captured by async move
        //     Box::pin(async move {
        //         // let mut data = data.lock().await;
        //         // let size = data.seek(SeekFrom::End(0)).await?; // stream_length(&mut *data).await?;
        //         // let pos = data.stream_position().await?;
        //         let mut new_data = T::from_config(config).await?;
        //         // if true {
        //         self.try_clone(config).await?;
        //         // }
        //         Ok(())
        //     })
        // }
        pub fn clear(self) -> impl Future<Output = ()> + Send {
            // let data = self.data.clone();
            async move {
                let mut guard = self.data.lock().await;
                // if guard.is_some() {
                //     let inner_data = guard.take().unwrap();
                //     let new_dir = Self::rebuild(inner_data).await;
                // }
                ()
            }
        }
        pub fn rebuild(data: T) -> impl Future<Output = Self> + Send {
            async move {
                Dir::<T> {
                    _marker: PhantomData,
                    age: 0,
                    b: "".to_string(),
                    data: Arc::new(async_lock::Mutex::new(data)),
                }
            }
        }
        pub fn decompressed_callback<'a>(
            &'a mut self,
            callback_fun: &'a mut ReadBytesFun<'a>,
        ) -> impl Future<Output = BinResult<()>> + Send + 'a {
            async move { Ok(()) }
        }
    }
    fn create_adapter<'a, CB>(
        total: u64,
        sum: &'a mut u64,
        mut cb: CB,
    ) -> impl FnMut(u64) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + 'a
    where
        CB: FnMut(u64, u64) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + 'a,
    {
        move |bytes| {
            *sum += bytes;
            cb(total, *sum)
        }
    }

    async fn call_generic<'a, F>(callback: F) -> Result<()>
    where
        F: FnMut(u64, u64) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + 'a,
    {
        let mut cursor = Cursor::new(vec![]);
        let config = MyConfig { size: 0 };
        let total_size = 100;
        let mut bytes_count = 0;
        {
            let mut adapter = create_adapter(total_size, &mut bytes_count, callback);

            let dir: Dir<MyData> = cursor
                .read_le_args((1, &config, &mut adapter as &mut ReadBytesFun))
                .await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_generic() -> Result<()> {
        call_generic(|total, read_bytes| {
            Box::pin(async move {
                // println!("Total: {}, Read: {}", total, read_bytes);
            })
        })
        .await?;
        Ok(())
    }
}
