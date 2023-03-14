use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::utils::get_epoch_ts;

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Money {
    #[serde(default)]
    real: u64,
    #[serde(default)]
    bonus: u64,
}

impl Money {
    pub fn new(real: u64, bonus: u64) -> Self {
        Self { real, bonus }
    }
    pub fn real(&self) -> u64 {
        self.real
    }
    pub fn bonus(&self) -> u64 {
        self.bonus
    }

    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Money(real: {}, bonus: {})", self.real, self.bonus)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wallet {
    user_id: u32,
    balance: Money,
    created_ts: Option<u64>,
    updated_ts: Option<u64>,
    created_by: Option<u32>,
    updated_by: Option<u32>,
}
impl Wallet {
    pub fn balance(&self) -> Money {
        self.balance
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WalltetTransactionType {
    AddBalance,
    Withdraw,
    PayForContest,
    ContestWin,
    SignupBonus,
    ReferralBonus,
    RefereeBonus,
}

impl WalltetTransactionType {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WalletTransactionStatus {
    Pending,
    Completed,
    Error,
}

impl WalletTransactionStatus {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletTransaction {
    user_id: u32,
    transaction_type: WalltetTransactionType,
    amount: Money,
    status: WalletTransactionStatus,
    balance_before: Money,
    balance_after: Option<Money>,
    tracking_id: Option<String>,
    remarks: Option<String>,
    receiver_upi_id: Option<String>,
    error_reason: Option<String>,
    created_ts: Option<u64>,
    updated_ts: Option<u64>,
    created_by: Option<u32>,
    updated_by: Option<u32>,
}

impl WalletTransaction {
    pub fn add_bal_init_trans(user_id: u32, amount: Money, balance_before: Money) -> Self {
        let ts = get_epoch_ts();
        Self {
            user_id,
            transaction_type: WalltetTransactionType::AddBalance,
            amount,
            status: WalletTransactionStatus::Pending,
            balance_before,
            balance_after: None,
            tracking_id: None,
            remarks: None,
            receiver_upi_id: None,
            error_reason: None,
            created_ts: Some(ts),
            created_by: Some(user_id),
            updated_ts: None,
            updated_by: None,
        }
    }

    pub fn user_id(&self) -> u32 {
        self.user_id
    }

    pub fn amount(&self) -> Money {
        self.amount
    }

    pub fn balance_before(&self) -> Money {
        self.balance_before
    }
}
