pub
mod impls;

use async_trait::async_trait;
use crate::Required;
use crate::{BinResult, Endian};
use crate::io::read::Read;
use crate::io::seek::Seek;

pub trait BinRead: Sized {
    type Args<'a>;
    

    #[inline]
    async fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self>
    where
        for<'a> Self::Args<'a>: Required,
    {
        Self::read_args(reader, Self::Args::args()).await
    }

    #[inline]
    async fn read_be<R: Read + Seek>(reader: &mut R) -> BinResult<Self>
    where
        for<'a> Self::Args<'a>: Required,
    {
        Self::read_be_args(reader, Self::Args::args()).await
    }

    #[inline]
    async fn read_le<R: Read + Seek>(reader: &mut R) -> BinResult<Self>
    where
        for<'a> Self::Args<'a>: Required,
    {
        Self::read_le_args(reader, Self::Args::args()).await
    }

    #[inline]
    async fn read_ne<R: Read + Seek>(reader: &mut R) -> BinResult<Self>
    where
        for<'a> Self::Args<'a>: Required,
    {
        Self::read_ne_args(reader, Self::Args::args()).await
    }

    #[inline]
    async fn read_args<R: Read + Seek>(reader: &mut R, args: Self::Args<'_>) -> BinResult<Self> {
        Self::read_options(reader, Endian::Little, args).await
    }

    #[inline]
    async fn read_be_args<R: Read + Seek>(reader: &mut R, args: Self::Args<'_>) -> BinResult<Self> {
        Self::read_options(reader, Endian::Big, args).await
    }

    #[inline]
    async fn read_le_args<R: Read + Seek>(reader: &mut R, args: Self::Args<'_>) -> BinResult<Self> {
        Self::read_options(reader, Endian::Little, args).await
    }

    #[inline]
    async fn read_ne_args<R: Read + Seek>(reader: &mut R, args: Self::Args<'_>) -> BinResult<Self> {
        Self::read_options(reader, Endian::NATIVE, args).await
    }

    async fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self>;
}

pub trait BinReaderExt: Read + Seek + Sized {
    #[inline]
    async fn read_type<'a, T>(&mut self, endian: Endian) -> BinResult<T>
    where
        T: BinRead,
        T::Args<'a>: Required,
    {
        self.read_type_args(endian, T::Args::args()).await
    }

    #[inline]
    async fn read_be<'a, T>(&mut self) -> BinResult<T>
    where
        T: BinRead,
        T::Args<'a>: Required,
    {
        self.read_type(Endian::Big).await
    }

    #[inline]
    async fn read_le<'a, T>(&mut self) -> BinResult<T>
    where
        T: BinRead,
        T::Args<'a>: Required,
    {
        self.read_type(Endian::Little).await
    }

    #[inline]
    async fn read_ne<'a, T>(&mut self) -> BinResult<T>
    where
        T: BinRead,
        T::Args<'a>: Required,
    {
        self.read_type(Endian::NATIVE).await
    }

    #[inline]
    async fn read_type_args<T>(&mut self, endian: Endian, args: T::Args<'_>) -> BinResult<T>
    where
        T: BinRead,
    {
        T::read_options(self, endian, args).await
    }

    #[inline]
    async fn read_be_args<T>(&mut self, args: T::Args<'_>) -> BinResult<T>
    where
        T: BinRead,
    {
        self.read_type_args(Endian::Big, args).await
    }

    #[inline]
    async fn read_le_args<T>(&mut self, args: T::Args<'_>) -> BinResult<T>
    where
        T: BinRead,
    {
        self.read_type_args(Endian::Little, args).await
    }

    #[inline]
    async fn read_ne_args<T>(&mut self, args: T::Args<'_>) -> BinResult<T>
    where
        T: BinRead,
    {
        self.read_type_args(Endian::NATIVE, args).await
    }
}

impl<R: Read + Seek + Sized> BinReaderExt for R {}
