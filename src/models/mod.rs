use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod clip;
pub mod contest;
pub mod movie;
pub mod notification;
pub mod otp;
pub mod play_tracker;
pub mod user;
pub mod wallet;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenericResponse {
    pub success: bool,
    pub message: String,
}

pub use clip::*;
pub use contest::*;
pub use contest::*;
pub use movie::*;
pub use notification::*;
pub use otp::*;
pub use play_tracker::*;
pub use user::*;
pub use wallet::*;
