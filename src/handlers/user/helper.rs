use helper_inner::{check_uniq_email, check_uniq_phone};
use mockall_double::double;

#[cfg(test)]
use mockall::automock;

#[double]
use crate::database::AppDatabase;

#[cfg_attr(test, automock)]
pub mod helper_inner {
    use std::sync::Arc;

    use mockall_double::double;
    use mongodb::bson::{doc, Document};

    #[double]
    use crate::database::AppDatabase;
    use crate::{constants::*, utils::AppError};

    // check if the given phone already exists in users collection
    pub async fn check_uniq_phone(db: &Arc<AppDatabase>, phone: &str) -> Result<(), AppError> {
        let filter = Some(doc! {"phone": phone});
        let result = db
            .find_one::<Document>(DB_NAME, COLL_USERS, filter, None)
            .await?;
        if result.is_some() {
            let err = format!("User already exists with same phone: {}", phone);
            let err = AppError::BadRequestErr(err);
            return Err(err);
        }

        Ok(())
    }

    // check if the given email already exists in the users collection
    pub async fn check_uniq_email(db: &Arc<AppDatabase>, email: &str) -> Result<(), AppError> {
        let filter = Some(doc! {"email": email});
        let result = db
            .find_one::<Document>(DB_NAME, COLL_USERS, filter, None)
            .await?;
        if result.is_some() {
            let err = format!("User already exists with same email: {}", email);
            let err = AppError::BadRequestErr(err);
            return Err(err);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::{eq, function};
    use mongodb::{
        bson::{doc, Document},
        options::FindOneOptions,
    };

    use super::*;

    use crate::{constants::*, utils::AppError};

    #[tokio::test]
    async fn test_check_uniq_phone() {
        let phone = "1234567890";
        let filter = Some(doc! {"phone": phone});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<Document>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(filter), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(None));
        let db = Arc::new(mock_db);
        let _ = check_uniq_phone(&db, phone).await.unwrap();
    }

    #[tokio::test]
    async fn test_check_uniq_phone_exists() {
        let phone = "1234567890";
        let filter = Some(doc! {"phone": phone});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<Document>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(filter), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(Some(doc! {"id": 1})));
        let db = Arc::new(mock_db);
        let result = check_uniq_phone(&db, phone).await;
        assert_eq!(result.is_err(), true);
        let msg = format!("User already exists with same phone: {}", phone);
        let result = result.err().unwrap();
        if let AppError::BadRequestErr(err) = result {
            assert_eq!(err, msg);
        } else {
            panic!("AppError::BadRequestErr should be received");
        }
    }

    #[tokio::test]
    async fn test_check_uniq_email() {
        let email = "testemail@email.com";
        let filter = Some(doc! {"email": email});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<Document>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(filter), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(None));
        let db = Arc::new(mock_db);
        let _ = check_uniq_email(&db, email).await.unwrap();
    }

    #[tokio::test]
    async fn test_check_uniq_email_exists() {
        let email = "testemail@email.com";
        let filter = Some(doc! {"email": email});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<Document>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(filter), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(Some(doc! {"id": 1})));
        let db = Arc::new(mock_db);
        let result = check_uniq_email(&db, email).await;
        assert_eq!(result.is_err(), true);
        let msg = format!("User already exists with same email: {}", email);
        let result = result.err().unwrap();
        if let AppError::BadRequestErr(err) = result {
            assert_eq!(err, msg);
        } else {
            panic!("AppError::BadRequestErr should be received");
        }
    }
}
