use std::{
    error, fmt,
    io::{self, Read},
};

use crate::{
    binning_index::index::header::{Format, format},
    io::reader::num::read_i32_le,
};

pub fn read_format<R>(reader: &mut R) -> Result<Format, ReadError>
where
    R: Read,
{
    read_i32_le(reader)
        .map_err(ReadError::Io)
        .and_then(|n| Format::try_from(n).map_err(ReadError::Invalid))
}

/// An error returned when a CSI header format fails to be read.
#[derive(Debug)]
pub enum ReadError {
    /// An I/O error.
    Io(io::Error),
    /// The input is invalid.
    Invalid(format::TryFromIntError),
}

impl error::Error for ReadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ReadError::Io(e) => Some(e),
            ReadError::Invalid(e) => Some(e),
        }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => write!(f, "I/O error"),
            Self::Invalid(_) => write!(f, "invalid input"),
        }
    }
}
