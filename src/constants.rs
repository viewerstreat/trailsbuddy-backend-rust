pub const REQUEST_TIMEOUT_SECS: u64 = 30;
pub const MONGO_MIN_POOL_SIZE: u32 = 5;
pub const MONGO_MAX_POOL_SIZE: u32 = 10;
pub const MONGO_CONN_TIMEOUT: u64 = 10;
pub const DEFAULT_QUERY_LIMIT: u64 = 1000;
pub const OTP_LENGTH: u32 = 6;
pub const REFERRAL_CODE_LEN: usize = 8;
pub const OTP_VALIDITY_MINS: u64 = 10;
pub const FIREBASE_SERVICE_CLIENT_EMAIL: &str =
    "firebase-adminsdk-xjwbm@trailsbuddy-1-3fbd5.iam.gserviceaccount.com";
pub const FIREBASE_SERVICE_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDGlFbWFP6s9vyF\nOU+gOoqHRDVRJfEhPi7Ry9lzoiC2f27AcJ64G9tf7c9Z1yFbXi6ocmxbuwKt20RZ\n10mVi1lBchfrMeIyZpLCLJ/us3nshwQyXvFDQQtiiTwkVlc2rtdrnFWiXRWW5Apx\njmbIh2bRgPK7hqD6vOvgWQnvwrl6ccnipaVU0nOzkmn/vb+aG+WS8oiocmsyO0Y8\nSLMq0TxEMOfu7rpu6+Xfy19ctnqAGUgXKNE9fIncGXeP0jC2ZmX+TwqzpBZD8VLt\nEnojDaI5ClnF6/ei+bIzPp94NP3PiteO36Pff+QAfYI2fkuprnDfh+Qr99+Vm29g\nRABNDfB3AgMBAAECggEANIKxW7b9iVtedxQqnCIe05oTxzuTQckhtFSFUfCDWf7k\nmHqvXdvv7LQD6qvapECJcf2f7bnMAZFDx8YILUVF+upirMzqFY3OLQ6D1CkdipBB\nadh1T+V5Tzse7jTupwUg7dHPSzn2JYpzwId5Ynl7lNbWWQRGuUcP0Kl62S3Swi+x\nZW5wyIlbaZcdIpUMLZFbBLOsI+KiVVK9Q3s9qrUQ9xK5RCzMTEuscAbs2YKHviEI\nERCaSV8niA81hnFDyqn2Mm19BemEK1APrm/daajzziH222EyGH46VaekrvhiEOb0\nxOXVs+S9VYUUMX0j83HTGSwJvDuQ1sRJ0bXHECyaUQKBgQDsOcgVswXjl7CKmRKz\ntEKbwR83sPNeaF7i8j18MIW0useZuqrMOWEAUOqQhY9m6OEUFSdhCgwRKXw88m0x\n3GVAT7FPQJmXM6Y29p85WHDrVEaswBcNpsS+siZgDdzYCDe6Djhcd84F3wIlfmHF\nVbEl0UsEnRzgAh6KXCPzZHqs5QKBgQDXM9BhlntJqWQXr+fHK0jPEBgkReUfKmUy\ntySTgr+kjAudsbV2L0eVlifKIvFJpZEKHOGCubnJZPYTB+O09nWJ1LuUdcBHvqjW\njl918slsMKMaheefid3BEvvo7j5AaI++o3KQNLHwLa1xmTfU77hMsgODwqgbhRze\nEsFou37uKwKBgQCpMYmZ4Suqo48S9ihrBgVfQad2YAsv51lu+0oGlUpu9AjaltSW\nidJsQ4h+EutvLgVoOO6HloamFCykCo8jU1RCB9JbjU10+s1mOKY6kJnwM+CbAsqA\nQJ5SZ48M5WD0ao5feKftsvGhSuVirW6hxIqpJ4qvt0hjOaFeQDiPr7wd8QKBgA37\nXdSZVFVK3ifz09lK5KYfY5InwGUv+fc7kvLKkez89FxAiYuuMrZzVQ57CrZAPZYs\nnjJCIuIE30AJSTAeuzBDVBSnOeDvcETQZz9gkNmop1A31v60lGXQ9/EAWacRpBU8\nxVq9MbprHVO+IrSBBrZk8nmDEi0HjwKWsV4+oFaVAoGBAMUI+cefA+aq9gfU1GAu\n1+VTbG6ua6gIruE9E4I+LZCt8+Q6ARmMC7ppkVrWWQdkTR+ifZFLzGzFgcnDgCuW\nghBVYf/CfPPAV8ADprsxX9XNepMUWfhW1ErEu5rNT647aMBQbl8c+XnMoHS/f4qm\nd9eDx/EWDN4i73FMhyL7euFh\n-----END PRIVATE KEY-----\n";
pub const FIREBASE_MESSAGE_SCOPE: &str = "https://www.googleapis.com/auth/firebase.messaging";
pub const GOOGLE_JWKS_URI: &str = "https://www.googleapis.com/oauth2/v3/certs";
pub const GOOGLE_TOKEN_URL: &str = "https://www.googleapis.com/oauth2/v4/token";
pub const FB_ME_URL: &str = "https://graph.facebook.com/me";
pub const FCM_ENDPOINT: &str =
    "https://fcm.googleapis.com/v1/projects/trailsbuddy-1-3fbd5/messages:send";
pub const AWS_REGION: &str = "ap-south-1";
pub const AWS_BUCKET: &str = "trailsbuddy-1";
pub const MULTIPART_BODY_LIMIT: usize = 100 * 1024 * 1024;
pub const WITHDRAW_BAL_MIN_AMOUNT: u64 = 10;
pub const FINALIZE_CONTEST_JOB_INTERVAL: u64 = 5 * 60;
pub const NOTIFICATION_JOB_INTERVAL: u64 = 2 * 60;
pub const CLEANUP_JOB_INTERVAL: u64 = 24 * 60 * 60;
pub const USED_TOKEN_RETENTION: u64 = 10;
pub const OTP_RETENTION: u64 = 10;
pub const PUSH_ICON_COLOR: &str = "#EA3333";
pub const PUSH_MSG_LOGO_PATH: &str =
    "https://trailsbuddy-1.s3.ap-south-1.amazonaws.com/1657115801399-441.jpeg";
pub const PUSH_MESSAGE_TITLE: &str = "Trailsbuddy";
pub const NOTI_JOB_MAX_RETRY_COUNT: u32 = 10;
pub const NOTI_JOB_FETCH_LIMIT: i64 = 10;
pub const REFERRER_BONUS: u64 = 100;
pub const REFERRAL_BONUS: u64 = 100;

pub const DB_NAME: &str = "treatviewerstest";

pub const COLL_SEQUENCES: &str = "sequences";
pub const COLL_CLIPS: &str = "clips";
pub const COLL_MOVIES: &str = "movies";
pub const COLL_USERS: &str = "users";
pub const COLL_OTP: &str = "otps";
pub const COLL_USED_TOKENS: &str = "usedTokens";
pub const COLL_NOTIFICATIONS: &str = "notifications";
pub const COLL_CONTESTS: &str = "contests";
pub const COLL_PLAY_TRACKERS: &str = "playTrackers";
pub const COLL_WALLETS: &str = "wallets";
pub const COLL_WALLET_TRANSACTIONS: &str = "walletTransactions";
pub const COLL_NOTIFICATION_REQUESTS: &str = "notificationRequests";
pub const COLL_NOTIFICATION_CONTENTS: &str = "notificationContents";
pub const COLL_SPECIAL_REFERRAL_CODES: &str = "specialReferralCodes";

pub const USER_ID_SEQ: &str = "USER_ID_SEQ";

pub const EVENT_CREDIT_PRIZE: &str = "EVENT_CREDIT_PRIZE";
