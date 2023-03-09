use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub enum WalltetTransactionType {
    AddBalance,
    Withdraw,
    PayForContest,
    ContestWin,
    SignupBonus,
    ReferralBonus,
    RefereeBonus,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletTransactionStatus {
    Pending,
    Completed,
    Error,
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
