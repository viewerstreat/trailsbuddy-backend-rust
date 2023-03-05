pub const REQUEST_TIMEOUT_SECS: u64 = 30;
pub const MONGO_MIN_POOL_SIZE: u32 = 5;
pub const MONGO_MAX_POOL_SIZE: u32 = 10;
pub const MONGO_CONN_TIMEOUT: u64 = 10;
pub const DEFAULT_QUERY_LIMIT: u64 = 1000;
pub const OTP_LENGTH: u32 = 6;
pub const OTP_VALIDITY_MINS: u64 = 10;
pub const GOOGLE_JWKS_URI: &str = "https://www.googleapis.com/oauth2/v3/certs";
pub const FB_ME_URL: &str = "https://graph.facebook.com/me";

pub const DB_NAME: &str = "treatviewers";

pub const COLL_SEQUENCES: &str = "sequences";
pub const COLL_CLIPS: &str = "clips";
pub const COLL_MOVIES: &str = "movies";
pub const COLL_USERS: &str = "users";
pub const COLL_OTP: &str = "otps";
pub const COLL_USED_TOKENS: &str = "usedTokens";
pub const COLL_NOTIFICATIONS: &str = "notifications";

pub const USER_ID_SEQ: &str = "USER_ID_SEQ";
