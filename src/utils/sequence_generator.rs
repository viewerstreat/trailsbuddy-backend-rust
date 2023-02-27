use mockall_double::double;
use mongodb::{
    bson::{doc, Document},
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use std::sync::Arc;

use crate::constants::*;

#[double]
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

#[cfg(test)]
mod tests {

    use mockall::predicate::{eq, function};

    use super::*;

    #[tokio::test]
    async fn test_get_seq_nxt_val() {
        let seq_id = "TEST_SEQ_ID";
        let filter = doc! {"_id": seq_id};
        let update = doc! {"$inc": {"val": 1}};
        let mut options = FindOneAndUpdateOptions::default();
        options.upsert = Some(true);
        options.return_document = Some(ReturnDocument::After);
        let check_options = function(|options: &Option<FindOneAndUpdateOptions>| {
            options
                .as_ref()
                .and_then(|option| {
                    option
                        .return_document
                        .as_ref()
                        .and_then(|rd| match rd {
                            ReturnDocument::After => Some(()),
                            _ => None,
                        })
                        .and(option.upsert)
                        .and_then(|upsert| if upsert { Some(()) } else { None })
                })
                .is_some()
        });
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one_and_update::<Document>()
            .with(
                eq(DB_NAME),
                eq(COLL_SEQUENCES),
                eq(filter),
                eq(update),
                check_options,
            )
            .times(1)
            .returning(move |_, _, _, _, _| Ok(Some(doc! {"val": 5})));

        let db = Arc::new(mock_db);
        let result = get_seq_nxt_val(seq_id, &db).await.unwrap();
        assert_eq!(result, 5);
    }
}
