mod impls;

use crate::io::seek::Seek;
use crate::io::write::Write;
use crate::{BinResult, Endian, Required};

pub trait BinWrite {
    type Args<'a>: Send;

    #[inline]
    fn write<W: Write + Seek + Send>(&self, writer: &mut W) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
        for<'a> Self::Args<'a>: Required,
    {
        async move {
            self.write_args(writer, Self::Args::args()).await
        }
    }

    #[inline]
    fn write_be<W: Write + Seek + Send>(&self, writer: &mut W) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
        for<'a> Self::Args<'a>: Required,
    {
        async move {
            self.write_be_args(writer, Self::Args::args()).await
        }
    }

    #[inline]
    fn write_le<W: Write + Seek + Send>(&self, writer: &mut W) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
        for<'a> Self::Args<'a>: Required,
    {
        async move {
            self.write_le_args(writer, Self::Args::args()).await
        }
    }

    #[inline]
    fn write_ne<W: Write + Seek + Send>(&self, writer: &mut W) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
        for<'a> Self::Args<'a>: Required,
    {
        async move {
            self.write_ne_args(writer, Self::Args::args()).await
        }
    }

    #[inline]
    fn write_args<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        args: Self::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
    {
        async move {
            self.write_options(writer, Endian::Little, args).await
        }
    }

    #[inline]
    fn write_be_args<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        args: Self::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
    {
        async move {
            self.write_options(writer, Endian::Big, args).await
        }
    }

    #[inline]
    fn write_le_args<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        args: Self::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
    {
        async move {
            self.write_options(writer, Endian::Little, args).await
        }
    }

    #[inline]
    fn write_ne_args<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        args: Self::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
    {
        async move {
            self.write_options(writer, Endian::NATIVE, args).await
        }
    }

    fn write_options<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync;
}

pub trait BinWriterExt: Write + Seek + Sized + Send {
    #[inline]
    fn write_type<T: BinWrite + Sync>(&mut self, value: &T, endian: Endian) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Required + Send,
    {
        async move {
            self.write_type_args(value, endian, T::Args::args()).await
        }
    }

    #[inline]
    fn write_be<T: BinWrite + Sync>(&mut self, value: &T) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Required + Send,
    {
        async move {
            self.write_type(value, Endian::Big).await
        }
    }

    #[inline]
    fn write_le<T: BinWrite + Sync>(&mut self, value: &T) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Required + Send,
    {
        async move {
            self.write_type(value, Endian::Little).await
        }
    }

    #[inline]
    fn write_ne<T: BinWrite + Sync>(&mut self, value: &T) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Required + Send,
    {
        async move {
            self.write_type(value, Endian::NATIVE).await
        }
    }

    #[inline]
    fn write_type_args<T: BinWrite + Sync>(
        &mut self,
        value: &T,
        endian: Endian,
        args: T::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Send,
    {
        async move {
            T::write_options(value, self, endian, args).await
        }
    }

    #[inline]
    fn write_be_args<T: BinWrite + Sync>(&mut self, value: &T, args: T::Args<'_>) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Send,
    {
        async move {
            self.write_type_args(value, Endian::Big, args).await
        }
    }

    #[inline]
    fn write_le_args<T: BinWrite + Sync>(&mut self, value: &T, args: T::Args<'_>) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Send,
    {
        async move {
            self.write_type_args(value, Endian::Little, args).await
        }
    }

    #[inline]
    fn write_ne_args<T: BinWrite + Sync>(&mut self, value: &T, args: T::Args<'_>) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Send,
    {
        async move {
            self.write_type_args(value, Endian::NATIVE, args).await
        }
    }
}

impl<W: Write + Seek + Sized + Send> BinWriterExt for W {}
