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

#[derive(Debug)]
pub enum SpoolSetError {
    CreateSpoolSetCacheFailed,
}

impl fmt::Display for SpoolSetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SpoolSetError::*;
        match self {
            CreateSpoolSetCacheFailed => write!(f, "Failed to spool set create cache."),
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
        }
    }
}

#[derive(Debug)]
pub enum SpoolError {
    CreateSpoolSetCacheFailed,
    IoError(IoError),
}

impl fmt::Display for SpoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SpoolError::*;
        match self {
            CreateSpoolSetCacheFailed => write!(f, "Failed to spool set create cache."),
            IoError(x) => x.fmt(f),
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
            CreateSpoolSetCacheFailed => None,
            IoError(x) => x.source(),
        }
    }
}

impl From<IoError> for SpoolError {
    fn from(error: IoError) -> Self {
        SpoolError::IoError(error)
    }
}
