use mongodb::{
    bson::{doc, Document},
    options::{FindOneAndUpdateOptions, ReturnDocument},
};

use crate::constants::*;

/// Generates the next val for a given sequence id
pub async fn get_seq_nxt_val(seq_id: &str) -> anyhow::Result<u32> {
    // let coll = client
    //     .database(DB_NAME)
    //     .collection::<Document>(COLL_SEQUENCES);
    // let filter = doc! {"_id": seq_id};
    // let update_doc = doc! {
    //   "$inc": {"val": 1}
    // };
    // let mut options = FindOneAndUpdateOptions::default();
    // options.upsert = Some(true);
    // options.return_document = Some(ReturnDocument::After);
    // let result = coll
    //     .find_one_and_update(filter, update_doc, options)
    //     .await?
    //     .ok_or(anyhow::anyhow!(
    //         "Not able to get next sequence value for {seq_id}"
    //     ))?;

    // let val = result.get_i32("val")?;
    // // corner case check, there shouldn't be any scenario when val is negative
    // if val <= 0 {
    //     return Err(anyhow::anyhow!(
    //         "Invalid sequence value received: {val} for {seq_id}"
    //     ));
    // }
    Ok(0 as u32)
}
