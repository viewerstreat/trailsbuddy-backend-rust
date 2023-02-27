use crate::constants::*;
use futures::stream::StreamExt;
use mongodb::bson::{Bson, Document};
use mongodb::error::Result as MongoResult;
use mongodb::options::{FindOneAndUpdateOptions, FindOneOptions, FindOptions, InsertOneOptions};
use mongodb::{options::ClientOptions, Client};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;

#[cfg(test)]
use mockall::automock;

pub struct AppDatabase(Client);

#[cfg_attr(test, automock)]
impl AppDatabase {
    pub async fn new() -> MongoResult<Self> {
        // get all database parameters from environment
        // when not found in environemtn it should panic
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
        let app_db = Self(client);
        Ok(app_db)
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

        let err = anyhow::anyhow!("Not able to get the ObjectId value in string format");
        Err(err)
    }
}
