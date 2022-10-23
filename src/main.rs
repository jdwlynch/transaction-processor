
mod models;
mod consumer;
mod producer;
//mod processor;
//mod manager;

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;
use csv::DeserializeRecordsIntoIter;
use crate::consumer::Consumer;
use crate::models::engine::Transaction;
use crate::models::engine::Engine;
use crate::models::manager::{Client, Manager};

fn example() -> Result<(), Box<dyn Error>> {
    let mut test = Consumer::new()?;
    for tx in test{
        let record: Transaction = tx?;
        println!("Result was: {:?}", record);
    }
    Ok(())
}

fn main() {
/*    if let Err(err) = example() {
        println!("error running example: {}", err);
        process::exit(1);
    }*/
    let cons = Consumer::new().unwrap();
    let clients = Manager::new();
    Engine::process_transactions(cons, clients);
}