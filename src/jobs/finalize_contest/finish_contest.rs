use futures::FutureExt;
use mongodb::{
    bson::{doc, Document},
    ClientSession,
};
use std::{cmp::Ordering, sync::Arc};

use super::{credit_prize::credit_prize_value, push_message::create_push_for_prize_win};
use crate::{
    constants::*,
    database::AppDatabase,
    handlers::{
        contest::create::{Contest, ContestStatus, PrizeSelection},
        play_tracker::model::{PlayTracker, PlayTrackerStatus},
    },
    models::wallet::Money,
    utils::{get_epoch_ts, parse_object_id},
};

pub async fn finish_contest(db: &Arc<AppDatabase>, contest: &Contest) {
    tracing::debug!("finish_contest: {:?}", contest._id);
    let result = db
        .execute_transaction(None, None, |db, session| {
            let contest = contest.clone();
            async move {
                update_contest(db, session, &contest).await?;
                Ok(())
            }
            .boxed()
        })
        .await;
    if let Err(e) = result {
        tracing::debug!("Not able to finish contest: {:?}", e);
        update_contest_error(db, contest).await;
    }
}

async fn update_contest_error(db: &Arc<AppDatabase>, contest: &Contest) {
    let Some(contest_id) = contest._id.as_ref() else {
        tracing::debug!("contest_id not present: {:?}", contest);
        return;
    };
    let Ok(oid) = parse_object_id(contest_id, "") else {
        tracing::debug!("not able to parse contest_id");
        return;
    };
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"_id": oid};
    let update = doc! {
        "$set": {"error": "not able to update contest in finish_contest", "updatedTs": ts}
    };
    let r = db
        .update_one(DB_NAME, COLL_CONTESTS, filter, update, None)
        .await;
    if let Err(e) = r {
        tracing::debug!("Not able to update error in contest: {:?}", e);
    }
}

async fn update_contest(
    db: &AppDatabase,
    session: &mut ClientSession,
    contest: &Contest,
) -> anyhow::Result<()> {
    let contest_id = contest
        ._id
        .as_ref()
        .ok_or(anyhow::anyhow!("contest_id not present"))?;
    let prize_money = Money::new(
        contest.prize_value_real_money as u64,
        contest.prize_value_bonus_money as u64,
    );
    let all_play_trackers = get_all_play_trackers(db, session, contest_id).await?;
    let total_player = all_play_trackers.len() as u32;
    let winners_count = get_winners_count(contest, total_player);
    let winners = get_winners(&all_play_trackers, winners_count);
    for winner in winners.iter() {
        credit_prize_value(db, session, winner.user_id, prize_money, contest_id).await?;
        create_push_for_prize_win(db, session, winner.user_id, &contest.title, prize_money).await?;
    }
    update_users(db, session, &all_play_trackers, &winners, prize_money).await?;
    update_play_trackers(db, session, contest_id).await?;
    update_contest_status(db, session, contest_id, &all_play_trackers, &winners).await?;

    Ok(())
}

async fn update_contest_status(
    db: &AppDatabase,
    session: &mut ClientSession,
    contest_id: &str,
    all_play_trackers: &Vec<PlayTracker>,
    winners: &Vec<PlayTracker>,
) -> anyhow::Result<()> {
    let all_play_trackers = all_play_trackers
        .into_iter()
        .map(|pt| pt.to_bson())
        .collect::<anyhow::Result<Vec<_>>>()?;
    let winners = winners
        .into_iter()
        .map(|pt| pt.to_bson())
        .collect::<anyhow::Result<Vec<_>>>()?;
    let ts = get_epoch_ts() as i64;
    let oid = parse_object_id(contest_id, "")
        .map_err(|_| anyhow::anyhow!("not able to parse contest_id"))?;
    let filter = doc! {"_id": oid};
    let update = doc! {"$set": {
        "status": ContestStatus::ENDED.to_bson()?,
        "winners": winners,
        "allPlayTrackers": all_play_trackers,
        "updatedTs": ts
    }};
    db.update_one_with_session(session, DB_NAME, COLL_CONTESTS, filter, update, None)
        .await?;

    Ok(())
}

async fn get_all_play_trackers(
    db: &AppDatabase,
    session: &mut ClientSession,
    contest_id: &str,
) -> anyhow::Result<Vec<PlayTracker>> {
    let filter = play_tracker_filter(contest_id)?;
    let mut play_trackers = db
        .find_with_session::<PlayTracker>(session, DB_NAME, COLL_PLAY_TRACKERS, Some(filter), None)
        .await?;
    let ts = get_epoch_ts();
    for pt in play_trackers.iter_mut() {
        let finish = pt.finish_ts.and(pt.updated_ts).unwrap_or(u64::MAX);
        let start = pt.start_ts.unwrap_or(ts);
        pt.time_taken = if finish >= start {
            Some((finish - start) as u32)
        } else {
            Some(u32::MAX)
        };
        pt.status = PlayTrackerStatus::ENDED;
    }
    play_trackers.sort_unstable_by(sort_play_tracker);
    for (idx, pt) in play_trackers.iter_mut().enumerate() {
        let score = pt.score.unwrap_or(0);
        if score > 0 {
            pt.rank = Some((idx + 1) as u32);
        }
    }
    Ok(play_trackers)
}

async fn update_play_trackers(
    db: &AppDatabase,
    session: &mut ClientSession,
    contest_id: &str,
) -> anyhow::Result<()> {
    let ts = get_epoch_ts() as i64;
    let filter = play_tracker_filter(contest_id)?;
    let update = doc! {
        "$set": {"updatedTs": ts, "status": PlayTrackerStatus::ENDED.to_bson()?}
    };
    db.update_many_with_session(session, DB_NAME, COLL_PLAY_TRACKERS, filter, update, None)
        .await?;

    Ok(())
}

async fn update_users(
    db: &AppDatabase,
    session: &mut ClientSession,
    all_play_trackers: &Vec<PlayTracker>,
    winners: &Vec<PlayTracker>,
    prize_money: Money,
) -> anyhow::Result<()> {
    let users = all_play_trackers
        .iter()
        .map(|pt| pt.user_id)
        .collect::<Vec<_>>();
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"id": {"$in": users}};
    let update = doc! {"$set": {"updatedTs": ts}, "$inc": {"totalPlayed": 1}};
    db.update_many_with_session(session, DB_NAME, COLL_USERS, filter, update, None)
        .await?;
    let real = prize_money.real() as i64;
    let bonus = prize_money.bonus() as i64;
    let users = winners.iter().map(|pt| pt.user_id).collect::<Vec<_>>();
    let filter = doc! {"id": {"$in": users}};
    let update = doc! {
        "$set": {"updatedTs": ts},
        "$inc": {"contestWon": 1, "totalEarning.real": real, "totalEarning.bonus": bonus}
    };
    db.update_many_with_session(session, DB_NAME, COLL_USERS, filter, update, None)
        .await?;

    Ok(())
}

fn sort_play_tracker(a: &PlayTracker, b: &PlayTracker) -> Ordering {
    match a.score.cmp(&b.score) {
        Ordering::Equal => match a.time_taken.cmp(&b.time_taken) {
            Ordering::Equal => a.start_ts.cmp(&b.start_ts),
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        },
        Ordering::Less => Ordering::Greater,
        Ordering::Greater => Ordering::Less,
    }
}

fn get_winners_count(contest: &Contest, total_player: u32) -> u32 {
    match contest.prize_selection {
        PrizeSelection::TOP_WINNERS => contest.top_winners_count.unwrap_or_default(),
        PrizeSelection::RATIO_BASED => {
            let numerator = contest.prize_ratio_numerator.unwrap_or_default();
            let denominator = contest.prize_ratio_denominator.unwrap_or(1);
            (numerator * total_player) / denominator
        }
    }
}

fn get_winners(all_play_trackers: &Vec<PlayTracker>, winners_count: u32) -> Vec<PlayTracker> {
    let winners = all_play_trackers
        .iter()
        .filter(|pt| {
            let rank = pt.rank.unwrap_or(0);
            rank > 0 && rank < winners_count
        })
        .cloned()
        .collect::<Vec<_>>();
    winners
}

fn play_tracker_filter(contest_id: &str) -> anyhow::Result<Document> {
    let filter = doc! {
        "contestId": contest_id,
        "$or": [
            {"status": PlayTrackerStatus::PAID.to_bson()?},
            {"status": PlayTrackerStatus::STARTED.to_bson()?},
            {"status": PlayTrackerStatus::FINISHED.to_bson()?},
        ]
    };
    Ok(filter)
}
