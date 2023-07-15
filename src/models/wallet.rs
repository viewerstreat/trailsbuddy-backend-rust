use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use utoipa::ToSchema;

use crate::{constants::*, utils::get_epoch_ts};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WalltetTransactionType {
    #[default]
    AddBalance,
    Withdraw,
    PayForContest,
    ContestWin,
    SignupBonus,
    ReferralBonus,
    ReferrerBonus,
    RefundContestEntryFee,
}

impl WalltetTransactionType {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WalletTransactionStatus {
    #[default]
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

#[derive(Debug, Default, Serialize, Deserialize)]
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
        let mut transaction = Self::default();
        transaction.user_id = user_id;
        transaction.transaction_type = WalltetTransactionType::AddBalance;
        transaction.amount = amount;
        transaction.balance_before = balance_before;
        transaction.created_ts = Some(ts);
        transaction.created_by = Some(user_id);
        transaction
    }

    pub fn withdraw_bal_init_trans(
        user_id: u32,
        amount: Money,
        balance_before: Money,
        receiver_upi_id: &str,
    ) -> Self {
        let ts = get_epoch_ts();
        let mut transaction = Self::default();
        transaction.user_id = user_id;
        transaction.transaction_type = WalltetTransactionType::Withdraw;
        transaction.amount = amount;
        transaction.balance_before = balance_before;
        transaction.receiver_upi_id = Some(receiver_upi_id.to_string());
        transaction.created_ts = Some(ts);
        transaction.created_by = Some(user_id);
        transaction
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
        let mut transaction = Self::default();
        transaction.user_id = user_id;
        transaction.transaction_type = WalltetTransactionType::PayForContest;
        transaction.amount = amount;
        transaction.status = WalletTransactionStatus::Completed;
        transaction.balance_before = balance_before;
        transaction.balance_after = Some(balance_after);
        transaction.remarks = Some(format!("Pay for contest: {}", contest_id));
        transaction.created_ts = Some(ts);
        transaction.created_by = Some(user_id);
        transaction
    }

    pub fn contest_win_trans(
        user_id: u32,
        amount: Money,
        balance_before: Money,
        balance_after: Money,
        remarks: &str,
    ) -> Self {
        let mut transaction = Self::default();
        transaction.user_id = user_id;
        transaction.transaction_type = WalltetTransactionType::ContestWin;
        transaction.amount = amount;
        transaction.status = WalletTransactionStatus::Completed;
        transaction.balance_before = balance_before;
        transaction.balance_after = Some(balance_after);
        transaction.remarks = Some(remarks.into());
        transaction.created_ts = Some(get_epoch_ts());
        transaction.created_by = Some(user_id);
        transaction
    }

    pub fn referral_bonus_trans(
        user_id: u32,
        bonus: u64,
        balance_before: Money,
        balance_after: Money,
    ) -> Self {
        let mut transaction = Self::default();
        transaction.user_id = user_id;
        transaction.transaction_type = WalltetTransactionType::ReferralBonus;
        transaction.amount = Money::new(0, bonus);
        transaction.status = WalletTransactionStatus::Completed;
        transaction.balance_before = balance_before;
        transaction.balance_after = Some(balance_after);
        transaction.remarks = Some(format!("adding referral bonus: {}", bonus));
        transaction.created_ts = Some(get_epoch_ts());
        transaction.created_by = Some(user_id);
        transaction
    }

    pub fn referrer_bonus_trans(
        referrer_id: u32,
        balance_before: Money,
        balance_after: Money,
        user_id: u32,
    ) -> Self {
        let mut transaction = Self::default();
        transaction.user_id = referrer_id;
        transaction.transaction_type = WalltetTransactionType::ReferrerBonus;
        transaction.amount = Money::new(0, REFERRER_BONUS);
        transaction.status = WalletTransactionStatus::Completed;
        transaction.balance_before = balance_before;
        transaction.balance_after = Some(balance_after);
        transaction.remarks = Some(format!("adding referrer bonus: {}", REFERRER_BONUS));
        transaction.created_ts = Some(get_epoch_ts());
        transaction.created_by = Some(user_id);
        transaction
    }

    pub fn refund_contest_entry_fee_trans(
        user_id: u32,
        contest_id: &str,
        amount: Money,
        balance_before: Money,
        balance_after: Money,
    ) -> Self {
        let mut transaction = Self::default();
        transaction.user_id = user_id;
        transaction.transaction_type = WalltetTransactionType::RefundContestEntryFee;
        transaction.status = WalletTransactionStatus::Completed;
        transaction.amount = amount;
        transaction.balance_before = balance_before;
        transaction.balance_after = Some(balance_after);
        transaction.remarks = Some(format!(
            "refund for contest: {}, amount: {}",
            contest_id, amount
        ));
        transaction.created_ts = Some(get_epoch_ts());
        transaction
    }

    pub fn amount(&self) -> Money {
        self.amount
    }

    pub fn balance_before(&self) -> Money {
        self.balance_before
    }
}
