use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, ReturnDocument},
    ClientSession,
};

use crate::{
    constants::*,
    database::AppDatabase,
    models::wallet::{Money, Wallet, WalletTransaction},
    utils::get_epoch_ts,
};

pub async fn credit_prize_value(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    amount: Money,
    contest_id: &str,
) -> anyhow::Result<()> {
    let balance_before = get_user_balance(db, session, user_id).await?;
    let filter = doc! {"userId": user_id};
    let real = amount.real() as i64;
    let bonus = amount.bonus() as i64;
    let ts = get_epoch_ts() as i64;
    let update = doc! {
        "$inc": {"balance.real": real, "balance.bonus": bonus},
        "$set": {"updatedTs": ts}
    };
    let options = FindOneAndUpdateOptions::builder()
        .upsert(Some(true))
        .return_document(Some(ReturnDocument::After))
        .build();
    let wallet = db
        .find_one_and_update_with_session::<Wallet>(
            session,
            DB_NAME,
            COLL_WALLETS,
            filter,
            update,
            Some(options),
        )
        .await?
        .ok_or(anyhow::anyhow!("not able to update wallet"))?;
    let balance_after = wallet.balance();
    if balance_after != balance_before + amount {
        let err = anyhow::anyhow!(
            "balance_before {:?} and balance_after {:?} not matching",
            balance_before,
            balance_after
        );
        return Err(err);
    }
    let remarks = format!("Credit prize value {} for contest {}", amount, contest_id);
    let transaction = WalletTransaction::contest_win_trans(
        user_id,
        amount,
        balance_before,
        balance_after,
        &remarks,
    );
    db.insert_one_with_session(
        session,
        DB_NAME,
        COLL_WALLET_TRANSACTIONS,
        &transaction,
        None,
    )
    .await?;
    Ok(())
}

async fn get_user_balance(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
) -> anyhow::Result<Money> {
    let filter = doc! {"userId": user_id};
    let wallet = db
        .find_one_with_session::<Wallet>(session, DB_NAME, COLL_WALLETS, Some(filter), None)
        .await?;
    let balance = wallet
        .and_then(|wallet| Some(wallet.balance()))
        .unwrap_or_default();
    Ok(balance)
}
