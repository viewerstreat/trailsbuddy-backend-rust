use std::collections::HashMap;

use mongodb::{bson::doc, ClientSession};

use crate::{
    constants::*,
    database::AppDatabase,
    handlers::wallet::helper::{insert_wallet_transaction_session, update_wallet_with_session},
    models::{
        contest::{Contest, ContestStatus},
        notification::NotificationReq,
        play_tracker::PlayTracker,
        wallet::{Money, WalletTransaction},
    },
    utils::{get_epoch_ts, parse_object_id},
};

use super::finish_contest::update_play_trackers;

pub async fn cancel_contest(
    db: &AppDatabase,
    session: &mut ClientSession,
    contest: &Contest,
    all_play_trackers: &Vec<PlayTracker>,
) -> anyhow::Result<()> {
    let contest_id = contest
        ._id
        .as_ref()
        .ok_or(anyhow::anyhow!("contest_id not present"))?;
    if contest.status != ContestStatus::ACTIVE {
        return Err(anyhow::anyhow!("Contest is not in ACTIVE status"));
    }
    for play_tracker in all_play_trackers {
        refund_entry_fee(db, session, contest, play_tracker).await?;
    }
    update_play_trackers(db, session, contest_id).await?;
    update_contest_status(db, session, contest, all_play_trackers).await?;
    Ok(())
}

async fn refund_entry_fee(
    db: &AppDatabase,
    session: &mut ClientSession,
    contest: &Contest,
    play_tracker: &PlayTracker,
    total_players: u32,
) -> anyhow::Result<()> {
    let contest_id = contest
        ._id
        .as_ref()
        .ok_or(anyhow::anyhow!("contest_id not present"))?;
    let entry_fee = contest.props.entry_fee;
    if entry_fee == 0 {
        return Ok(());
    }
    let real = play_tracker
        .paid_amount
        .as_ref()
        .and_then(|m| Some(m.real()))
        .unwrap_or_default();
    let bonus = play_tracker
        .paid_amount
        .as_ref()
        .and_then(|m| Some(m.bonus()))
        .unwrap_or_default();
    let amount = Money::new(real, bonus);
    let user_id = play_tracker.user_id;
    let (balance_before, balance_after) =
        update_wallet_with_session(db, session, user_id, real, bonus, false, false).await?;
    let transaction = WalletTransaction::refundContestEntryFeeTrans(
        user_id,
        contest_id,
        amount,
        balance_before,
        balance_after,
    );
    insert_wallet_transaction_session(db, session, &transaction).await?;
    Ok(())
}

async fn create_push_for_cancel(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    contest_title: &str,
    amount: Money,
) -> anyhow::Result<()> {
    let mut data = HashMap::new();
    data.insert("userId".into(), user_id.to_string());
    data.insert("amount".into(), amount.to_string());
    data.insert("title".into(), contest_title.to_string());
    let notification_req = NotificationReq::new(user_id, EVENT_CREDIT_PRIZE, data);
    db.insert_one_with_session::<NotificationReq>(
        session,
        DB_NAME,
        COLL_NOTIFICATION_REQUESTS,
        &notification_req,
        None,
    )
    .await?;

    Ok(())
}

async fn update_contest_status(
    db: &AppDatabase,
    session: &mut ClientSession,
    contest: &Contest,
    all_play_trackers: &Vec<PlayTracker>,
) -> anyhow::Result<()> {
    let all_play_trackers = all_play_trackers
        .into_iter()
        .map(|pt| pt.to_bson())
        .collect::<anyhow::Result<Vec<_>>>()?;
    let ts = get_epoch_ts() as i64;
    let contest_id = contest
        ._id
        .as_ref()
        .ok_or(anyhow::anyhow!("contest_id not present"))?;
    let oid = parse_object_id(contest_id, "")
        .map_err(|_| anyhow::anyhow!("not able to parse contest_id"))?;
    let filter = doc! {"_id": oid};
    let update = doc! {"$set": {
        "status": ContestStatus::CANCELLED.to_bson()?,
        "allPlayTrackers": all_play_trackers,
        "updatedTs": ts
    }};
    db.update_one_with_session(session, DB_NAME, COLL_CONTESTS, filter, update, None)
        .await?;

    Ok(())
}
