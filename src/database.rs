use crate::constants::*;
use futures::{future::BoxFuture, stream::StreamExt};
use mongodb::{
    bson::{Bson, Document},
    error::{Result as MongoResult, UNKNOWN_TRANSACTION_COMMIT_RESULT},
    options::{
        AggregateOptions, ClientOptions, DeleteOptions, FindOneAndUpdateOptions, FindOneOptions,
        FindOptions, InsertOneOptions, SessionOptions, TransactionOptions, UpdateModifications,
        UpdateOptions,
    },
    Client, ClientSession,
};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

pub struct AppDatabase(pub Client);

#[derive(Debug)]
pub struct UpdateResult {
    pub matched_count: u64,
    pub modified_count: u64,
    pub upserted_id: Option<String>,
}

impl AppDatabase {
    pub async fn new() -> MongoResult<Self> {
        // get all database parameters from environment
        // when not found in environemtn it should panic
        // #[allow(unused_variables)]
        let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI not found in .env file");
        let min_pool = std::env::var("MONGODB_MIN_POOL_SIZE").unwrap_or_default();
        let max_pool = std::env::var("MONGODB_MAX_POOL_SIZE").unwrap_or_default();
        let min_pool = min_pool.parse::<u32>().unwrap_or(MONGO_MIN_POOL_SIZE);
        let max_pool = max_pool.parse::<u32>().unwrap_or(MONGO_MAX_POOL_SIZE);
        let timeout = Duration::from_secs(MONGO_CONN_TIMEOUT);
        // create the mongodb client options
        let mut client_options = ClientOptions::parse(uri).await?;
        client_options.max_pool_size = Some(max_pool);
        client_options.min_pool_size = Some(min_pool);
        client_options.connect_timeout = Some(timeout);
        // create the client and return Result object
        let client = Client::with_options(client_options)?;
        let app_database = Self(client);
        Ok(app_database)
    }

    pub async fn find_one<T>(
        &self,
        db: &str,
        coll: &str,
        filter: Option<Document>,
        options: Option<FindOneOptions>,
    ) -> MongoResult<Option<T>>
    where
        T: DeserializeOwned + Unpin + Send + Sync + 'static,
    {
        let coll = self.0.database(db).collection::<T>(coll);
        coll.find_one(filter, options).await
    }

    pub async fn find<T>(
        &self,
        db: &str,
        coll: &str,
        filter: Option<Document>,
        options: Option<FindOptions>,
    ) -> MongoResult<Vec<T>>
    where
        T: DeserializeOwned + Unpin + Send + Sync + 'static,
    {
        let coll = self.0.database(db).collection::<T>(coll);
        let mut cursor = coll.find(filter, options).await?;
        let mut data = vec![];
        while let Some(doc) = cursor.next().await {
            data.push(doc?);
        }
        Ok(data)
    }

    pub async fn find_one_and_update<T>(
        &self,
        db: &str,
        coll: &str,
        filter: Document,
        update: Document,
        options: Option<FindOneAndUpdateOptions>,
    ) -> MongoResult<Option<T>>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        let coll = self.0.database(db).collection::<T>(coll);
        coll.find_one_and_update(filter, update, options).await
    }

    pub async fn insert_one<T>(
        &self,
        db: &str,
        coll: &str,
        doc: &T,
        options: Option<InsertOneOptions>,
    ) -> anyhow::Result<String>
    where
        T: Serialize + 'static,
    {
        let collection = self.0.database(db).collection::<T>(coll);
        let result = collection.insert_one(doc, options).await?;
        if let Bson::ObjectId(oid) = result.inserted_id {
            return Ok(oid.to_hex());
        }

        tracing::debug!("Invalid insert_one result: {:?}", result);
        let err = anyhow::anyhow!("Not able to get the ObjectId value in string format");
        Err(err)
    }

    pub async fn update_one(
        &self,
        db: &str,
        coll: &str,
        query: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> anyhow::Result<UpdateResult> {
        let collection = self.0.database(db).collection::<Document>(coll);
        let result = collection.update_one(query, update, options).await?;
        let upserted_id = match result.upserted_id {
            None => None,
            Some(uid) => {
                let Bson::ObjectId(oid) = uid else {
                let err = anyhow::anyhow!("Not able to get the ObjectId value in string format");
                    return Err (err);
                };
                Some(oid.to_hex())
            }
        };
        let update_result = UpdateResult {
            modified_count: result.modified_count,
            matched_count: result.matched_count,
            upserted_id,
        };
        Ok(update_result)
    }

    pub async fn update_many(
        &self,
        db: &str,
        coll: &str,
        query: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> anyhow::Result<UpdateResult> {
        let collection = self.0.database(db).collection::<Document>(coll);
        let result = collection.update_many(query, update, options).await?;
        let upserted_id = match result.upserted_id {
            None => None,
            Some(uid) => {
                let Bson::ObjectId(oid) = uid else {
                let err = anyhow::anyhow!("Not able to get the ObjectId value in string format");
                    return Err (err);
                };
                Some(oid.to_hex())
            }
        };
        let update_result = UpdateResult {
            modified_count: result.modified_count,
            matched_count: result.matched_count,
            upserted_id,
        };
        Ok(update_result)
    }

    pub async fn delete_many(
        &self,
        db: &str,
        coll: &str,
        query: Document,
        options: Option<DeleteOptions>,
    ) -> MongoResult<u64> {
        let collection = self.0.database(db).collection::<Document>(coll);
        let result = collection.delete_many(query, options).await?;
        Ok(result.deleted_count)
    }

    pub async fn aggregate(
        &self,
        db: &str,
        coll: &str,
        pipeline: Vec<Document>,
        options: Option<AggregateOptions>,
    ) -> MongoResult<Vec<Document>> {
        let collection = self.0.database(db).collection::<Document>(coll);
        let mut cursor = collection.aggregate(pipeline, options).await?;
        let mut data = vec![];
        while let Some(doc) = cursor.next().await {
            data.push(doc?);
        }
        Ok(data)
    }

    pub async fn execute_transaction<F>(
        &self,
        session_options: Option<SessionOptions>,
        transaction_options: Option<TransactionOptions>,
        f: F,
    ) -> anyhow::Result<()>
    where
        F: for<'a> Fn(&'a AppDatabase, &'a mut ClientSession) -> BoxFuture<'a, anyhow::Result<()>>,
    {
        let mut session = self.0.start_session(session_options).await.unwrap();
        session.start_transaction(transaction_options).await?;
        let result = f(&self, &mut session).await;
        if result.is_err() {
            tracing::debug!("Abort transaction due to error: {:?}", result);
            session.abort_transaction().await?;
            let _ = result?;
        }

        loop {
            let commit_result = session.commit_transaction().await;
            if let Err(error) = commit_result.as_ref() {
                if error.contains_label(UNKNOWN_TRANSACTION_COMMIT_RESULT) {
                    continue;
                }
            }
            let _ = commit_result?;
            break;
        }
        Ok(())
    }

    pub async fn insert_one_with_session<T>(
        &self,
        session: &mut ClientSession,
        db: &str,
        coll: &str,
        doc: &T,
        options: Option<InsertOneOptions>,
    ) -> anyhow::Result<String>
    where
        T: Serialize + 'static,
    {
        let collection = self.0.database(db).collection::<T>(coll);
        let result = collection
            .insert_one_with_session(doc, options, session)
            .await?;
        if let Bson::ObjectId(oid) = result.inserted_id {
            return Ok(oid.to_hex());
        }

        tracing::debug!("Invalid insert_one_with_session result: {:?}", result);
        let err = anyhow::anyhow!("Not able to get the ObjectId value in string format");
        Err(err)
    }

    pub async fn find_with_session<T>(
        &self,
        session: &mut ClientSession,
        db: &str,
        coll: &str,
        filter: Option<Document>,
        options: Option<FindOptions>,
    ) -> MongoResult<Vec<T>>
    where
        T: DeserializeOwned + Unpin + Send + Sync + 'static,
    {
        let coll = self.0.database(db).collection::<T>(coll);
        let mut cursor = coll.find_with_session(filter, options, session).await?;
        let mut data = vec![];
        while let Some(doc) = cursor.next(session).await {
            data.push(doc?);
        }
        Ok(data)
    }

    pub async fn find_one_with_session<T>(
        &self,
        session: &mut ClientSession,
        db: &str,
        coll: &str,
        filter: Option<Document>,
        options: Option<FindOneOptions>,
    ) -> MongoResult<Option<T>>
    where
        T: DeserializeOwned + Unpin + Send + Sync + 'static,
    {
        let coll = self.0.database(db).collection::<T>(coll);
        coll.find_one_with_session(filter, options, session).await
    }

    pub async fn find_one_and_update_with_session<T>(
        &self,
        session: &mut ClientSession,
        db: &str,
        coll: &str,
        filter: Document,
        update: impl Into<UpdateModifications>,
        options: Option<FindOneAndUpdateOptions>,
    ) -> MongoResult<Option<T>>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        let coll = self.0.database(db).collection::<T>(coll);
        coll.find_one_and_update_with_session(filter, update, options, session)
            .await
    }

    pub async fn update_one_with_session(
        &self,
        session: &mut ClientSession,
        db: &str,
        coll: &str,
        query: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> anyhow::Result<UpdateResult> {
        let collection = self.0.database(db).collection::<Document>(coll);
        let result = collection
            .update_one_with_session(query, update, options, session)
            .await?;
        let upserted_id = match result.upserted_id {
            None => None,
            Some(uid) => {
                let Bson::ObjectId(oid) = uid else {
                let err = anyhow::anyhow!("Not able to get the ObjectId value in string format");
                    return Err (err);
                };
                Some(oid.to_hex())
            }
        };
        let update_result = UpdateResult {
            modified_count: result.modified_count,
            matched_count: result.matched_count,
            upserted_id,
        };
        Ok(update_result)
    }

    pub async fn update_many_with_session(
        &self,
        session: &mut ClientSession,
        db: &str,
        coll: &str,
        query: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> anyhow::Result<UpdateResult> {
        let collection = self.0.database(db).collection::<Document>(coll);
        let result = collection
            .update_many_with_session(query, update, options, session)
            .await?;
        let upserted_id = match result.upserted_id {
            None => None,
            Some(uid) => {
                let Bson::ObjectId(oid) = uid else {
                let err = anyhow::anyhow!("Not able to get the ObjectId value in string format");
                    return Err (err);
                };
                Some(oid.to_hex())
            }
        };
        let update_result = UpdateResult {
            modified_count: result.modified_count,
            matched_count: result.matched_count,
            upserted_id,
        };
        Ok(update_result)
    }
}
