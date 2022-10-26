use crate::error;
use log::{debug, trace};
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::Deserialize;

/// Types of transactions. See the README for details.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TxTypes {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

///Associated functions that validate and format transactions according to the rules of the
/// transaction-engine. Provides a dto to serialize transactions into.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TxTypes,
    /// Unique client identification number
    pub client: u16,
    #[serde(rename = "tx")]
    /// Unique transaction number, or ID of transaction related to a dispute
    pub tx_id: u32,
    /// Amounts are not present with dispute related transactions
    pub amount: Option<Decimal>,
    /// Disputed is set by the transaction-engine, so it is defaulted when serializing
    #[serde(skip)]
    pub disputed: bool,
}

impl Transaction {
    /// Checks that the transaction is valid and trims amount to 4 decimal places using
    /// rust_decimal .round_dp(). Amounts must be positive and only presenton deposits
    /// or withdrawals.
    pub fn validate_transaction(amount: &mut Option<Decimal>, tx_type: &TxTypes) -> Result<(), error::Error> {
        match tx_type {
            TxTypes::Deposit | TxTypes::Withdrawal => {
                trace!("Deposit or withdrawal detected, calling validate: {:?}", tx_type);
                Self::validate_deposit_withdrawal_structure(amount)
            }
            TxTypes::Dispute | TxTypes::Resolve | TxTypes::Chargeback => {
                trace!("Dispute related transaction detected, calling validate: {:?}", tx_type);
                Self::validate_dispute_related_structure(amount)
            }
        }
    }

    fn validate_deposit_withdrawal_structure(amount: &mut Option<Decimal>) -> Result<(), error::Error> {
        if let Some(tx_amount) = amount {
            if tx_amount < &mut dec!(0) {
                Err(error::Error::Transaction(String::from("Amount must be a positive number")))
            } else {
                *amount = Some(tx_amount.round_dp(4));
                trace!("withdrawal or deposit of {:?} successfully validated.", amount);
                Ok(())
            }
        } else {
            Err(error::Error::Transaction(String::from("Deposits and withdrawals require an amount.")))
        }
    }

    fn validate_dispute_related_structure(amount: &mut Option<Decimal>) -> Result<(), error::Error> {
        if amount.is_some() {
            Err(error::Error::Transaction(String::from("Disputes, resolutions and chargebacks shouldn't have amounts")))
        } else {
            debug!("Dispute related transaction ok.");
            Ok(())
        }
    }

    ///Called once a transactions format is confirmed to be valid. Takes the disputed status of the
    /// transaction related to the dispute, and whether the current transaction is resolving that
    /// transaction or not. If resolving is true, the transaction must be disputed. We can't
    /// resolve an undisputed transaction. Likewise, if resolving is false, the transaction can't
    /// must be undisputed. We can't dispute a transaction already being disputed.
    pub fn check_transaction_dispute_valid(tx_resolving: bool, ledger_disputed: bool) -> Result<(), error::Error> {
        if tx_resolving != ledger_disputed {
            Err(error::Error::Transaction(String::from("Cannot dispute a disputed transaction, \
                        or resolve an undisputed transaction.")))
        } else {
            trace!("Dispute related transaction ok. The targeted transaction disputed = {} \
            and the new transaction has resolving = {}", ledger_disputed, tx_resolving);
            Ok(())
        }
    }
    ///Only deposits with valid amounts may be disputed.
    ///# Panics
    /// If the ledger has a deposit that doesn't have a value, the system must panic. The system
    /// is not keeping track of transactions, and data is being lost, which is a serious error.
    pub fn check_transaction_is_disputable(amount: Option<Decimal>, tx_type: &TxTypes) -> Result<(), error::Error> {
        match tx_type {
            TxTypes::Deposit => {
                amount.expect("System error, ledger shows a deposit with no amount");
                debug!("A deposit is being disputed or resolved for a value of {:?}", amount);
                Ok(())

            }
            _ => Err(error::Error::Transaction(format!("Trying to dispute or resolve a {:?}. Only Deposits are valid targets.", tx_type))),
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use crate::transaction::{Transaction, TxTypes};

    #[test]
    fn deposit_is_disputable() {
        let amount: Option<Decimal> = Some(Decimal::new(10000, 4));
        let result = Transaction::check_transaction_is_disputable( amount, &TxTypes::Deposit).is_ok();
        assert!(result);
    }

    #[test]
    fn withdraw_not_disputable() {
        let amount: Option<Decimal> = Some(Decimal::new(10000, 4));
        let result = Transaction::check_transaction_is_disputable( amount, &TxTypes::Withdrawal).is_ok();
        assert!(!result);
    }

    #[test]
    fn dispute_not_disputable() {
        let amount: Option<Decimal> = Some(Decimal::new(10000, 4));
        let result = Transaction::check_transaction_is_disputable( amount, &TxTypes::Dispute).is_ok();
        assert!(!result);
    }

    #[test]
    fn resolve_not_disputable() {
        let amount: Option<Decimal> = Some(Decimal::new(10000, 4));
        let result = Transaction::check_transaction_is_disputable( amount, &TxTypes::Resolve).is_ok();
        assert!(!result);
    }

    #[test]
    fn chargeback_not_disputable() {
        let amount: Option<Decimal> = Some(Decimal::new(10000, 4));
        let result = Transaction::check_transaction_is_disputable( amount, &TxTypes::Chargeback).is_ok();
        assert!(!result);
    }

    #[test]
    #[should_panic(expected = "System error, ledger shows a deposit with no amount")]
    fn valid_dispute_with_no_amount() {
        let amount: Option<Decimal> = None;
        let _result = Transaction::check_transaction_is_disputable(amount, &TxTypes::Deposit);
    }

    #[test]
    fn resolving_disputed_transaction() {
        let result = Transaction::check_transaction_dispute_valid(true, true).is_ok();
        assert!(result);
    }

    #[test]
    fn disputing_undisputed_transaction() {
        let result = Transaction::check_transaction_dispute_valid(true, true).is_ok();
        assert!(result);
    }

    #[test]
    fn resolving_undisputed_transaction() {
        let result = Transaction::check_transaction_dispute_valid(true, false).is_ok();
        assert!(!result);
    }

    #[test]
    fn disputing_disputed_transaction() {
        let result =  Transaction::check_transaction_dispute_valid(false, true).is_ok();
        assert!(!result);
    }

    #[test]
    fn valid_deposit() {
        let mut amount: Option<Decimal> = Some(Decimal::new(10000, 4));
        let result = Transaction::validate_transaction(& mut amount, &TxTypes::Deposit).is_ok();
        assert!(result);
    }

    #[test]
    fn valid_withdrawal() {
        let mut amount: Option<Decimal> = Some(Decimal::new(10000, 4));
        let result = Transaction::validate_transaction(& mut amount, &TxTypes::Withdrawal).is_ok();
        assert!(result);
    }

    #[test]
    fn negative_deposit() {
        let mut amount: Option<Decimal> = Some(Decimal::new(-10000, 4));
        let result = Transaction::validate_transaction(& mut amount, &TxTypes::Deposit).is_ok();
        assert!(!result);
    }

    #[test]
    fn negative_withdrawal() {
        let mut amount: Option<Decimal> = Some(Decimal::new(-10000, 4));
        let result = Transaction::validate_transaction(& mut amount, &TxTypes::Withdrawal).is_ok();
        assert!(!result);
    }

    #[test]
    fn missing_amount_deposit(){
        let mut amount: Option<Decimal> = None;
        let result = Transaction::validate_transaction(& mut amount, &TxTypes::Deposit).is_ok();
        assert!(!result);
    }

    #[test]
    fn missing_amount_withdrawal(){
        let mut amount: Option<Decimal> = None;
        let result = Transaction::validate_transaction(& mut amount, &TxTypes::Withdrawal).is_ok();
        assert!(!result);
    }

    #[test]
    fn valid_dispute() {
        let mut amount: Option<Decimal> = None;
        let result = Transaction::validate_transaction(& mut amount, &TxTypes::Dispute).is_ok();
        assert!(result);
    }

    #[test]
    fn valid_resolve() {
        let mut amount: Option<Decimal> = None;
        let result = Transaction::validate_transaction(& mut amount, &TxTypes::Resolve).is_ok();
        assert!(result);
    }

    #[test]
    fn valid_chargeback() {
        let mut amount: Option<Decimal> = None;
        let result = Transaction::validate_transaction(& mut amount, &TxTypes::Chargeback).is_ok();
        assert!(result);
    }

    #[test]
    fn amount_with_dispute() {
        let mut amount: Option<Decimal> = Some(Decimal::new(-10000, 4));
        let result = Transaction::validate_transaction(& mut amount, &TxTypes::Dispute).is_ok();
        assert!(!result);
    }

    #[test]
    fn amount_with_resolve() {
        let mut amount: Option<Decimal> = Some(Decimal::new(-10000, 4));
        let result = Transaction::validate_transaction(& mut amount, &TxTypes::Resolve).is_ok();
        assert!(!result);
    }

    #[test]
    fn amount_with_chargeback() {
        let mut amount: Option<Decimal> = Some(Decimal::new(-10000, 4));
        let result = Transaction::validate_transaction(& mut amount, &TxTypes::Chargeback).is_ok();
        assert!(!result);
    }
}