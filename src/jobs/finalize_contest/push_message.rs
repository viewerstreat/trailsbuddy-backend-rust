use mongodb::ClientSession;
use std::collections::HashMap;

use crate::{
    constants::*, database::AppDatabase, handlers::wallet::model::Money,
    jobs::notification::notification_req::NotificationReq,
};

pub async fn create_push_for_prize_win(
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
