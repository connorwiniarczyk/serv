/// Custom Error Type

use std::error;
use std::fmt;


pub enum Error {
    CustomError,
}

pub struct CustomError;


impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "test")
    }

}
impl fmt::Debug for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "test")
    }
}

impl error::Error for CustomError {}
