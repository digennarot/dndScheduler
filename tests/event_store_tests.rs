use dnd_scheduler::core::events::{AvailabilityEntryV1, Event, PollCreatedV1, VoteUpdatedV1};
use dnd_scheduler::core::models::PollAggregate;
use dnd_scheduler::core::store::RedbEventStore;

use uuid::Uuid;

#[tokio::test]
async fn test_event_store_persistence() {
    // 1. Setup
    let test_file = format!("test_{}.redb", Uuid::new_v4());
    let store = RedbEventStore::new(&test_file).unwrap();
    let stream_id = "test-stream-1";

    // 2. Append Event
    let event = PollCreatedV1 {
        id: "poll-1".to_string(),
        title: "Test Poll".to_string(),
        description: "Desc".to_string(),
        location: "Loc".to_string(),
        dates: vec!["2023-12-25".to_string()],
    };
    let event_enum = Event::V1PollCreated(event.clone());
    let data = bincode::serialize(&event_enum).unwrap();

    store
        .append(stream_id, &data, 0)
        .await
        .expect("Append failed");

    // 3. Read Event
    let raw_events = store.read_stream(stream_id).await.expect("Read failed");
    assert_eq!(raw_events.len(), 1);

    let loaded_event: Event = bincode::deserialize(&raw_events[0]).unwrap();
    match loaded_event {
        Event::V1PollCreated(e) => {
            assert_eq!(e.title, "Test Poll");
        }
        _ => panic!("Wrong event type"),
    }

    // 4. Cleanup
    std::fs::remove_file(test_file).ok();
}

#[tokio::test]
async fn test_poll_aggregate_reconstruction() {
    // 1. Setup
    let event1 = PollCreatedV1 {
        id: "poll-1".to_string(),
        title: "Original Title".to_string(),
        description: "Desc".to_string(),
        location: "Loc".to_string(),
        dates: vec![],
    };
    let event2 = VoteUpdatedV1 {
        participant_email: "alice@example.com".to_string(),
        participant_name: "Alice".to_string(),
        availability: vec![AvailabilityEntryV1 {
            date: "2023-12-25".to_string(),
            slot: "afternoon".to_string(),
            status: "yes".to_string(),
        }],
    };

    let history = vec![Event::V1PollCreated(event1), Event::V1VoteUpdated(event2)];

    // 2. Reconstruct
    let aggregate = PollAggregate::load_from_history(history).unwrap();

    // 3. Assert
    assert_eq!(aggregate.title, "Original Title");
    assert_eq!(aggregate.votes.len(), 1);
    assert_eq!(aggregate.votes[0].participant_email, "alice@example.com");
    assert_eq!(aggregate.version, 2);
}

#[tokio::test]
async fn test_concurrency_control() {
    let test_file = format!("test_concurrency_{}.redb", Uuid::new_v4());
    let store = RedbEventStore::new(&test_file).unwrap();
    let stream_id = "conc-stream";

    // Append version 0 -> 1
    store
        .append(stream_id, &[], 0)
        .await
        .expect("First append should work");

    // Try to append version 0 again (Should fail)
    let result = store.append(stream_id, &[], 0).await;
    assert!(result.is_err());

    // Append version 1 -> 2 (Should work)
    store
        .append(stream_id, &[], 1)
        .await
        .expect("Second append should work");

    std::fs::remove_file(test_file).ok();
}

#[tokio::test]
async fn test_aggregate_time_travel() {
    // 1. Setup History
    let event1 = PollCreatedV1 {
        id: "poll-1".to_string(),
        title: "Title V1".to_string(),
        description: "Desc".to_string(),
        location: "Loc".to_string(),
        dates: vec![],
    };
    let event2 = VoteUpdatedV1 {
        participant_email: "alice@example.com".to_string(),
        participant_name: "Alice".to_string(),
        availability: vec![AvailabilityEntryV1 {
            date: "2023-12-25".to_string(),
            slot: "am".to_string(),
            status: "yes".to_string(),
        }],
    };

    // Simulate a stream with 2 events
    let history_full = vec![Event::V1PollCreated(event1), Event::V1VoteUpdated(event2)];

    // 2. Time Travel: Load at Version 1 (only first event)
    // We take the first 1 event
    let history_v1 = history_full[0..1].to_vec();
    let aggregate_v1 = PollAggregate::load_from_history(history_v1).unwrap();

    assert_eq!(aggregate_v1.version, 1);
    assert_eq!(aggregate_v1.title, "Title V1");
    assert_eq!(aggregate_v1.votes.len(), 0); // Vote shouldn't exist yet

    // 3. Current State: Load at Version 2
    let aggregate_v2 = PollAggregate::load_from_history(history_full).unwrap();
    assert_eq!(aggregate_v2.version, 2);
    assert_eq!(aggregate_v2.votes.len(), 1);
}
