use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::{constants::*, utils::get_epoch_ts};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Money {
    #[serde(default)]
    real: u64,
    #[serde(default)]
    bonus: u64,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    withdrawable: Option<u64>,
}

impl Money {
    pub fn new(real: u64, bonus: u64) -> Self {
        Self {
            real,
            bonus,
            withdrawable: None,
        }
    }
    pub fn real(&self) -> u64 {
        self.real
    }
    pub fn bonus(&self) -> u64 {
        self.bonus
    }
    pub fn withdrawable(&self) -> u64 {
        self.withdrawable.unwrap_or_default()
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

impl std::ops::Add for Money {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            real: self.real() + other.real(),
            bonus: self.bonus() + other.bonus(),
            withdrawable: None,
        }
    }
}

impl std::ops::Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        if self.real() < rhs.real() || self.bonus() < rhs.bonus() {
            Default::default()
        } else {
            Self {
                real: self.real() - rhs.real(),
                bonus: self.bonus() - rhs.bonus(),
                withdrawable: None,
            }
        }
    }
}

impl std::cmp::PartialEq for Money {
    fn eq(&self, other: &Self) -> bool {
        self.real() == other.real() && self.bonus() == other.bonus()
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
    ReferrerBonus,
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

    pub fn withdraw_bal_init_trans(
        user_id: u32,
        amount: Money,
        balance_before: Money,
        receiver_upi_id: &str,
    ) -> Self {
        let ts = get_epoch_ts();
        Self {
            user_id,
            transaction_type: WalltetTransactionType::Withdraw,
            amount,
            status: WalletTransactionStatus::Pending,
            balance_before,
            balance_after: None,
            tracking_id: None,
            remarks: None,
            receiver_upi_id: Some(receiver_upi_id.to_string()),
            error_reason: None,
            created_ts: Some(ts),
            created_by: Some(user_id),
            updated_ts: None,
            updated_by: None,
        }
    }

    pub fn pay_for_contest_trans(
        user_id: u32,
        contest_id: &str,
        real: u64,
        bonus: u64,
        balance_before: Money,
        balance_after: Money,
    ) -> Self {
        let ts = get_epoch_ts();
        let amount = Money::new(real, bonus);
        Self {
            user_id,
            transaction_type: WalltetTransactionType::PayForContest,
            amount,
            status: WalletTransactionStatus::Completed,
            balance_before,
            balance_after: Some(balance_after),
            tracking_id: None,
            receiver_upi_id: None,
            remarks: Some(format!("Pay for contest: {}", contest_id)),
            error_reason: None,
            created_ts: Some(ts),
            created_by: Some(user_id),
            updated_ts: None,
            updated_by: None,
        }
    }

    pub fn contest_win_trans(
        user_id: u32,
        amount: Money,
        balance_before: Money,
        balance_after: Money,
        remarks: &str,
    ) -> Self {
        let ts = get_epoch_ts();
        Self {
            user_id,
            transaction_type: WalltetTransactionType::ContestWin,
            amount,
            status: WalletTransactionStatus::Completed,
            balance_before,
            balance_after: Some(balance_after),
            tracking_id: None,
            receiver_upi_id: None,
            remarks: Some(remarks.into()),
            error_reason: None,
            created_ts: Some(ts),
            created_by: Some(user_id),
            updated_ts: None,
            updated_by: None,
        }
    }

    pub fn referral_bonus_trans(
        user_id: u32,
        bonus: u64,
        balance_before: Money,
        balance_after: Money,
    ) -> Self {
        let ts = get_epoch_ts();
        Self {
            user_id,
            transaction_type: WalltetTransactionType::ReferralBonus,
            amount: Money::new(0, bonus),
            status: WalletTransactionStatus::Completed,
            balance_before,
            balance_after: Some(balance_after),
            tracking_id: None,
            receiver_upi_id: None,
            remarks: Some(format!("adding referral bonus: {}", bonus)),
            error_reason: None,
            created_ts: Some(ts),
            created_by: Some(user_id),
            updated_ts: None,
            updated_by: None,
        }
    }

    pub fn referrer_bonus_trans(
        referrer_id: u32,
        balance_before: Money,
        balance_after: Money,
        user_id: u32,
    ) -> Self {
        let ts = get_epoch_ts();
        Self {
            user_id: referrer_id,
            transaction_type: WalltetTransactionType::ReferrerBonus,
            amount: Money::new(0, REFERRER_BONUS),
            status: WalletTransactionStatus::Completed,
            balance_before,
            balance_after: Some(balance_after),
            tracking_id: None,
            receiver_upi_id: None,
            remarks: Some(format!("adding referrer bonus: {}", REFERRER_BONUS)),
            error_reason: None,
            created_ts: Some(ts),
            created_by: Some(user_id),
            updated_ts: None,
            updated_by: None,
        }
    }

    pub fn amount(&self) -> Money {
        self.amount
    }

    pub fn balance_before(&self) -> Money {
        self.balance_before
    }
}
