/// Error type(s).

use self::TMDescError::*;
use std::io;
use std::fmt::{self, Display, Formatter};

pub enum TMDescError {
    Io(io::Error)
}

impl From<io::Error> for TMDescError {
    fn from(error: io::Error) -> TMDescError {
        Io(error)
    }
}

impl Display for TMDescError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &Io(ref e) => write!(f, "I/O error: {}", e)
        }
    }
}
