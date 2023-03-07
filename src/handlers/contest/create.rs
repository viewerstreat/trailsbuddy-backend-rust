use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ContestStatus {
    CREATED,
    ACTIVE,
    INACTIVE,
    FINISHED,
    ENDED,
}

impl Display for ContestStatus {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::CREATED => write!(f, "CREATED"),
            Self::ACTIVE => write!(f, "ACTIVE"),
            Self::INACTIVE => write!(f, "INACTIVE"),
            Self::FINISHED => write!(f, "FINISHED"),
            Self::ENDED => write!(f, "ENDED"),
        }
    }
}
