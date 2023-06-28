use mongodb::ClientSession;

use crate::{
    database::AppDatabase,
    handlers::wallet::helper::{insert_wallet_transaction_session, update_wallet_with_session},
    models::wallet::{Money, WalletTransaction},
};

pub async fn credit_prize_value(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    amount: Money,
    contest_id: &str,
) -> anyhow::Result<()> {
    let real = amount.real();
    let bonus = amount.bonus();
    let (balance_before, balance_after) =
        update_wallet_with_session(db, session, user_id, real, bonus, false, true).await?;
    let remarks = format!("Credit prize value {} for contest {}", amount, contest_id);
    let transaction = WalletTransaction::contest_win_trans(
        user_id,
        amount,
        balance_before,
        balance_after,
        &remarks,
    );
    insert_wallet_transaction_session(db, session, &transaction).await?;
    Ok(())
}
