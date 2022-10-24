use crate::error;
use decimal::d128;
use log::{debug, trace};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Accounts {
    pub clients: HashMap<u16, Client>,
}

impl Accounts {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }
    pub fn get_client(&mut self, id: u16) -> Result<&mut Client, error::Error> {
        let client = self.clients.entry(id).or_insert(Client::new(id));
        if client.locked {
            return Err(error::Error::AccountError(format!(
                "Client {} is locked",
                client.client
            )));
        } else {
            debug!("[!] Client {} returned from get_client", client.client);
            return Ok(client);
        }
    }
}

#[derive(Default, Debug, Serialize)]
pub struct Client {
    pub client: u16,
    available: d128,
    held: d128,
    total: d128,
    locked: bool,
}

impl Client {
    pub fn new(id: u16) -> Self {
        Self {
            client: id,
            ..Default::default()
        }
    }
    pub fn deposit(&mut self, amount: d128) {
        self.total += amount;
        self.available += amount;
        trace!(
            "[!] Client {} deposited ${} and has total = ${} and available = ${}.",
            self.client,
            amount,
            self.total,
            self.available
        );
    }
    pub fn withdraw(&mut self, amount: d128) -> Result<(), error::Error> {
        if amount <= self.available {
            self.total -= amount;
            self.available -= amount;
            trace!(
                "[!] Client {} withdrew ${} and has total = ${} and available = ${}.",
                self.client,
                amount,
                self.total,
                self.available
            );
            Ok(())
        } else {
            Err(error::Error::AccountError(format!(
                "Insufficient funds for client {}. \
                {} requested, {} available",
                self.client, amount, self.available
            )))
        }
    }
    pub fn dispute(&mut self, amount: d128) {
        self.available -= amount;
        self.held += amount;
        trace!(
            "[!] Client {} disputed ${} and has available = ${} and held = ${}.",
            self.client,
            amount,
            self.available,
            self.held
        );
    }
    pub fn resolve(&mut self, amount: d128) {
        if self.held < amount {
            //This should be impossible. The ledger is malfunctioning, so the system can't be trusted
            panic!(
                "System error on client {}. Trying to resolve but amount: {}\
            is greater than the value of held funds: {}",
                self.client, amount, self.held
            )
        }
        self.available += amount;
        self.held -= amount;
        trace!(
            "[!] Client {} resolved ${} and has available = ${} and held = ${}.",
            self.client,
            amount,
            self.available,
            self.held
        );
    }
    pub fn chargeback(&mut self, amount: d128) {
        if self.held < amount {
            //This should be impossible. The ledger is malfunctioning, so the system can't be trusted
            panic!(
                "System error on client {}. Trying to chargeback but amount: {}\
            is greater than the value of held funds: {}",
                self.client, amount, self.held
            )
        } else {
            self.available -= amount;
            self.held -= amount;
            self.locked = true;
        }
        trace!(
            "[!] Client {} charged back ${} and has available = ${} and held = ${}.",
            self.client,
            amount,
            self.available,
            self.held
        );
    }
}