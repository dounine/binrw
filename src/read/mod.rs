pub mod impls;

use crate::Required;
use crate::io::read::Read;
use crate::io::seek::Seek;
use crate::{BinResult, Endian};

pub trait BinRead: Sized {
    type Args<'a>: Send;

    fn read<R: Read + Seek + Send>(reader: &mut R) -> impl Future<Output = BinResult<Self>> + Send
    where
        Self: Send,
        for<'a> Self::Args<'a>: Required,
    {
        Self::read_args(reader, Self::Args::args())
    }

    fn read_be<R: Read + Seek + Send>(
        reader: &mut R,
    ) -> impl Future<Output = BinResult<Self>> + Send
    where
        Self: Send,
        for<'a> Self::Args<'a>: Required,
    {
        Self::read_be_args(reader, Self::Args::args())
    }

    fn read_le<R: Read + Seek + Send>(
        reader: &mut R,
    ) -> impl Future<Output = BinResult<Self>> + Send
    where
        Self: Send,
        for<'a> Self::Args<'a>: Required,
    {
        Self::read_le_args(reader, Self::Args::args())
    }

    fn read_ne<R: Read + Seek + Send>(
        reader: &mut R,
    ) -> impl Future<Output = BinResult<Self>> + Send
    where
        Self: Send,
        for<'a> Self::Args<'a>: Required,
    {
        Self::read_ne_args(reader, Self::Args::args())
    }

    fn read_args<R: Read + Seek + Send>(
        reader: &mut R,
        args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send
    where
        Self: Send,
    {
        Self::read_options(reader, Endian::Little, args)
    }

    fn read_be_args<R: Read + Seek + Send>(
        reader: &mut R,
        args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send
    where
        Self: Send,
    {
        Self::read_options(reader, Endian::Big, args)
    }

    fn read_le_args<R: Read + Seek + Send>(
        reader: &mut R,
        args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send
    where
        Self: Send,
    {
        Self::read_options(reader, Endian::Little, args)
    }

    fn read_ne_args<R: Read + Seek + Send>(
        reader: &mut R,
        args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send
    where
        Self: Send,
    {
        Self::read_options(reader, Endian::NATIVE, args)
    }

    fn read_options<R: Read + Seek + Send>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send
    where
        Self: Send;
}

pub trait BinReaderExt: Read + Seek + Sized + Send {
    fn read_type<'a, T>(&mut self, endian: Endian) -> impl Future<Output = BinResult<T>> + Send
    where
        T: BinRead + Send,
        T::Args<'a>: Required,
    {
        self.read_type_args(endian, T::Args::args())
    }

    fn read_be<'a, T>(&mut self) -> impl Future<Output = BinResult<T>> + Send
    where
        T: BinRead + Send,
        T::Args<'a>: Required,
    {
        self.read_type(Endian::Big)
    }

    fn read_le<'a, T>(&mut self) -> impl Future<Output = BinResult<T>> + Send
    where
        T: BinRead + Send,
        T::Args<'a>: Required,
    {
        self.read_type(Endian::Little)
    }

    fn read_ne<'a, T>(&mut self) -> impl Future<Output = BinResult<T>> + Send
    where
        T: BinRead + Send,
        T::Args<'a>: Required,
    {
        self.read_type(Endian::NATIVE)
    }

    fn read_type_args<T>(
        &mut self,
        endian: Endian,
        args: T::Args<'_>,
    ) -> impl Future<Output = BinResult<T>> + Send
    where
        T: BinRead + Send,
    {
        T::read_options(self, endian, args)
    }

    fn read_be_args<T>(&mut self, args: T::Args<'_>) -> impl Future<Output = BinResult<T>> + Send
    where
        T: BinRead + Send,
    {
        self.read_type_args(Endian::Big, args)
    }

    fn read_le_args<T>(&mut self, args: T::Args<'_>) -> impl Future<Output = BinResult<T>> + Send
    where
        T: BinRead + Send,
    {
        self.read_type_args(Endian::Little, args)
    }

    fn read_ne_args<T>(&mut self, args: T::Args<'_>) -> impl Future<Output = BinResult<T>> + Send
    where
        T: BinRead + Send,
    {
        self.read_type_args(Endian::NATIVE, args)
    }
}

impl<R: Read + Seek + Sized + Send> BinReaderExt for R {}
