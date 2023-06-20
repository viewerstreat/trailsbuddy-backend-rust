# trailsbuddy-backend-rust
The backend web server for Trailsbuddy app written in Rust.


# Contents of .env
- RUST_LOG
- PORT
- MONGODB_URI
- MONGODB_MIN_POOL_SIZE
- MONGODB_MAX_POOL_SIZE
- JWT_SECRET_KEY
- JWT_EXPIRY
- REFRESH_TOKEN_EXPIRY
- AWS_ACCESS_KEY_ID
- AWS_SECRET_ACCESS_KEY
- AWS_REGION
- APP_UPI_ID

# DB Indexes to be created
```
db.users.createIndex({"id": 1}, {"unique": true});
db.clips.createIndex({"name": 1}, {"unique": true});
db.movies.createIndex({"name": 1}, {"unique": true});
db.contests.createIndex({"title": 1}, {"unique": true});
db.playTrackers.createIndex({"contestId": 1, "userId": 1}, {"unique": true});
db.wallets.createIndex({"userId": 1}, {"unique": true});
db.walletTransactions.createIndex({"userId": 1});
db.notifications.createIndex({"userId": 1});
db.notificationRequests.createIndex({"userId": 1});
db.notificationRequests.createIndex({"status": 1});
```
