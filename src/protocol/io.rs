//! Traits for reading/writing Minecraft-encoded values.

use crate::{entities::metadata::{MetaEntry, EntityMetadata}, network::metadata::Metadata};

use super::ProtocolVersion;
use anyhow::{anyhow, bail, Context};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use encoding::{all::UTF_16BE, Encoding, EncoderTrap};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    io::{self, Cursor, Read, Write},
    iter::{self, FromIterator},
    marker::PhantomData,
    num::TryFromIntError, fmt::Display,
};
use thiserror::Error;
/// Trait implemented for types which can be read
/// from a buffer.
pub trait Readable {
    /// Reads this type from the given buffer.
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized;
}

/// Trait implemented for types which can be written
/// to a buffer.
pub trait Writeable: Sized {
    /// Writes this value to the given buffer.
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()>;
}

impl<'a, T> Writeable for &'a T
where
    T: Writeable,
{
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        T::write(*self, buffer, version)?;
        Ok(())
    }
}

/// Error when reading a value.
#[derive(Debug, Error)]
pub enum Error {
    #[error("unexpected end of input: failed to read value of type `{0}`")]
    UnexpectedEof(&'static str),
}

macro_rules! integer_impl {
    ($($int:ty, $read_fn:tt, $write_fn:tt),* $(,)?) => {
        $(
            impl Readable for $int {
                fn read(buffer: &mut Cursor<&[u8]>, _version: ProtocolVersion) -> anyhow::Result<Self> {
                    buffer.$read_fn::<BigEndian>().map_err(anyhow::Error::from)
                }
            }

            impl Writeable for $int {
                fn write(&self, buffer: &mut Vec<u8>, _version: ProtocolVersion) -> anyhow::Result<()> {
                    buffer.$write_fn::<BigEndian>(*self)?;
                    Ok(())
                }
            }
        )*
    }
}

integer_impl! {
    u16, read_u16, write_u16,
    u32, read_u32, write_u32,
    u64, read_u64, write_u64,

    i16, read_i16, write_i16,
    i32, read_i32, write_i32,
    i64, read_i64, write_i64,

    f32, read_f32, write_f32,
    f64, read_f64, write_f64,
}

impl Readable for u8 {
    fn read(buffer: &mut Cursor<&[u8]>, _version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        buffer.read_u8().map_err(anyhow::Error::from)
    }
}

impl Writeable for u8 {
    fn write(&self, buffer: &mut Vec<u8>, _version: ProtocolVersion) -> anyhow::Result<()> {
        buffer.write_u8(*self)?;
        Ok(())
    }
}
impl Readable for i8 {
    fn read(buffer: &mut Cursor<&[u8]>, _version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        buffer.read_i8().map_err(anyhow::Error::from)
    }
}

impl Writeable for i8 {
    fn write(&self, buffer: &mut Vec<u8>, _version: ProtocolVersion) -> anyhow::Result<()> {
        buffer.write_i8(*self)?;
        Ok(())
    }
}

impl<T> Readable for Option<T>
where
    T: Readable,
{
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        // Assume boolean prefix.
        let present = bool::read(buffer, version)?;

        if present {
            Ok(Some(T::read(buffer, version)?))
        } else {
            Ok(None)
        }
    }
}

impl<T> Writeable for Option<T>
where
    T: Writeable,
{
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        let present = self.is_some();
        present.write(buffer, version)?;

        if let Some(value) = self {
            value.write(buffer, version)?;
        }

        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct PingData {
    motd: String,
    online_players: usize,
    slots: usize,
}
impl PingData {
    pub fn new(motd: impl Into<String>, online_players: usize, slots: usize) -> Self {
        Self { motd: motd.into(), online_players, slots }
    }
}
impl Readable for PingData {
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized {
        unreachable!()
    }
}
impl Writeable for PingData {
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        let format = format!("{}\u{00a7}{}\u{00a7}{}", self.motd, self.online_players, self.slots);
        String16(format).write(buffer, version)?;
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct AbsoluteInt(pub f64);
impl Readable for AbsoluteInt {
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized {
        let val = i32::read(buffer, version)?;
        Ok(Self((val as f64) / 32.0))
    }
}
impl Writeable for AbsoluteInt {
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        let val = (self.0 * 32.0) as i32;
        val.write(buffer, version)
    }
}
impl Into<AbsoluteInt> for f64 {
    fn into(self) -> AbsoluteInt {
        AbsoluteInt(self)
    }
}
#[derive(Clone, Debug)]
pub struct String16(pub String);
impl Display for String16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Into<String16> for String {
    fn into(self) -> String16 {
        String16(self)
    }
}
impl Into<String16> for &str {
    fn into(self) -> String16 {
        String16(self.to_string())
    }
}
impl Writeable for String16 {
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        (self.0.chars().count() as i16).write(buffer, version)?;
        let vec: Vec<u16> = self.0.encode_utf16().collect();
        for short in vec {
            short.write(buffer, version)?;
        }
        Ok(())
    }
}
impl Readable for String16 {
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let length = i16::read(buffer, version).context("failed to read string length")? as usize * 2;

        // Read string into buffer.
        let mut temp = vec![0u8; length];
        buffer
            .read_exact(&mut temp)
            .map_err(|_| Error::UnexpectedEof("String"))?;
        let temp: Vec<u16> = temp
            .chunks_exact(2)
            .into_iter()
            .map(|a| u16::from_be_bytes([a[0], a[1]]))
            .collect();
        let temp = temp.as_slice();

        let s = String::from_utf16(temp).context("string contained invalid UTF8")?;

        Ok(String16(s))
    }
}
impl Readable for String {
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        // Length is encoded as a short.
        // Following `length` bytes are the UTF8-encoded
        // string.

        let length = i16::read(buffer, version).context("failed to read string length")? as usize;

        // Read string into buffer.
        let mut temp = vec![0u8; length];
        buffer
            .read_exact(&mut temp)
            .map_err(|_| Error::UnexpectedEof("String"))?;
        let s = std::str::from_utf8(&temp).context("string contained invalid UTF8")?;

        Ok(s.to_owned())
    }
}

impl Writeable for String {
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        (self.len() as i16).write(buffer, version)?;
        buffer.extend_from_slice(self.as_bytes());

        Ok(())
    }
}

impl Readable for bool {
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let x = u8::read(buffer, version)?;

        if x == 0 {
            Ok(false)
        } else if x == 1 {
            Ok(true)
        } else {
            Err(anyhow::anyhow!("invalid boolean tag {}", x))
        }
    }
}

impl Writeable for bool {
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        let x = if *self { 1u8 } else { 0 };
        x.write(buffer, version)?;

        Ok(())
    }
}

pub const MAX_LENGTH: usize = 1024 * 1024; // 2^20 elements

/// Reads and writes an array of inner `Writeable`s.
/// The array is prefixed with a `VarInt` length.
///
/// This will reject arrays of lengths larger than MAX_LENGTH.
pub struct LengthPrefixedVec<'a, P, T>(pub Cow<'a, [T]>, PhantomData<P>)
where
    [T]: ToOwned<Owned = Vec<T>>;

impl<'a, P, T> Readable for LengthPrefixedVec<'a, P, T>
where
    T: Readable,
    [T]: ToOwned<Owned = Vec<T>>,
    P: TryInto<usize> + Readable,
    P::Error: std::error::Error + Send + Sync + 'static,
{
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let length: usize = P::read(buffer, version)?.try_into()?;

        if length > MAX_LENGTH {
            bail!("array length too large ({} > {})", length, MAX_LENGTH);
        }

        let vec = iter::repeat_with(|| T::read(buffer, version))
            .take(length)
            .collect::<anyhow::Result<Vec<T>>>()?;
        Ok(Self(Cow::Owned(vec), PhantomData))
    }
}

impl<'a, P, T> Writeable for LengthPrefixedVec<'a, P, T>
where
    T: Writeable,
    [T]: ToOwned<Owned = Vec<T>>,
    P: TryFrom<usize> + Writeable,
    P::Error: std::error::Error + Send + Sync + 'static,
{
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        P::try_from(self.0.len())?.write(buffer, version)?;
        self.0
            .iter()
            .for_each(|item| item.write(buffer, version).expect("failed to write to vec"));

        Ok(())
    }
}

impl<'a, P, T> From<LengthPrefixedVec<'a, P, T>> for Vec<T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(x: LengthPrefixedVec<'a, P, T>) -> Self {
        x.0.into_owned()
    }
}

impl<'a, P, T> From<&'a [T]> for LengthPrefixedVec<'a, P, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(slice: &'a [T]) -> Self {
        Self(Cow::Borrowed(slice), PhantomData)
    }
}

impl<'a, P, T> From<Vec<T>> for LengthPrefixedVec<'a, P, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(vec: Vec<T>) -> Self {
        Self(Cow::Owned(vec), PhantomData)
    }
}

pub type ShortPrefixedVec<'a, T> = LengthPrefixedVec<'a, u16, T>;

/// A vector of bytes which consumes all remaining bytes in this packet.
/// This is used by the plugin messaging packets, for one.
pub struct LengthInferredVecU8<'a>(pub Cow<'a, [u8]>);

impl<'a> Readable for LengthInferredVecU8<'a> {
    fn read(buffer: &mut Cursor<&[u8]>, _version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut vec = Vec::new();
        buffer.read_to_end(&mut vec)?;
        Ok(LengthInferredVecU8(Cow::Owned(vec)))
    }
}

impl<'a> Writeable for LengthInferredVecU8<'a> {
    fn write(&self, buffer: &mut Vec<u8>, _version: ProtocolVersion) -> anyhow::Result<()> {
        buffer.extend_from_slice(&*self.0);
        Ok(())
    }
}

impl<'a> From<&'a [u8]> for LengthInferredVecU8<'a> {
    fn from(slice: &'a [u8]) -> Self {
        LengthInferredVecU8(Cow::Borrowed(slice))
    }
}

impl<'a> From<LengthInferredVecU8<'a>> for Vec<u8> {
    fn from(x: LengthInferredVecU8<'a>) -> Self {
        x.0.into_owned()
    }
}
impl Writeable for Metadata {
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        buffer.append(&mut self.finish());
        Ok(())
    }
}
impl Readable for Metadata {
    fn read(buffer: &mut Cursor<&[u8]>, version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized {
        unimplemented!();
    }
}
