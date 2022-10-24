use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use csv::{DeserializeRecordsIntoIter, DeserializeRecordsIter};
use serde::de::DeserializeOwned;
use serde::ser::StdError;
use crate::models::engine::Transaction;
use crate::error;

/// Returns the first positional argument sent to this process. If there are no
/// positional arguments, then this returns an error.
fn get_first_arg() -> Result<OsString, error::Error> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

pub struct Consumer{
    iter : DeserializeRecordsIntoIter<File, Transaction>
}

impl Consumer{
    pub fn new() -> Result<Consumer, error::Error> {
        let file_path = get_first_arg()?;
        let mut rdr = csv::Reader::from_path(file_path)?;
        Ok(Self{
            iter: rdr.into_deserialize()
        })
    }
    fn get_transaction(&mut self) -> Option<Result<Transaction, csv::Error>>{
        self.iter.next()
    }
}

impl Iterator for Consumer {
    type Item = Result<Transaction, csv::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
