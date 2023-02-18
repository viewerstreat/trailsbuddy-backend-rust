use mongodb::error::Result as MongoResult;
use mongodb::{options::ClientOptions, Client};
use std::time::Duration;

use crate::constants::*;

pub async fn get_db() -> MongoResult<Client> {
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
    Client::with_options(client_options)
}

#[cfg(test)]
mod tests {

    use mongodb::bson::{doc, Document};

    use super::*;

    #[tokio::test]
    #[should_panic]
    async fn test_get_db_no_env_should_panic() {
        let _client = get_db().await;
    }

    #[tokio::test]
    async fn test_get_db() {
        dotenvy::dotenv().ok();
        let client = get_db().await;
        assert_eq!(client.is_ok(), true);
    }

    #[tokio::test]
    async fn test_get_db_sample_read() {
        dotenvy::dotenv().ok();
        // ======================================================================================
        // For this test to pass there must be a collection "sample" in "treatviewertest" database
        // with one document inside with "message" field.
        // ======================================================================================
        let client = get_db().await.unwrap();
        let db = client.database("treatviewerstest");
        let collection = db.collection::<Document>("sample");
        let filter = doc! {};
        let item = collection.find_one(filter, None).await.unwrap().unwrap();
        assert_eq!(item.get("_id").is_some(), true);
        assert_eq!(item.get("message").is_some(), true);
    }
}
