use crate::core::events::Event;
use crate::core::models::{Availability, Participant, Poll, PollInstance};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollView {
    pub poll: Poll,
    pub participants: Vec<Participant>,
    pub availability: Vec<Availability>,
    pub instances: Vec<PollInstance>,
}

#[derive(Clone)]
pub struct PollsProjection {
    // Key: poll_id
    data: Arc<RwLock<HashMap<String, PollView>>>,
}

impl PollsProjection {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, poll_id: &str) -> Option<PollView> {
        self.data.read().unwrap().get(poll_id).cloned()
    }

    pub fn get_all(&self) -> Vec<Poll> {
        // Return summary list (just the poll part)
        // In a real list_polls, we usually just need the Poll struct
        self.data
            .read()
            .unwrap()
            .values()
            .map(|view| view.poll.clone())
            .collect()
    }

    pub fn apply(&self, event: Event) {
        let mut data = self.data.write().unwrap();

        match event {
            Event::V1PollCreated(e) => {
                let dates_json = serde_json::to_string(&e.dates).unwrap_or_default();

                // For MVP Projections, defaults for missing fields
                let poll = Poll {
                    id: e.id.clone(),
                    title: e.title,
                    description: e.description,
                    location: e.location,
                    // We don't have created_at in V1 event? We should add it to event or use 0
                    // Checking implementation: src/api/handlers/general.rs puts created_at in SQL
                    // But V1PollCreated definition in mod.rs MISSES created_at!
                    // This is a gap. For now use 0 or current time if we can't recover.
                    // Ideally we fix the event definition, but that breaks compat if we had strict schema.
                    // Let's assume 0 for now or todo fix.
                    created_at: chrono::Utc::now().timestamp(),
                    dates: dates_json,
                    time_range: "{}".to_string(), // Default
                    status: "active".to_string(),
                    finalized_at: None,
                    finalized_time: None,
                    notes: None,
                    organizer_id: None,    // Missing from V1
                    recurrence_rule: None, // Missing from V1 (added in SQL but maybe not event?)
                };

                let view = PollView {
                    poll,
                    participants: Vec::new(),
                    availability: Vec::new(),
                    instances: Vec::new(),
                };
                data.insert(e.id, view);
            }
            Event::V1ParticipantJoined(e) => {
                if let Some(view) = data.get_mut(&e.poll_id) {
                    let p = Participant {
                        id: e.id,
                        poll_id: e.poll_id,
                        name: e.name,
                        email: e.email,
                        access_token: Some(e.access_token),
                    };
                    view.participants.push(p);
                }
            }
            Event::V1VoteUpdated(_e) => {
                // This event is tricky because it contains name/email but not poll_id at top level?
                // Let's check VoteUpdatedV1 def.
                // It has: participant_name, participant_email, availability.
                // It MISSES poll_id! How do we know which poll?
                // The stream_id in Redb is "poll-{id}". So we know context when replaying.
                // BUT `apply` signature takes just `Event`.
                // WE NEED TO PASS STREAM_ID or ENRICH EVENT.
                // For now, let's look at the def again.
            }
            Event::V1PollFinalized(e) => {
                if let Some(view) = data.get_mut(&e.id) {
                    view.poll.status = "finalized".to_string();
                    view.poll.finalized_at = Some(e.finalized_at);
                    view.poll.finalized_time = Some(e.finalized_time);
                    view.poll.notes = e.notes;
                }
            }
            Event::V1PollDeleted(e) => {
                data.remove(&e.id);
            }
            _ => {}
        }
    }

    // Helper to apply with context if event is missing IDs
    pub fn apply_with_id(&self, poll_id: &str, event: Event) {
        // First, handle events that need special context (VoteUpdated)
        // using a write lock
        {
            let mut data = self.data.write().unwrap();
            if let Event::V1VoteUpdated(e) = &event {
                if let Some(view) = data.get_mut(poll_id) {
                    // Find participant by email (fallback to name?)
                    let participant = view.participants.iter().find(|p| {
                        p.email.as_ref() == Some(&e.participant_email)
                            || p.name == e.participant_name
                    });

                    if let Some(p) = participant {
                        let p_id = p.id.clone();
                        // Remove old availability for this participant
                        view.availability.retain(|a| a.participant_id != p_id);

                        // Add new
                        for entry in &e.availability {
                            view.availability.push(Availability {
                                id: None, // In-memory doesn't need DB ID
                                poll_id: poll_id.to_string(),
                                participant_id: p_id.clone(),
                                date: entry.date.clone(),
                                time_slot: entry.slot.clone(),
                                status: entry.status.clone(),
                            });
                        }
                    }
                }
                return; // Handled
            }
        } // Drop lock

        // For all other events, they are self-contained or handled by standard apply
        // Note: apply() takes a read/write lock internally, so we must not hold the lock here
        self.apply(event);
    }
}

// --- Users Projection ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserView {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: String,
    pub created_at: i64,
    pub phone: Option<String>,
    pub last_login: Option<i64>,
}

#[derive(Clone)]
pub struct UsersProjection {
    // Primary Key: user_id -> UserView
    data: Arc<RwLock<HashMap<String, UserView>>>,
    // Secondary Index: email -> user_id
    email_index: Arc<RwLock<HashMap<String, String>>>,
}

impl UsersProjection {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            email_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, user_id: &str) -> Option<UserView> {
        self.data.read().unwrap().get(user_id).cloned()
    }

    pub fn get_by_email(&self, email: &str) -> Option<UserView> {
        let index = self.email_index.read().unwrap();
        if let Some(user_id) = index.get(email) {
            self.get(user_id)
        } else {
            None
        }
    }

    pub fn apply(&self, event: Event) {
        let mut data = self.data.write().unwrap();
        let mut email_index = self.email_index.write().unwrap();

        match event {
            Event::V1UserRegistered(e) => {
                let user = UserView {
                    id: e.id.clone(),
                    email: e.email.clone(),
                    password_hash: e.password_hash,
                    name: e.name,
                    role: e.role,
                    created_at: e.created_at,
                    phone: e.phone,
                    last_login: None,
                };

                // Update indexes
                email_index.insert(e.email, e.id.clone());
                data.insert(e.id, user);
            }
            Event::V1UserUpdated(e) => {
                if let Some(user) = data.get_mut(&e.id) {
                    if let Some(name) = e.name {
                        user.name = name;
                    }
                    if let Some(email) = e.email {
                        // Remove old email from index
                        email_index.remove(&user.email);
                        // Update email
                        user.email = email.clone();
                        // Add new email to index
                        email_index.insert(email, e.id.clone());
                    }
                    if let Some(phone) = e.phone {
                        user.phone = Some(phone);
                    }
                }
            }
            Event::V1UserRoleUpdated(e) => {
                if let Some(user) = data.get_mut(&e.id) {
                    user.role = e.role;
                }
            }
            Event::V1UserPasswordChanged(e) => {
                if let Some(user) = data.get_mut(&e.id) {
                    user.password_hash = e.password_hash;
                }
            }
            Event::V1UserLoggedIn(e) => {
                if let Some(user) = data.get_mut(&e.id) {
                    user.last_login = Some(e.timestamp);
                }
            }
            Event::V1UserDeleted(e) => {
                data.remove(&e.id);
                email_index.remove(&e.email);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::events::{
        AvailabilityEntryV1, Event, ParticipantJoinedV1, PollCreatedV1, VoteUpdatedV1,
    };

    #[test]
    fn test_apply_with_id_handles_poll_created() {
        let projection = PollsProjection::new();
        let poll_id = "test-poll-123".to_string();
        let event = Event::V1PollCreated(PollCreatedV1 {
            id: poll_id.clone(),
            title: "Test Poll".to_string(),
            description: "Desc".to_string(),
            location: "Loc".to_string(),
            dates: vec!["2023-01-01".to_string()],
        });

        projection.apply_with_id(&poll_id, event);

        let view = projection.get(&poll_id);
        assert!(view.is_some(), "Poll should be created in projection");
        assert_eq!(view.unwrap().poll.title, "Test Poll");
    }

    #[test]
    fn test_apply_with_id_handles_vote_updated() {
        let projection = PollsProjection::new();
        let poll_id = "test-poll-123".to_string();

        // 1. Create Poll
        projection.apply(Event::V1PollCreated(PollCreatedV1 {
            id: poll_id.clone(),
            title: "Test".to_string(),
            description: "Desc".to_string(),
            location: "Loc".to_string(),
            dates: vec!["2023-01-01".to_string()],
        }));

        // 2. Add Participant via Event (Fixing Bug 2 simulation)
        projection.apply(Event::V1ParticipantJoined(ParticipantJoinedV1 {
            id: "p1".to_string(),
            poll_id: poll_id.clone(),
            name: "User".to_string(),
            email: Some("user@test.com".to_string()),
            access_token: "token".to_string(),
        }));

        // 3. Update Vote
        let event = Event::V1VoteUpdated(VoteUpdatedV1 {
            participant_name: "User".to_string(),
            participant_email: "user@test.com".to_string(),
            availability: vec![AvailabilityEntryV1 {
                date: "2023-01-01".to_string(),
                slot: "12:00".to_string(),
                status: "available".to_string(),
            }],
        });

        projection.apply_with_id(&poll_id, event);

        let view = projection.get(&poll_id).unwrap();
        assert_eq!(view.availability.len(), 1);
        assert_eq!(view.availability[0].status, "available");
    }
}
