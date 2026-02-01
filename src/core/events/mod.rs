use serde::{Deserialize, Serialize};

// Concrete Event Definitions (V1)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PollCreatedV1 {
    pub id: String,
    pub title: String,
    pub description: String,
    pub location: String,
    pub dates: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AvailabilityEntryV1 {
    pub date: String,
    pub slot: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoteUpdatedV1 {
    pub participant_name: String,
    pub participant_email: String,
    pub availability: Vec<AvailabilityEntryV1>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParticipantJoinedV1 {
    pub id: String,
    pub poll_id: String,
    pub name: String,
    pub email: Option<String>,
    pub access_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParticipantUpdatedV1 {
    pub id: String,
    pub poll_id: String,
    pub name: String,
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PollFinalizedV1 {
    pub id: String,
    pub finalized_at: i64,
    pub finalized_time: String,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserRegisteredV1 {
    pub id: String,
    pub email: String,
    pub password_hash: String, // We store the hash, not plain text!
    pub name: String,
    pub role: String,
    pub created_at: i64,
    pub phone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserUpdatedV1 {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>, // If email changes, ID stays same but index needs update
    pub phone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserRoleUpdatedV1 {
    pub id: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserPasswordChangedV1 {
    pub id: String,
    pub password_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserLoggedInV1 {
    pub id: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserDeletedV1 {
    pub id: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PollDeletedV1 {
    pub id: String,
}

// The Versioned Enum
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    V1PollCreated(PollCreatedV1),
    V1VoteUpdated(VoteUpdatedV1),
    V1ParticipantJoined(ParticipantJoinedV1),
    V1ParticipantUpdated(ParticipantUpdatedV1),
    V1PollFinalized(PollFinalizedV1),
    V1PollDeleted(PollDeletedV1),
    V1UserRegistered(UserRegisteredV1),
    V1UserUpdated(UserUpdatedV1),
    V1UserRoleUpdated(UserRoleUpdatedV1),
    V1UserPasswordChanged(UserPasswordChangedV1),
    V1UserLoggedIn(UserLoggedInV1),
    V1UserDeleted(UserDeletedV1),
    // Future: V2
}
