mod impls;

use crate::io::seek::Seek;
use crate::io::write::Write;
use crate::{BinResult, Endian, Required};

pub trait BinWrite {
    type Args<'a>;

    #[inline]
    async fn write<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()>
    where
        for<'a> Self::Args<'a>: Required,
    {
        self.write_args(writer, Self::Args::args()).await
    }

    #[inline]
    async fn write_be<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()>
    where
        for<'a> Self::Args<'a>: Required,
    {
        self.write_be_args(writer, Self::Args::args()).await
    }

    #[inline]
    async fn write_le<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()>
    where
        for<'a> Self::Args<'a>: Required,
    {
        self.write_le_args(writer, Self::Args::args()).await
    }

    #[inline]
    async fn write_ne<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()>
    where
        for<'a> Self::Args<'a>: Required,
    {
        self.write_ne_args(writer, Self::Args::args()).await
    }

    #[inline]
    async fn write_args<W: Write + Seek>(
        &self,
        writer: &mut W,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        self.write_options(writer, Endian::Little, args).await
    }

    #[inline]
    async fn write_be_args<W: Write + Seek>(
        &self,
        writer: &mut W,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        self.write_options(writer, Endian::Big, args).await
    }

    #[inline]
    async fn write_le_args<W: Write + Seek>(
        &self,
        writer: &mut W,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        self.write_options(writer, Endian::Little, args).await
    }

    #[inline]
    async fn write_ne_args<W: Write + Seek>(
        &self,
        writer: &mut W,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        self.write_options(writer, Endian::NATIVE, args).await
    }

    async fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()>;
}

pub trait BinWriterExt: Write + Seek + Sized {
    #[inline]
    async fn write_type<T: BinWrite>(&mut self, value: &T, endian: Endian) -> BinResult<()>
    where
        for<'a> T::Args<'a>: Required,
    {
        self.write_type_args(value, endian, T::Args::args()).await
    }

    #[inline]
    async fn write_be<T: BinWrite>(&mut self, value: &T) -> BinResult<()>
    where
        for<'a> T::Args<'a>: Required,
    {
        self.write_type(value, Endian::Big).await
    }

    #[inline]
    async fn write_le<T: BinWrite>(&mut self, value: &T) -> BinResult<()>
    where
        for<'a> T::Args<'a>: Required,
    {
        self.write_type(value, Endian::Little).await
    }

    #[inline]
    async fn write_ne<T: BinWrite>(&mut self, value: &T) -> BinResult<()>
    where
        for<'a> T::Args<'a>: Required,
    {
        self.write_type(value, Endian::NATIVE).await
    }

    #[inline]
    async fn write_type_args<T: BinWrite>(
        &mut self,
        value: &T,
        endian: Endian,
        args: T::Args<'_>,
    ) -> BinResult<()> {
        T::write_options(value, self, endian, args).await
    }

    #[inline]
    async fn write_be_args<T: BinWrite>(&mut self, value: &T, args: T::Args<'_>) -> BinResult<()> {
        self.write_type_args(value, Endian::Big, args).await
    }

    #[inline]
    async fn write_le_args<T: BinWrite>(&mut self, value: &T, args: T::Args<'_>) -> BinResult<()> {
        self.write_type_args(value, Endian::Little, args).await
    }

    #[inline]
    async fn write_ne_args<T: BinWrite>(&mut self, value: &T, args: T::Args<'_>) -> BinResult<()> {
        self.write_type_args(value, Endian::NATIVE, args).await
    }
}

impl<W: Write + Seek + Sized> BinWriterExt for W {}
