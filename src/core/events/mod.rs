use serde::{Deserialize, Serialize};

// Concrete Event Definitions (V1)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoteAddedV1 {
    pub participant_email: String,
    pub date: String, // ISO8601 YYYY-MM-DD
    pub slot: String, // e.g. "18:00"
    pub vote: String, // "yes", "no", "ifneedbe"
}

// The Versioned Enum
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    V1(VoteAddedV1),
    // Future: V2(VoteAddedV2)
}
