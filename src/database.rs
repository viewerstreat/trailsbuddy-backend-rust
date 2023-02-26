use crate::constants::*;
use mongodb::bson::Document;
use mongodb::error::Result as MongoResult;
use mongodb::options::FindOneOptions;
use mongodb::{options::ClientOptions, Client};
use serde::de::DeserializeOwned;
use std::{sync::Arc, time::Duration};

#[cfg(test)]
use mockall::automock;

pub struct AppDatabase(Client);
pub type AppDB = Arc<AppDatabase>;

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
}
