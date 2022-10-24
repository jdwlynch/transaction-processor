use decimal::d128;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub enum TxTypes {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback
}

#[derive(Debug, Deserialize)]
pub struct Transaction{
    #[serde(rename = "type")]
    pub tx_type: TxTypes,
    pub client: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    pub amount: Option<d128>,
    #[serde(skip)]
    pub disputed: bool
}