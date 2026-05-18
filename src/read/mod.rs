pub mod impls;

use crate::io::read::Read;
use crate::io::seek::Seek;
use crate::{BinResult, Endian};

pub trait BinRead: Sized {
    type Args<'a>: Send
    where
        Self: 'a;

    fn read<'a, 'r, R>(reader: &'r mut R) -> impl Future<Output = BinResult<Self>> + Send + 'r
    where
        'a: 'r,
        R: Read + Seek + Send,
        Self: Send + 'a,
        Self::Args<'a>: std::default::Default,
    {
        Self::read_args(reader, Self::Args::default())
    }

    fn read_be<'a, 'r, R>(reader: &'r mut R) -> impl Future<Output = BinResult<Self>> + Send + 'r
    where
        'a: 'r,
        R: Read + Seek + Send,
        Self: Send + 'a,
        Self::Args<'a>: std::default::Default,
    {
        Self::read_be_args(reader, Self::Args::default())
    }

    fn read_le<'a, 'r, R>(reader: &'r mut R) -> impl Future<Output = BinResult<Self>> + Send + 'r
    where
        'a: 'r,
        R: Read + Seek + Send,
        Self: Send + 'a,
        Self::Args<'a>: std::default::Default,
    {
        Self::read_le_args(reader, Self::Args::default())
    }

    fn read_ne<'a, 'r, R>(reader: &'r mut R) -> impl Future<Output = BinResult<Self>> + Send + 'r
    where
        'a: 'r,
        R: Read + Seek + Send,
        Self: Send + 'a,
        Self::Args<'a>: std::default::Default,
    {
        Self::read_ne_args(reader, Self::Args::default())
    }

    fn read_args<'a, 'r, R>(
        reader: &'r mut R,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<Self>> + Send + 'r
    where
        'a: 'r,
        R: Read + Seek + Send,
        Self: Send + 'a,
    {
        Self::read_options(reader, Endian::Little, args)
    }

    fn read_be_args<'a, 'r, R>(
        reader: &'r mut R,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<Self>> + Send + 'r
    where
        'a: 'r,
        R: Read + Seek + Send,
        Self: Send + 'a,
    {
        Self::read_options(reader, Endian::Big, args)
    }

    fn read_le_args<'a, 'r, R>(
        reader: &'r mut R,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<Self>> + Send + 'r
    where
        'a: 'r,
        R: Read + Seek + Send,
        Self: Send + 'a,
    {
        Self::read_options(reader, Endian::Little, args)
    }

    fn read_ne_args<'a, 'r, R>(
        reader: &'r mut R,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<Self>> + Send + 'r
    where
        'a: 'r,
        R: Read + Seek + Send,
        Self: Send + 'a,
    {
        Self::read_options(reader, Endian::NATIVE, args)
    }

    fn read_options<'a, 'r, R>(
        reader: &'r mut R,
        endian: Endian,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<Self>> + Send + 'r
    where
        'a: 'r,
        R: Read + Seek + Send,
        Self: Send + 'a;
}

pub trait BinReaderExt: Read + Seek + Sized + Send {
    fn read_type<'a, 'r, T>(
        &'r mut self,
        endian: Endian,
    ) -> impl Future<Output = BinResult<T>> + Send + 'r
    where
        'a: 'r,
        T: BinRead + Send + 'a,
        T::Args<'a>: std::default::Default,
    {
        self.read_type_args(endian, T::Args::default())
    }

    fn read_be<'a, 'r, T>(&'r mut self) -> impl Future<Output = BinResult<T>> + Send + 'r
    where
        'a: 'r,
        T: BinRead + Send + 'a,
        T::Args<'a>: std::default::Default,
    {
        self.read_type(Endian::Big)
    }

    fn read_le<'a, 'r, T>(&'r mut self) -> impl Future<Output = BinResult<T>> + Send + 'r
    where
        'a: 'r,
        T: BinRead + Send + 'a,
        T::Args<'a>: std::default::Default,
    {
        self.read_type(Endian::Little)
    }

    fn read_ne<'a, 'r, T>(&'r mut self) -> impl Future<Output = BinResult<T>> + Send + 'r
    where
        'a: 'r,
        T: BinRead + Send + 'a,
        T::Args<'a>: std::default::Default,
    {
        self.read_type(Endian::NATIVE)
    }

    fn read_type_args<'a, 'r, T>(
        &'r mut self,
        endian: Endian,
        args: T::Args<'a>,
    ) -> impl Future<Output = BinResult<T>> + Send + 'r
    where
        'a: 'r,
        T: BinRead + Send + 'a,
    {
        T::read_options(self, endian, args)
    }

    fn read_be_args<'a, 'r, T>(
        &'r mut self,
        args: T::Args<'a>,
    ) -> impl Future<Output = BinResult<T>> + Send + 'r
    where
        'a: 'r,
        T: BinRead + Send + 'a,
    {
        self.read_type_args(Endian::Big, args)
    }

    fn read_le_args<'a, 'r, T>(
        &'r mut self,
        args: T::Args<'a>,
    ) -> impl Future<Output = BinResult<T>> + Send + 'r
    where
        'a: 'r,
        T: BinRead + Send + 'a,
    {
        self.read_type_args(Endian::Little, args)
    }

    fn read_ne_args<'a, 'r, T>(
        &'r mut self,
        args: T::Args<'a>,
    ) -> impl Future<Output = BinResult<T>> + Send + 'r
    where
        'a: 'r,
        T: BinRead + Send + 'a,
    {
        self.read_type_args(Endian::NATIVE, args)
    }
}

impl<R: Read + Seek + Sized + Send> BinReaderExt for R {}
