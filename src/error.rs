use core::fmt;
use std::any::Any;
use std::borrow::Cow;
use crate::backtrace::{Backtrace, BacktraceFrame};

pub type BinResult<T> = Result<T, Error>;
mod private {
    use core::fmt;
    pub trait Sealed {}
    impl<T: fmt::Display + fmt::Debug + Send + Sync + 'static> Sealed for T {}
}
pub trait ContextExt {
    /// Adds a new context frame to the error, consuming the original error.
    #[must_use]
    fn with_context<Frame: Into<BacktraceFrame>>(self, frame: Frame) -> Self;

    /// Adds a new frame of context to the error with the given message,
    /// consuming the original error.
    ///
    /// This also adds the file name and line number of the caller to the error.
    #[must_use]
    #[track_caller]
    fn with_message(self, message: impl Into<Cow<'static, str>>) -> Self;
}

impl ContextExt for Error {
    fn with_context<Frame: Into<BacktraceFrame>>(self, frame: Frame) -> Self {
        match self {
            Error::Backtrace(mut backtrace) => {
                backtrace.frames.push(frame.into());
                Error::Backtrace(backtrace)
            }
            error => Error::Backtrace(Backtrace::new(error, vec![frame.into()])),
        }
    }

    #[track_caller]
    fn with_message(self, message: impl Into<Cow<'static, str>>) -> Self {
        match self {
            Error::Backtrace(backtrace) => Error::Backtrace(backtrace.with_message(message)),
            error => {
                let caller = core::panic::Location::caller();
                Error::Backtrace(Backtrace::new(
                    error,
                    vec![BacktraceFrame::Full {
                        code: None,
                        message: message.into(),
                        file: caller.file(),
                        line: caller.line(),
                    }],
                ))
            }
        }
    }
}

impl<T> ContextExt for BinResult<T> {
    fn with_context<Frame: Into<BacktraceFrame>>(self, frame: Frame) -> Self {
        self.map_err(|err| err.with_context(frame))
    }

    #[track_caller]
    fn with_message(self, message: impl Into<Cow<'static, str>>) -> Self {
        match self {
            Err(err) => {
                let caller = core::panic::Location::caller();
                Err(match err {
                    Error::Backtrace(backtrace) => {
                        Error::Backtrace(backtrace.with_message(message))
                    }
                    error => Error::Backtrace(Backtrace::new(
                        error,
                        vec![BacktraceFrame::Full {
                            code: None,
                            message: message.into(),
                            file: caller.file(),
                            line: caller.line(),
                        }],
                    )),
                })
            }
            ok => ok,
        }
    }
}
pub trait CustomError: fmt::Display + fmt::Debug + Send + Sync + private::Sealed {
    #[doc(hidden)]
    fn as_any(&self) -> &(dyn Any + Send + Sync);

    #[doc(hidden)]
    fn as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync);

    #[doc(hidden)]
    fn as_box_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
}

impl<T: fmt::Display + fmt::Debug + Send + Sync + 'static> CustomError for T {
    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync) {
        self
    }

    fn as_box_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }
}

impl dyn CustomError {
    #[allow(clippy::missing_panics_doc)]
    pub fn downcast<T: CustomError + 'static>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if self.is::<T>() {
            Ok(self.as_box_any().downcast().unwrap())
        } else {
            Err(self)
        }
    }

    pub fn downcast_mut<T: CustomError + 'static>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut()
    }

    pub fn downcast_ref<T: CustomError + 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }

    pub fn is<T: CustomError + 'static>(&self) -> bool {
        core::any::TypeId::of::<T>() == self.as_any().type_id()
    }
}

pub enum Error {
    BadMagic {
        pos: u64,
        found: Box<dyn fmt::Debug + Send + Sync>,
    },
    AssertFail {
        pos: u64,
        message: String,
    },

    Io(std::io::Error),

    Custom {
        pos: u64,
        err: Box<dyn CustomError>,
    },

    NoVariantMatch {
        pos: u64,
    },

    EnumErrors {
        pos: u64,
        variant_errors: Vec<(&'static str, Error)>,
    },

    Backtrace(Backtrace),
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMagic { pos, found } => write!(f, "bad magic at 0x{pos:x}: {found:?}"),
            Self::AssertFail { pos, message } => write!(f, "{message} at 0x{pos:x}"),
            Self::Io(err) => fmt::Display::fmt(err, f),
            Self::Custom { pos, err } => write!(f, "{err} at 0x{pos:x}"),
            Self::NoVariantMatch { pos } => write!(f, "no variants matched at 0x{pos:x}"),
            Self::EnumErrors {
                pos,
                variant_errors,
            } => {
                write!(f, "no variants matched at 0x{pos:x}:")?;
                for (name, err) in variant_errors {
                    write!(f, "\n  {name}: {err}")?;
                }
                Ok(())
            }
            Self::Backtrace(backtrace) => fmt::Display::fmt(backtrace, f),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Error as fmt::Display>::fmt(self, f)
    }
}
impl std::error::Error for Error {}