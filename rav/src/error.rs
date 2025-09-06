use std::fmt;
use std::io;
use std::result;

/// `Error` provides an enumeration of all possible errors reported by Symphonia.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Error {
    /// An IO error occured while reading, writing, or seeking the stream.
//    IoError(std::io::Error),
    /// The stream contained malformed data and could not be decoded or demuxed.
    DecodeError(&'static str),
    /// An unsupported container or codec feature was encounted.
    Unsupported(&'static str),
    /// A default or user-defined limit was reached while decoding or demuxing the stream. Limits
    /// are used to prevent denial-of-service attacks from malicious streams.
    LimitError(&'static str),
    /// The demuxer or decoder needs to be reset before continuing.
    ResetRequired,
    /// The demuxer cannot get more data from input for now, retry again later.
    RetryLater,
    /// Invalid input parameters.
    InvalidInput,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
//            Error::IoError(ref err) => err.fmt(f),
            Error::DecodeError(msg) => {
                write!(f, "malformed stream: {}", msg)
            }
            Error::Unsupported(feature) => {
                write!(f, "unsupported feature: {}", feature)
            }
            Error::LimitError(constraint) => {
                write!(f, "limit reached: {}", constraint)
            }
            Error::ResetRequired => {
                write!(f, "decoder needs to be reset")
            }
            Error::RetryLater => {
                write!(f, "no data, retry later")
            }
            Error::InvalidInput => {
                write!(f, "invalid input parameters")
            }
        }
    }
}

// impl From<io::Error> for Error {
//     fn from(err: io::Error) -> Error {
//         Error::IoError(err)
//     }
// }

pub type Result<T> = result::Result<T, Error>;

/// Convenience function to create a decode error.
pub fn decode_error<T>(desc: &'static str) -> Result<T> {
    Err(Error::DecodeError(desc))
}

/// Convenience function to create an unsupport feature error.
pub fn unsupported_error<T>(feature: &'static str) -> Result<T> {
    Err(Error::Unsupported(feature))
}

/// Convenience function to create a limit error.
pub fn limit_error<T>(constraint: &'static str) -> Result<T> {
    Err(Error::LimitError(constraint))
}

/// Convenience function to create a reset required error.
pub fn reset_error<T>() -> Result<T> {
    Err(Error::ResetRequired)
}

/// Convenience function to create a invalid parameters error.
pub fn invalid_input_error<T>() -> Result<T> {
    Err(Error::InvalidInput)
}

/// Convenience function to create a retry later error.
pub fn retry_later_error<T>() -> Result<T> {
    Err(Error::RetryLater)
}

