 // @ts-ignore
use treatviewerstest
db.users.dropIndexes();
db.clips.dropIndexes();
db.movies.dropIndexes();
db.contests.dropIndexes();
db.playTrackers.dropIndexes();
db.wallets.dropIndexes();
db.walletTransactions.dropIndexes();
db.notifications.dropIndexes();
db.notificationRequests.dropIndexes();
db.specialReferralCodes.dropIndexes();
db.adminUsers.dropIndexes();

db.users.deleteMany({});
db.clips.deleteMany({});
db.movies.deleteMany({});
db.contests.deleteMany({});
db.playTrackers.deleteMany({});
db.wallets.deleteMany({});
db.walletTransactions.deleteMany({});
db.notifications.deleteMany({});
db.notificationRequests.deleteMany({});
db.specialReferralCodes.deleteMany({});
db.adminUsers.deleteMany({});


db.users.createIndex({ "id": 1 }, { "unique": true });
db.users.createIndex({ "referralCode": 1 }, { "unique": true });
db.users.createIndex({ "phone": 1 });
db.clips.createIndex({ "name": 1 }, { "unique": true });
db.movies.createIndex({ "name": 1 }, { "unique": true });
db.contests.createIndex({ "title": 1 }, { "unique": true });
db.playTrackers.createIndex({ "contestId": 1, "userId": 1 }, { "unique": true });
db.wallets.createIndex({ "userId": 1 }, { "unique": true });
db.walletTransactions.createIndex({ "userId": 1 });
db.notifications.createIndex({ "userId": 1 });
db.notificationRequests.createIndex({ "userId": 1 });
db.notificationRequests.createIndex({ "status": 1 });
db.specialReferralCodes.createIndex({ "referralCode": 1 }, { "unique": true });
db.adminUsers.createIndex({ "phone": 1 }, { "unique": true });
