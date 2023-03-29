use mongodb::{
    bson::{doc, Document},
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use std::sync::Arc;

use crate::constants::*;

use crate::database::AppDatabase;

/// Generates the next val for a given sequence id
pub async fn get_seq_nxt_val(seq_id: &str, db: &Arc<AppDatabase>) -> anyhow::Result<u32> {
    let filter = doc! {"_id": seq_id};
    let update = doc! {"$inc": {"val": 1}};
    let mut options = FindOneAndUpdateOptions::default();
    options.upsert = Some(true);
    options.return_document = Some(ReturnDocument::After);
    let err = anyhow::anyhow!("Not able to get next sequence value for {seq_id}");
    let result = db
        .find_one_and_update::<Document>(DB_NAME, COLL_SEQUENCES, filter, update, Some(options))
        .await?
        .ok_or(err)?;
    let val = result.get_i32("val")?;
    // corner case check, there shouldn't be any scenario when val is negative
    if val <= 0 {
        let err = anyhow::anyhow!("Invalid sequence value received: {val} for {seq_id}");
        return Err(anyhow::anyhow!(err));
    }
    Ok(val as u32)
}
