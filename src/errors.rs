// errors.rs - Multi-spool errors.
// Copyright (C) 2019  David Anthony Stainton.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::fmt;
use std::error::Error;
use std::io::Error as IoError;
use sled::Error as SledError;
use ed25519_dalek::SignatureError;


#[derive(Debug)]
pub enum SpoolError {
    CreateSpoolCacheFailed,
    SledError(SledError<()>),
    IoError(IoError),
    NoSuchMessage,
    CorruptSpool,
}

impl fmt::Display for SpoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SpoolError::*;
        match self {
            CreateSpoolCacheFailed => write!(f, "Failed to spool set create cache."),
            SledError(x) => x.fmt(f),
            IoError(x) => x.fmt(f),
            NoSuchMessage => write!(f, "No such message."),
            CorruptSpool => write!(f, "Corrupt spool."),
        }
    }
}

impl Error for SpoolError {
    fn description(&self) -> &str {
        "I'm a SpoolError."
    }

    fn cause(&self) -> Option<&Error> {
        use self::SpoolError::*;
        match self {
            CreateSpoolCacheFailed => None,
            SledError(x) => x.source(),
            IoError(x) => x.source(),
            NoSuchMessage => None,
            CorruptSpool => None,
        }
    }
}

impl From<SledError<()>> for SpoolError {
    fn from(error: SledError<()>) -> Self {
        SpoolError::SledError(error)
    }
}

impl From<IoError> for SpoolError {
    fn from(error: IoError) -> Self {
        SpoolError::IoError(error)
    }
}

#[derive(Debug)]
pub enum SpoolSetError {
    CreateSpoolSetCacheFailed,
    SledError(SledError<()>),
    NoSuchSpoolId,
    SignatureError(SignatureError),
}

impl fmt::Display for SpoolSetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SpoolSetError::*;
        match self {
            CreateSpoolSetCacheFailed => write!(f, "Failed to spool set create cache."),
            SledError(x) => x.fmt(f),
            NoSuchSpoolId => write!(f, "Failed to find spool identity."),
            SignatureError(x) => x.fmt(f),
        }
    }
}

impl Error for SpoolSetError {
    fn description(&self) -> &str {
        "I'm a SpoolSetError."
    }

    fn cause(&self) -> Option<&Error> {
        use self::SpoolSetError::*;
        match self {
            CreateSpoolSetCacheFailed => None,
            SledError(x) => x.source(),
            NoSuchSpoolId => None,
            SignatureError(_x) => None, // XXX no cause or source method available
        }
    }
}

impl From<SledError<()>> for SpoolSetError {
    fn from(error: SledError<()>) -> Self {
        SpoolSetError::SledError(error)
    }
}

impl From<SignatureError> for SpoolSetError {
    fn from(error: SignatureError) -> Self {
        SpoolSetError::SignatureError(error)
    }
}

#[derive(Debug)]
pub enum MultiSpoolError {
    SpoolSetError(SpoolSetError),
    SpoolError(SpoolError),
    SledError(SledError<()>),
    NoSuchSpool,
    SignatureError(SignatureError),
    IoError(IoError),
}

impl fmt::Display for MultiSpoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MultiSpoolError::*;
        match self {
            SpoolSetError(x) => x.fmt(f),
            SpoolError(x) => x.fmt(f),
            SledError(x) => x.fmt(f),
            NoSuchSpool => write!(f, "Error, no such spool."),
            SignatureError(x) => x.fmt(f),
            IoError(x) => x.fmt(f),
        }
    }
}

impl Error for MultiSpoolError {
    fn description(&self) -> &str {
        "I'm a MultiSpoolError."
    }

    fn cause(&self) -> Option<&Error> {
        use self::MultiSpoolError::*;
        match self {
            SpoolSetError(x) => x.source(),
            SpoolError(x) => x.source(),
            SledError(x) => x.source(),
            NoSuchSpool => None,
            SignatureError(_x) => None, // XXX no cause or source method available
            IoError(x) => x.source(),
        }
    }
}

impl From<SpoolSetError> for MultiSpoolError {
    fn from(error: SpoolSetError) -> Self {
        MultiSpoolError::SpoolSetError(error)
    }
}

impl From<SpoolError> for MultiSpoolError {
    fn from(error: SpoolError) -> Self {
        MultiSpoolError::SpoolError(error)
    }
}

impl From<SledError<()>> for MultiSpoolError {
    fn from(error: SledError<()>) -> Self {
        MultiSpoolError::SledError(error)
    }
}

impl From<SignatureError> for MultiSpoolError {
    fn from(error: SignatureError) -> Self {
        MultiSpoolError::SignatureError(error)
    }
}

impl From<IoError> for MultiSpoolError {
    fn from(error: IoError) -> Self {
        MultiSpoolError::IoError(error)
    }
}
