mod impls;

use crate::io::seek::Seek;
use crate::io::write::Write;
use crate::{BinResult, Endian, Required};

pub trait BinWrite {
    type Args<'a>: Send;

    fn write<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
        for<'a> Self::Args<'a>: Required,
    {
        self.write_args(writer, Self::Args::args())
    }

    fn write_be<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
        for<'a> Self::Args<'a>: Required,
    {
        self.write_be_args(writer, Self::Args::args())
    }

    fn write_le<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
        for<'a> Self::Args<'a>: Required,
    {
        self.write_le_args(writer, Self::Args::args())
    }

    fn write_ne<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
        for<'a> Self::Args<'a>: Required,
    {
        self.write_ne_args(writer, Self::Args::args())
    }

    fn write_args<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        args: Self::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
    {
        self.write_options(writer, Endian::Little, args)
    }

    fn write_be_args<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        args: Self::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
    {
        self.write_options(writer, Endian::Big, args)
    }

    fn write_le_args<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        args: Self::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
    {
        self.write_options(writer, Endian::Little, args)
    }

    fn write_ne_args<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        args: Self::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
    {
        self.write_options(writer, Endian::NATIVE, args)
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
    fn write_type<T: BinWrite + Sync>(
        &mut self,
        value: &T,
        endian: Endian,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Required + Send,
    {
        self.write_type_args(value, endian, T::Args::args())
    }

    fn write_be<T: BinWrite + Sync>(
        &mut self,
        value: &T,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Required + Send,
    {
        self.write_type(value, Endian::Big)
    }

    fn write_le<T: BinWrite + Sync>(
        &mut self,
        value: &T,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Required + Send,
    {
        self.write_type(value, Endian::Little)
    }

    fn write_ne<T: BinWrite + Sync>(
        &mut self,
        value: &T,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Required + Send,
    {
        self.write_type(value, Endian::NATIVE)
    }

    fn write_type_args<T: BinWrite + Sync>(
        &mut self,
        value: &T,
        endian: Endian,
        args: T::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Send,
    {
        T::write_options(value, self, endian, args)
    }

    fn write_be_args<T: BinWrite + Sync>(
        &mut self,
        value: &T,
        args: T::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Send,
    {
        self.write_type_args(value, Endian::Big, args)
    }

    fn write_le_args<T: BinWrite + Sync>(
        &mut self,
        value: &T,
        args: T::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Send,
    {
        self.write_type_args(value, Endian::Little, args)
    }

    fn write_ne_args<T: BinWrite + Sync>(
        &mut self,
        value: &T,
        args: T::Args<'_>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send
    where
        for<'a> T::Args<'a>: Send,
    {
        self.write_type_args(value, Endian::NATIVE, args)
    }
}

impl<W: Write + Seek + Sized + Send> BinWriterExt for W {}
