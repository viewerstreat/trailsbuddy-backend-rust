use std::sync::Arc;

use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    options::{FindOneAndUpdateOptions, ReturnDocument, UpdateModifications},
    ClientSession,
};

use crate::{
    constants::*,
    database::AppDatabase,
    models::wallet::{Money, Wallet, WalletTransaction, WalletTransactionStatus},
    utils::get_epoch_ts,
};

pub async fn get_user_wallet(
    db: &Arc<AppDatabase>,
    user_id: u32,
) -> anyhow::Result<Option<Wallet>> {
    let filter = doc! {"userId": user_id};
    let wallet = db
        .find_one::<Wallet>(DB_NAME, COLL_WALLETS, Some(filter), None)
        .await?;
    Ok(wallet)
}

pub async fn get_user_balance(
    db: &Arc<AppDatabase>,
    user_id: u32,
) -> anyhow::Result<Option<Money>> {
    let wallet = get_user_wallet(db, user_id).await?;
    let balance = wallet.and_then(|wallet| Some(wallet.balance()));
    Ok(balance)
}

pub async fn get_user_balance_session(
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

pub async fn get_wallet_transaction(
    db: &Arc<AppDatabase>,
    filter: Option<Document>,
) -> anyhow::Result<Option<WalletTransaction>> {
    let transaction = db
        .find_one::<WalletTransaction>(DB_NAME, COLL_WALLET_TRANSACTIONS, filter, None)
        .await?;
    Ok(transaction)
}

pub async fn update_wallet_with_session(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    real: u64,
    bonus: u64,
    subtract: bool,
    update_withdrawable: bool,
) -> anyhow::Result<(Money, Money)> {
    let balance_before = get_user_balance_session(db, session, user_id).await?;
    let withdrawable = if update_withdrawable { real } else { 0 };
    let wallet = if subtract {
        sub_wallet(db, session, user_id, real, bonus, withdrawable).await?
    } else {
        add_wallet(db, session, user_id, real, bonus, withdrawable).await?
    };
    let money = Money::new(real, bonus);
    let balance_after = if subtract {
        balance_before - money
    } else {
        balance_before + money
    };
    if wallet.balance() != balance_after {
        let err = anyhow::anyhow!(
            "balance_before {:?} and balance_after {:?} not matching, required balance_after {:?}",
            balance_before,
            wallet.balance(),
            balance_after
        );
        return Err(err);
    }
    Ok((balance_before, wallet.balance()))
}

async fn add_wallet(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    real: u64,
    bonus: u64,
    withdrawable: u64,
) -> anyhow::Result<Wallet> {
    let filter = doc! {"userId": user_id};
    let ts = get_epoch_ts() as i64;
    let update = doc! {
        "$inc": {
            "balance.bonus": bonus as i64,
            "balance.real": real as i64,
            "balance.withdrawable": withdrawable as i64
        },
        "$setOnInsert": {"createdTs": ts},
        "$set": {"updatedTs": ts},
    };
    find_and_modify_wallet(db, session, filter, update).await
}

async fn sub_wallet(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    real: u64,
    bonus: u64,
    withdrawable: u64,
) -> anyhow::Result<Wallet> {
    let real = real as i64;
    let bonus = bonus as i64;
    let wd = withdrawable as i64;
    let ts = get_epoch_ts() as i64;
    let filter = doc! {
        "userId": user_id,
        "balance.real": {"$gte": real},
        "balance.bonus": {"$gte": bonus}
    };
    let update = vec![
        doc! {
            "$set": {
                "balance.real": {
                    "$cond": [
                        {"$gte": ["$balance.real", real]},
                        {"$subtract": ["$balance.real", real]},
                        0
                    ]
                },
                "balance.bonus": {
                    "$cond": [
                        {"$gte": ["$balance.bonus", bonus]},
                        {"$subtract": ["$balance.bonus", bonus]},
                        0
                    ]
                },
                "balance.withdrawable": {
                    "$cond": [
                        {"$gte": ["$balance.withdrawable", wd]},
                        {"$subtract": ["$balance.withdrawable", wd]},
                        0
                    ]
                },
                "updatedTs": ts
            }
        },
        doc! {
            "$set": {
                "balance.withdrawable": {
                    "$cond":[
                        {"$gt": ["$balance.withdrawable", "$balance.real"]},
                        "$balance.real",
                        "$balance.withdrawable"
                    ]
                }
            }
        },
    ];
    find_and_modify_wallet(db, session, filter, update).await
}

async fn find_and_modify_wallet(
    db: &AppDatabase,
    session: &mut ClientSession,
    filter: Document,
    update: impl Into<UpdateModifications>,
) -> anyhow::Result<Wallet> {
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
    Ok(wallet)
}

pub async fn update_wallet_transaction_session(
    db: &AppDatabase,
    session: &mut ClientSession,
    transaction_id: &ObjectId,
    balance_after: Money,
    tracking_id: &Option<String>,
) -> anyhow::Result<()> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"_id": transaction_id};
    let update = doc! {
        "$set": {
            "balanceAfter": balance_after.to_bson()?,
            "status": WalletTransactionStatus::Completed.to_bson()?,
            "trackingId": tracking_id,
            "updatedTs": ts
        }
    };
    db.update_one_with_session(
        session,
        DB_NAME,
        COLL_WALLET_TRANSACTIONS,
        filter,
        update,
        None,
    )
    .await?;
    Ok(())
}

pub async fn insert_wallet_transaction(
    db: &Arc<AppDatabase>,
    transaction: &WalletTransaction,
) -> anyhow::Result<String> {
    let transaction_id = db
        .insert_one::<WalletTransaction>(DB_NAME, COLL_WALLET_TRANSACTIONS, &transaction, None)
        .await?;
    Ok(transaction_id)
}

pub async fn insert_wallet_transaction_session(
    db: &AppDatabase,
    session: &mut ClientSession,
    transaction: &WalletTransaction,
) -> anyhow::Result<String> {
    let transaction_id = db
        .insert_one_with_session::<WalletTransaction>(
            session,
            DB_NAME,
            COLL_WALLET_TRANSACTIONS,
            &transaction,
            None,
        )
        .await?;
    Ok(transaction_id)
}

pub async fn updated_failed_transaction(
    db: &Arc<AppDatabase>,
    user_id: u32,
    transaction_id: &ObjectId,
    error_reason: &Option<String>,
    tracking_id: &Option<String>,
) -> anyhow::Result<()> {
    let filter = doc! {"_id": transaction_id};
    let update = doc! {
        "$set": {
            "status": WalletTransactionStatus::Error.to_bson()?,
            "errorReason": error_reason,
            "trackingId": tracking_id,
            "updatedBy": user_id,
            "updatedTs": get_epoch_ts() as i64
        }
    };
    db.update_one(DB_NAME, COLL_WALLET_TRANSACTIONS, filter, update, None)
        .await?;
    Ok(())
}
