mod impls;

use crate::io::seek::Seek;
use crate::io::write::Write;
use crate::{BinResult, Endian};

pub trait BinWrite {
    type Args<'a>: Send
    where
        Self: 'a;

    fn write<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
        Self::Args<'a>: std::default::Default,
    {
        self.write_args(writer, Self::Args::default())
    }

    fn write_be<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
        Self::Args<'a>: std::default::Default,
    {
        self.write_be_args(writer, Self::Args::default())
    }

    fn write_le<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
        Self::Args<'a>: std::default::Default,
    {
        self.write_le_args(writer, Self::Args::default())
    }

    fn write_ne<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
        Self::Args<'a>: std::default::Default,
    {
        self.write_ne_args(writer, Self::Args::default())
    }

    fn write_args<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
        args: Self::Args<'a>,
    ) -> impl std::future::Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
    {
        self.write_options(writer, Endian::Little, args)
    }

    fn write_be_args<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
    {
        self.write_options(writer, Endian::Big, args)
    }

    fn write_le_args<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
    {
        self.write_options(writer, Endian::Little, args)
    }

    fn write_ne_args<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a,
    {
        self.write_options(writer, Endian::NATIVE, args)
    }

    fn write_options<'a, 'w, W>(
        &'a self,
        writer: &'w mut W,
        endian: Endian,
        args: Self::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'w
    where
        'a: 'w,
        W: Write + Seek + Send,
        Self: Sync + 'a;
}

pub trait BinWriterExt: Write + Seek + Sized + Send {
    fn write_type<'a, 'v, T>(
        &'a mut self,
        value: &'v T,
        endian: Endian,
    ) -> impl Future<Output = BinResult<()>> + Send + 'a
    where
        'v: 'a,
        T: BinWrite + Sync,
        T::Args<'a>: std::default::Default + Send,
    {
        self.write_type_args(value, endian, T::Args::default())
    }

    fn write_be<'a, 't, T>(
        &'a mut self,
        value: &'t T,
    ) -> impl Future<Output = BinResult<()>> + Send + 'a
    where
        't: 'a,
        T: BinWrite + Sync,
        T::Args<'a>: std::default::Default + Send,
    {
        self.write_type(value, Endian::Big)
    }
    fn write_le<'a, 'v, T>(
        &'a mut self,
        value: &'v T,
    ) -> impl Future<Output = BinResult<()>> + Send + 'a
    where
        'v: 'a,
        T: BinWrite + Sync,
        T::Args<'a>: std::default::Default + Send,
    {
        self.write_type(value, Endian::Little)
    }

    fn write_ne<'a, 'v, T>(
        &'a mut self,
        value: &'v T,
    ) -> impl Future<Output = BinResult<()>> + Send + 'a
    where
        'v: 'a,
        T: BinWrite + Sync,
        T::Args<'a>: std::default::Default + Send,
    {
        self.write_type(value, Endian::NATIVE)
    }

    fn write_type_args<'a, 'v, T>(
        &'a mut self,
        value: &'v T,
        endian: Endian,
        args: T::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'a
    where
        'v: 'a,
        T: BinWrite + Sync,
    {
        T::write_options(value, self, endian, args)
    }
    fn write_be_args<'a, 'v, T>(
        &'a mut self,
        value: &'v T,
        args: T::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'a
    where
        'v: 'a,
        T: BinWrite + Sync,
    {
        self.write_type_args(value, Endian::Big, args)
    }
    fn write_le_args<'a, 'v, T>(
        &'a mut self,
        value: &'v T,
        args: T::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'a
    where
        'v: 'a,
        T: BinWrite + Sync,
    {
        self.write_type_args(value, Endian::Little, args)
    }
    fn write_ne_args<'a, 'v, T>(
        &'a mut self,
        value: &'v T,
        args: T::Args<'a>,
    ) -> impl Future<Output = BinResult<()>> + Send + 'a
    where
        'v: 'a,
        T: BinWrite + Sync,
    {
        self.write_type_args(value, Endian::NATIVE, args)
    }
}

impl<W: Write + Seek + Sized + Send> BinWriterExt for W {}
