use crate::core::events::{Event, VoteUpdatedV2};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantState {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PollAggregate {
    pub id: String,
    pub title: String,
    pub description: String,
    pub location: String,
    pub dates: Vec<String>,
    pub votes: Vec<VoteUpdatedV2>,
    pub participants: Vec<ParticipantState>,
    pub status: String,
    pub finalized_at: Option<i64>,
    pub finalized_time: Option<String>,
    pub notes: Option<String>,
    pub version: u64,
}

impl PollAggregate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&mut self, event: Event) {
        match event {
            Event::V1PollCreated(e) => {
                self.id = e.id;
                self.title = e.title;
                self.description = e.description;
                self.location = e.location;
                self.dates = e.dates;
                self.status = "active".to_string(); // Default status
                self.version += 1;
            }
            Event::V2PollCreated(e) => {
                self.id = e.id;
                self.title = e.title;
                self.description = e.description;
                self.location = e.location;
                self.dates = e.dates;
                self.status = "active".to_string(); // Default status
                self.version += 1;
            }
            Event::V1VoteUpdated(e) => {
                let v2_equiv = VoteUpdatedV2 {
                    participant_name: e.participant_name,
                    participant_email: Some(e.participant_email),
                    availability: e.availability,
                };
                if let Some(existing) = self
                    .votes
                    .iter_mut()
                    .find(|v| v.participant_email == v2_equiv.participant_email)
                {
                    *existing = v2_equiv;
                } else {
                    self.votes.push(v2_equiv);
                }
                self.version += 1;
            }
            Event::V2VoteUpdated(e) => {
                // If participant already voted, replace their vote
                if let Some(existing) = self
                    .votes
                    .iter_mut()
                    .find(|v| v.participant_email == e.participant_email)
                {
                    *existing = e;
                } else {
                    self.votes.push(e);
                }
                self.version += 1;
            }
            Event::V1ParticipantJoined(e) => {
                self.participants.push(ParticipantState {
                    id: e.id,
                    name: e.name,
                    email: e.email,
                });
                self.version += 1;
            }
            Event::V1ParticipantUpdated(e) => {
                if let Some(p) = self.participants.iter_mut().find(|p| p.id == e.id) {
                    p.name = e.name;
                    p.email = e.email;
                }
                self.version += 1;
            }
            Event::V1PollFinalized(e) => {
                self.status = "finalized".to_string();
                self.finalized_at = Some(e.finalized_at);
                self.finalized_time = Some(e.finalized_time);
                self.notes = e.notes;
                self.version += 1;
            }
            // User events do not affect PollAggregate
            _ => {}
        }
    }

    pub fn load_from_history(events: Vec<Event>) -> Result<Self> {
        let mut aggregate = Self::default();
        for event in events {
            aggregate.apply(event);
        }
        Ok(aggregate)
    }
}
