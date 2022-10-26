use crate::error;
use log::trace;
use rust_decimal::prelude::*;
use serde::Serialize;

///Holds all account details for a client, including funds, allocation, id, and status (locked).
/// These functions are designed to be called on only fully validated transactions. All amounts
/// are assumed to be valid.
/// See the readme for rules on how these fields are set and interact.
#[derive(Default, Debug, Serialize)]
pub struct Client {
    pub client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    pub locked: bool,
}

impl Client {
    pub fn new(id: u16) -> Self {
        Self {
            client: id,
            ..Default::default()
        }
    }
    ///Add money to the account
    pub fn deposit(&mut self, amount: Decimal) {
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
    ///Withdraw money from the account if there are sufficient funds
    pub fn withdraw(&mut self, amount: Decimal) -> Result<(), error::Error> {
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
            Err(error::Error::Client(format!(
                "Insufficient funds for client {}. \
                {} requested, {} available",
                self.client, amount, self.available
            )))
        }
    }
    ///Hold disputed funds removing them from the available balance
    pub fn dispute(&mut self, amount: Decimal) {
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
    ///Resolve a dispute, releasing the funds from held to available
    ///# Panics
    /// If less funds are held than are supposed to be resolved, the application must panic.
    /// This means funds are being leaked somewhere and there is a malfunction. This should be
    /// impossible.
    pub fn resolve(&mut self, amount: Decimal) {
        if self.held < amount {
            //This should be impossible. The ledger is malfunctioning, so the system can't be trusted
            panic!(
                "System error on client {}. Trying to resolve but amount: {} \
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
    ///Charge back a dispute, release funds from held and discharging them (subtract from total)
    ///# Panics
    /// If less funds are held than are supposed to be charged back, the application must panic.
    /// This means funds are being leaked somewhere and there is a malfunction. This should be
    /// impossible.
    pub fn chargeback(&mut self, amount: Decimal) {
        if self.held < amount {
            //This should be impossible. The ledger is malfunctioning, so the system can't be trusted
            panic!(
                "System error on client {}. Trying to chargeback but amount: {} \
            is greater than the value of held funds: {}",
                self.client, amount, self.held
            )
        } else {
            self.total -= amount;
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

#[cfg(test)]
mod tests {
    use std::ops::Neg;
    use rust_decimal::Decimal;
    use crate::models::client::Client;

    #[test]
    fn deposit() {
        let mut client = Client::new(1);
        let amount=Decimal::new(10000, 4);
        client.deposit(amount);
        assert_eq!(client.total, amount);
        assert_eq!(client.available, amount);
    }

    #[test]
    fn valid_withdrawal() {
        let mut client = Client::new(1);
        let amount=Decimal::new(10000, 4);
        let zero = Decimal::new(00000, 4);
        let result = match client.withdraw(amount){
            Ok(_) => true,
            Err(_) => false
        };
        assert!(!result);
        assert_eq!(client.total, zero);
        assert_eq!(client.available, zero);
    }

    #[test]
    fn insufficient_funds_withdrawal() {
        let mut client = Client::new(1);
        let amount=Decimal::new(10000, 4);
        let result = match client.withdraw(amount){
            Ok(_) => true,
            Err(_) => false
        };
        assert!(!result);
    }

    #[test]
    fn dispute() {
        let mut client = Client::new(1);
        let amount=Decimal::new(10000, 4);
        client.dispute(amount);
        assert_eq!(client.available, amount.neg());
        assert_eq!(client.held, amount);
    }

    #[test]
    fn resolve() {
        let mut client = Client::new(1);
        let amount=Decimal::new(10000, 4);
        let zero = Decimal::new(00000, 4);
        client.dispute(amount);
        client.resolve(amount);
        assert_eq!(client.available, zero);
        assert_eq!(client.held, zero);
    }

    #[test]
    fn chargeback() {
        let mut client = Client::new(1);
        let amount=Decimal::new(10000, 4);
        let zero = Decimal::new(00000, 4);
        client.dispute(amount);
        client.chargeback(amount);
        assert_eq!(client.available, amount.neg());
        assert_eq!(client.held, zero);
    }

    #[test]
    #[should_panic(expected = "System error on client 1. Trying to resolve but amount: 1.0000 is greater than the value of held funds: 0")]
    fn resolve_not_enough_held() {
        let mut client = Client::new(1);
        let amount=Decimal::new(10000, 4);
        client.resolve(amount);
    }

    #[test]
    #[should_panic(expected = "System error on client 1. Trying to chargeback but amount: 1.0000 is greater than the value of held funds: 0")]
    fn chargeback_not_enough_held() {
        let mut client = Client::new(1);
        let amount=Decimal::new(10000, 4);
        client.chargeback(amount);
    }
}