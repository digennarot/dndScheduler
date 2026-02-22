use crate::core::models::Poll;
use crate::db::DbPool;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use ics::{
    properties::{Description, DtEnd, DtStart, RRule, Status, Summary},
    Event, ICalendar,
};
use uuid::Uuid;

pub async fn export_poll_ics(
    State(pool): State<DbPool>,
    Path(poll_id): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    let poll: Poll = crate::db::queries::poll_repo::PollRepo::get_details(&pool, &poll_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(|(p, _, _, _)| p)
        .ok_or((StatusCode::NOT_FOUND, "Poll not found".to_string()))?;

    // 2. Create iCalendar
    let mut calendar = ICalendar::new("2.0", "dnd-scheduler");

    // 3. Determine Event Type
    if let Some(rrule_str) = &poll.recurrence_rule {
        // --- Recurring Event ---
        let mut event = Event::new(
            Uuid::new_v4().to_string(),
            chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string(),
        );

        // Parse Title & Description
        event.push(Summary::new(format!("D&D: {}", poll.title)));
        event.push(Description::new(poll.description.clone()));

        // RRULE
        // The stored RRULE string (e.g., "FREQ=WEEKLY;BYDAY=FR") needs to be added
        event.push(RRule::new(rrule_str.clone()));

        // DTSTART
        // For recurring events, we rely on the creating logic which used the first date in 'dates' array
        // The 'dates' field is JSON array of strings ["2023-12-01"]
        let dates: Vec<String> = serde_json::from_str(&poll.dates).unwrap_or_default();
        if let Some(first_date) = dates.first() {
            let mut start_time = "19:00".to_string();
            let mut end_time = "22:00".to_string();
            if let Ok(read_txn) = pool.begin_read() {
                if let Ok(table) = read_txn.open_table(crate::db::tables::POLL_INSTANCES_TABLE) {
                    use redb::ReadableTable;
                    if let Ok(iter) = table.iter() {
                        for result in iter {
                            if let Ok((_, v)) = result {
                                let inst: crate::core::models::PollInstance = bincode::deserialize(v.value()).unwrap();
                                if inst.poll_id == poll.id {
                                    start_time = inst.start_time;
                                    end_time = inst.end_time;
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Format: YYYYMMDDTHHMM00
            let dtstart = format!(
                "{}T{}00",
                first_date.replace("-", ""),
                start_time.replace(":", "")
            );
            event.push(DtStart::new(dtstart));

            let dtend = format!(
                "{}T{}00",
                first_date.replace("-", ""),
                end_time.replace(":", "")
            );
            event.push(DtEnd::new(dtend));
        }

        calendar.add_event(event);
    } else if poll.status == "finalized" {
        // --- Finalized One-Off Event ---
        if let (Some(_final_date), Some(final_time)) = (poll.finalized_at, &poll.finalized_time) {
            // finalized_at is just timestamp of WHEN it was finalized, not the event time
            // finalized_time is a string "YYYY-MM-DD HH:MM" ??
            // Let's check 'finalize_poll' logic.
            // payload.finalized_time is stored as TEXT.
            // Usually it's "YYYY-MM-DD HH:MM".

            let parts: Vec<&str> = final_time.split(' ').collect();
            if parts.len() >= 2 {
                let date_str = parts[0].replace("-", "");
                let time_str = parts[1].replace(":", "");

                let dtstart = format!("{}T{}00", date_str, time_str);

                let mut event = Event::new(
                    Uuid::new_v4().to_string(),
                    chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string(),
                );
                event.push(Summary::new(format!("D&D: {}", poll.title)));
                event.push(Description::new(poll.description.clone()));
                event.push(DtStart::new(dtstart));
                // No duration stored for finalized? Maybe default 3h
                event.push(Status::new("CONFIRMED"));

                calendar.add_event(event);
            }
        }
    } else {
        // --- Vote Phase / Not Finalized ---
        return Err((
            StatusCode::BAD_REQUEST,
            "Poll is not finalized or recurring".to_string(),
        ));
    }

    // 4. Return ICS content
    let start_date_slug = chrono::Utc::now().format("%Y%m%d").to_string();
    let filename = format!(
        "dnd-schedule-{}-{}.ics",
        poll.title.replace(" ", "-"),
        start_date_slug
    );

    let headers = [
        (header::CONTENT_TYPE, "text/calendar; charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{}\"", filename),
        ),
    ];

    Ok((headers, calendar.to_string()).into_response())
}
