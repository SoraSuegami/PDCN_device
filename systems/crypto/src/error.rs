extern crate failure;
use failure::Fail;
use wasmi::{Error,Trap};
use core::str;
use hex::{FromHexError};



pub struct CryptoError;