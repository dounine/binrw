//! Type definitions for string readers.
extern crate alloc;
use crate::io::read::Read;
use crate::io::seek::Seek;
use crate::io::write::Write;
use crate::{BinRead, BinResult, BinWrite, Endian};
use core::fmt::{self};
use std::string::{FromUtf8Error, FromUtf16Error};

#[derive(Clone, Eq, PartialEq, Default)]
pub struct NullString(
    pub Vec<u8>,
);

impl BinRead for NullString {
    type Args<'a> = ();

    fn read_options<R: Read + Seek + Send>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<Self>> + Send
    where
        Self: Send,
    {
        async move {
            let mut values = vec![];
            loop {
                let val = u8::read_options(reader, endian, ()).await?;
                if val == 0 {
                    return Ok(Self(values));
                }
                values.push(val);
            }
        }
    }
}

impl BinWrite for NullString {
    type Args<'a> = ();

    fn write_options<W: Write + Seek + Send>(
        &self,
        writer: &mut W,
        _endian: Endian,
        _args: Self::Args<'_>,
    ) -> impl Future<Output = BinResult<()>> + Send
    where
        Self: Sync,
    {
        async {
            writer.write_all(self.0.as_slice()).await?;
            writer.write_all(&[0u8]).await?;
            Ok(())
        }
    }
}

impl From<&str> for NullString {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

impl From<String> for NullString {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl From<NullString> for Vec<u8> {
    fn from(s: NullString) -> Self {
        s.0
    }
}

impl TryFrom<NullString> for String {
    type Error = FromUtf8Error;

    fn try_from(value: NullString) -> Result<Self, Self::Error> {
        String::from_utf8(value.0)
    }
}

impl core::ops::Deref for NullString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for NullString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for NullString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NullString(\"")?;
        display_utf8(&self.0, f, str::escape_debug)?;
        write!(f, "\")")
    }
}

impl fmt::Display for NullString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_utf8(&self.0, f, str::chars)
    }
}

#[derive(Clone, Eq, PartialEq, Default)]
pub struct NullWideString(
    /// The raw wide byte string.
    pub Vec<u16>,
);

impl BinRead for NullWideString {
    type Args<'a> = ();

    async fn read_options<R: Read + Seek + std::marker::Send>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut values = vec![];

        loop {
            let val = <u16>::read_options(reader, endian, ()).await?;
            if val == 0 {
                return Ok(Self(values));
            }
            values.push(val);
        }
    }
}

impl BinWrite for NullWideString {
    type Args<'a> = ();

    async fn write_options<W: Write + Seek + std::marker::Send>(
        &self,
        writer: &mut W,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        for &val in &self.0 {
            let bytes = match endian {
                Endian::Big => val.to_be_bytes(),
                Endian::Little => val.to_le_bytes(),
            };
            writer.write_all(&bytes).await?;
        }
        let null_bytes = match endian {
            Endian::Big => 0u16.to_be_bytes(),
            Endian::Little => 0u16.to_le_bytes(),
        };
        writer.write_all(&null_bytes).await?;

        Ok(())
    }
}

impl From<NullWideString> for Vec<u16> {
    fn from(s: NullWideString) -> Self {
        s.0
    }
}

impl From<&str> for NullWideString {
    fn from(s: &str) -> Self {
        Self(s.encode_utf16().collect())
    }
}

impl From<String> for NullWideString {
    fn from(s: String) -> Self {
        Self(s.encode_utf16().collect())
    }
}

impl TryFrom<NullWideString> for String {
    type Error = FromUtf16Error;

    fn try_from(value: NullWideString) -> Result<Self, Self::Error> {
        String::from_utf16(&value.0)
    }
}

impl core::ops::Deref for NullWideString {
    type Target = Vec<u16>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for NullWideString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for NullWideString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_utf16(&self.0, f, core::iter::once)
    }
}

impl fmt::Debug for NullWideString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NullWideString(\"")?;
        display_utf16(&self.0, f, char::escape_debug)?;
        write!(f, "\")")
    }
}

fn display_utf16<Transformer: Fn(char) -> O, O: Iterator<Item = char>>(
    input: &[u16],
    f: &mut fmt::Formatter<'_>,
    t: Transformer,
) -> fmt::Result {
    use std::fmt::Write;
    char::decode_utf16(input.iter().copied())
        .flat_map(|r| t(r.unwrap_or(char::REPLACEMENT_CHARACTER)))
        .try_for_each(|c| f.write_char(c))
}

fn display_utf8<'a, Transformer: Fn(&'a str) -> O, O: Iterator<Item = char> + 'a>(
    mut input: &'a [u8],
    f: &mut fmt::Formatter<'_>,
    t: Transformer,
) -> fmt::Result {
    use std::fmt::Write;
    // Adapted from <https://doc.rust-lang.org/std/str/struct.Utf8Error.html>
    loop {
        match core::str::from_utf8(input) {
            Ok(valid) => {
                t(valid).try_for_each(|c| f.write_char(c))?;
                break;
            }
            Err(error) => {
                let (valid, after_valid) = input.split_at(error.valid_up_to());

                t(core::str::from_utf8(valid).unwrap()).try_for_each(|c| f.write_char(c))?;
                f.write_char(char::REPLACEMENT_CHARACTER)?;

                if let Some(invalid_sequence_length) = error.error_len() {
                    input = &after_valid[invalid_sequence_length..];
                } else {
                    break;
                }
            }
        }
    }
    Ok(())
}
