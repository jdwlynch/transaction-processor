use decimal::d128;
use std::collections::HashMap;
use crate::models::engine::Transaction;

#[derive(Default)]
pub struct Client{
    pub id: u16,
    total: d128,
    held: d128,
    available: d128,
    locked: bool,
}

pub struct Manager{
    clients: HashMap<u16, Client>
}

impl Manager {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new()
        }
    }
    pub fn get_client(&mut self, id: u16) -> Result<&mut Client, String> {
        let test = self.clients.entry(id).or_insert(Client::new());
        if test.locked {
            return Err(String::from("Client is locked"));
        } else {
            return Ok(test)
        }
    }
}
impl Client {
    pub fn new(/*id: u32*/) -> Self {
        Self {
            //id,
            ..Default::default()
        }
    }
    pub fn deposit(&mut self, amount: d128){
        self.total += amount;
        self.available += amount;
        println!("Deposited money. Balance = {}", self.available)
    }
    pub fn withdraw(&mut self, amount: d128){
        if amount <= self.available{
            self.total -= amount;
            self.available -= amount;
            println!("Withdrew money. Balance = {}", self.available);
        }else {
            println!("Insufficient Funds.\nRequested: {}, Available: {}", amount, self.available);
        }
    }
    pub fn dispute(&mut self, amount: d128){
        self.available -= amount;
        self.held += amount;
    }
    pub fn resolve(&mut self, amount: d128){
        self.available += amount;
        self.held -= amount;
    }
    pub fn chargeback(&mut self, amount: d128){
        self.available -= amount;
        self.held -= amount;
        self.locked = true;
    }
}