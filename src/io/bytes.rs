use std::{pin::Pin};

use crate::BinResult;

pub trait BytesCallback {
    type Future<'a>: Future<Output = BinResult<()>> + Send + 'a
    where
        Self: 'a;
    fn call<'a>(&mut self, bytes: u64) -> Self::Future<'a>
    where
        Self: 'a;
}

pub trait TotalBytesCallback {
    type Future<'a>: Future<Output = BinResult<()>> + Send + 'a
    where
        Self: 'a;
    fn call<'a>(&mut self, bytes: u64, total: u64) -> Self::Future<'a>
    where
        Self: 'a;
}
pub struct NullBytesCallback;

impl BytesCallback for NullBytesCallback {
    type Future<'a>
        = Pin<Box<dyn Future<Output = BinResult<()>> + Send + 'a>>
    where
        Self: 'a;

    fn call<'a>(&mut self, _bytes: u64) -> Self::Future<'a> {
        Box::pin(async move { Ok(()) })
    }
}
// /// 将 `FnMut(u64)` 闭包包装成 `BytesCallback`，避免与 `&mut C` 的通用转发实现冲突。
pub struct BytesCallbackFn<F>(F);

impl<F> BytesCallbackFn<F> {
    pub fn new(inner: F) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> F {
        self.0
    }
}

impl<F> BytesCallback for BytesCallbackFn<F>
where
    F: FnMut(u64) -> Pin<Box<dyn Future<Output = BinResult<()>> + Send>> + Send,
{
    type Future<'a>
        = Pin<Box<dyn Future<Output = BinResult<()>> + Send + 'a>>
    where
        Self: 'a;

    fn call<'a>(&mut self, bytes: u64) -> Self::Future<'a>
    where
        Self: 'a,
        F: 'a,
    {
        (self.0)(bytes)
    }
}

// /// 将 `FnMut(u64, u64)` 闭包包装成 `TotalBytesCallback`。
pub struct TotalBytesCallbackFn<F>(F);

impl<F> TotalBytesCallbackFn<F> {
    pub fn new(inner: F) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> F {
        self.0
    }
}

impl<F> TotalBytesCallback for TotalBytesCallbackFn<F>
where
    F: FnMut(u64, u64) -> Pin<Box<dyn Future<Output = BinResult<()>> + Send>> + Send,
{
    type Future<'a>
        = Pin<Box<dyn Future<Output = BinResult<()>> + Send + 'a>>
    where
        Self: 'a;

    fn call<'a>(&mut self, bytes: u64, total: u64) -> Self::Future<'a>
    where
        Self: 'a,
    {
        (self.0)(bytes, total)
    }
}

impl<C> BytesCallback for &mut C
where
    C: BytesCallback + Send + ?Sized,
{
    type Future<'a>
        = C::Future<'a>
    where
        Self: 'a;

    fn call<'a>(&mut self, bytes: u64) -> Self::Future<'a>
    where
        Self: 'a,
    {
        (**self).call(bytes)
    }
}

impl<C> TotalBytesCallback for &mut C
where
    C: TotalBytesCallback + Send + ?Sized,
{
    type Future<'a>
        = C::Future<'a>
    where
        Self: 'a;

    fn call<'a>(&mut self, bytes: u64, total: u64) -> Self::Future<'a>
    where
        Self: 'a,
    {
        (**self).call(bytes, total)
    }
}

pub struct BytesToTotalAdapter<'a, C> {
    inner: &'a mut C,
    total: u64,
    sum: u64,
}
impl<'a, C> BytesToTotalAdapter<'a, C> {
    pub fn new(total: u64, callback: &'a mut C) -> Self {
        Self {
            inner: callback,
            total,
            sum: 0,
        }
    }
}
impl<'a, C> BytesCallback for BytesToTotalAdapter<'a, C>
where
    C: TotalBytesCallback + Send,
{
    type Future<'m>
        = C::Future<'m>
    where
        Self: 'm;

    fn call<'m>(&mut self, bytes: u64) -> Self::Future<'m>
    where
        Self: 'm,
    {
        self.sum += bytes;
        self.inner.call(self.sum, self.total)
    }
}

pub struct NullBytesTotalCallback;

impl TotalBytesCallback for NullBytesTotalCallback {
    type Future<'a>
        = Pin<Box<dyn Future<Output = BinResult<()>> + Send + 'a>>
    where
        Self: 'a;

    fn call<'a>(&mut self, _bytes: u64, _total: u64) -> Self::Future<'a> {
        Box::pin(async move { Ok(()) })
    }
}
// 辅助函数，让创建回调更简单
pub fn make_callback<'a, Fut, F>(
    mut f: F,
) -> BytesCallbackFn<impl FnMut(u64) -> Pin<Box<dyn Future<Output = BinResult<()>> + Send + 'a>>>
where
    Fut: Future<Output = BinResult<()>> + Send + 'a,
    F: FnMut(u64) -> Fut + Send + 'a,
{
    BytesCallbackFn::new(move |bytes| {
        let fut: Pin<Box<dyn Future<Output = BinResult<()>> + Send>> = Box::pin(f(bytes));
        fut
    })
}
// 辅助函数，让创建回调更简单
pub fn make_total_callback<'a, Fut, F>(
    mut f: F,
) -> TotalBytesCallbackFn<impl FnMut(u64, u64) -> Pin<Box<dyn Future<Output = BinResult<()>> + Send + 'a>>>
where
    Fut: Future<Output = BinResult<()>> + Send + 'a,
    F: FnMut(u64, u64) -> Fut + Send + 'a,
{
    TotalBytesCallbackFn::new(move |bytes, total| {
        let fut: Pin<Box<dyn Future<Output = BinResult<()>> + Send>> = Box::pin(f(bytes, total));
        fut
    })
}
