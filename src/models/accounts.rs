use decimal::d128;
use std::collections::HashMap;
use crate::error;

#[derive(Default, Debug)]
pub struct Client{
    pub id: u16,
    total: d128,
    held: d128,
    available: d128,
    locked: bool,
}

#[derive(Default, Debug)]
pub struct Accounts {
    clients: HashMap<u16, Client>
}

impl Accounts {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new()
        }
    }
    pub fn get_client(&mut self, id: u16) -> Result<&mut Client, error::Error> {
        let test = self.clients.entry(id).or_insert(Client::new());
        if test.locked {
            return Err(error::Error::AccountError(
                format!("Client {} is locked", test.id))
            );
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
    }
    pub fn withdraw(&mut self, amount: d128) -> Result<(), error::Error>{
        if amount <= self.available{
            self.total -= amount;
            self.available -= amount;
            Ok(())
        }else {
            Err(error::Error::AccountError(
                format!("Insufficient funds for client {}. \
                {} requested, {} available", self.id, amount, self.available)
            ))
        }
    }
    pub fn dispute(&mut self, amount: d128) {
        self.available -= amount;
        self.held += amount;
    }
    pub fn resolve(&mut self, amount: d128) {
        if self.held < amount{
            panic!("System error on client {}. Trying to resolve but amount: {}\
            is greater than the value of held funds: {}", self.id, amount, self.held)
        }
        self.available += amount;
        self.held -= amount;
    }
    pub fn chargeback(&mut self, amount: d128){
        if self.held < amount{
            panic!("System error on client {}. Trying to chargeback but amount: {}\
            is greater than the value of held funds: {}", self.id, amount, self.held)
        }else {
            self.available -= amount;
            self.held -= amount;
            self.locked = true;
        }
    }
}